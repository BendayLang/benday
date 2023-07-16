use std::f64::consts::{PI, TAU};
use crate::input::{Input, KeyState};
use crate::widgets::{Widget, HOVER, PUSH, Orientation, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA};
use crate::custom_rect::Rect;
use nalgebra::{Point2, Vector2};
use crate::vector2::Vector2Plus;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::ttf::FontStyle;

use crate::camera::Camera;
use crate::color::{darker, paler, Colors, with_alpha};
use crate::primitives::{draw_circle, draw_polygon, draw_rect, draw_rounded_rect, draw_text, fill_circle, fill_polygon, fill_rect, fill_rounded_rect};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use sdl2::video::Window;


pub struct SliderStyle {
	filled_track_color: Color,
	empty_track_color: Color,
	thumb_color: Color,
	thumb_hovered_color: Color,
	thumb_pushed_color: Color,
	focused_color: Color,
	border_color: Color,
}

impl Default for SliderStyle {
	fn default() -> Self {
		Self {
			filled_track_color: Colors::LIGHT_GREY,
			empty_track_color: Colors::LIGHTER_GREY,
			thumb_color: Colors::LIGHT_GREY,
			thumb_hovered_color: darker(Colors::LIGHT_GREY, HOVER),
			thumb_pushed_color: darker(Colors::LIGHT_GREY, PUSH),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}

impl SliderStyle {
	pub fn new(track_color: Color, thumb_color: Color) -> Self {
		let empty_track_color = paler(track_color, 0.6);
		Self {
			filled_track_color: track_color,
			empty_track_color,
			thumb_color,
			thumb_hovered_color: darker(thumb_color, HOVER),
			thumb_pushed_color: darker(thumb_color, PUSH),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}

/// A slider can be:
///
/// **discrete** (with a number of **snap** points) or **continuous**
/// , It has:
///
/// a **default value**
///
/// a **display** function that says if and how the value should be displayed
pub enum SliderType {
	Discrete { snap: u32, default_value: u32, display: Option<Box<dyn Fn(u32) -> String>> },
	Continuous { default_value: f32, display: Option<Box<dyn Fn(f32) -> String>> },
}

/// A slider is a widget that can be dragged to change a value.
///
/// It can be discrete or continuous
pub struct Slider {
	rect: Rect,
	state: KeyState,
	has_camera: bool,
	/// Internal value of the slider (0.0 - 1.0)
	value: f32,
	style: SliderStyle,
	orientation: Orientation,
	slider_type: SliderType,
}

impl Slider {
	const SPEED: f32 = 2.5;
	
	pub fn new(rect: Rect, style: SliderStyle, slider_type: SliderType, has_camera: bool) -> Self {
		let orientation = if rect.width() > rect.height() { Orientation::Horizontal } else { Orientation::Vertical };
		Self {
			rect,
			style,
			orientation,
			state: KeyState::new(),
			value: match slider_type {
				SliderType::Discrete { default_value, snap, .. } => default_value as f32 / snap as f32,
				SliderType::Continuous { default_value, .. } => default_value,
			},
			slider_type,
			has_camera,
		}
	}
	
	/// Renvoie la valeur du slider (u32 si le slider est discret, f32 s'il est continu).
	pub fn get_value(&self) -> f32 {
		match &self.slider_type {
			SliderType::Discrete { snap, .. } => (self.value * *snap as f32).round(),
			SliderType::Continuous { .. } => self.value,
		}
	}
	
	pub fn reset_value(&mut self) {
		self.value = match &self.slider_type {
			SliderType::Discrete { snap, default_value, .. } => *default_value as f32 / *snap as f32,
			SliderType::Continuous { default_value, .. } => *default_value,
		};
	}
	
	fn thumb_position(&self) -> f64 { self.value as f64 * self.length() }
	
	fn length(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.rect.width() - self.rect.height(),
			Orientation::Vertical => self.rect.height() - self.rect.width(),
		}
	}
	
	fn thickness(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.rect.height(),
			Orientation::Vertical => self.rect.width(),
		}
	}
	
	fn scroll_with_keys(&mut self, delta_sec: f64, plus_key: KeyState, minus_key: KeyState) -> bool {
		let mut changed = false;
		if plus_key.is_pressed() && self.value != 1. {
			match self.slider_type {
				SliderType::Discrete { snap, .. } => {
					self.value += 1. / snap as f32
				},
				SliderType::Continuous { .. } => {
					self.value += delta_sec as f32 * Self::SPEED;
				}
			}
			self.value = self.value.min(1.);
			self.state.press();
			changed = true;
		} else if plus_key.is_released() {
			self.state.release();
			changed = true;
		}
		if minus_key.is_pressed() && self.value != 0. {
			match self.slider_type {
				SliderType::Discrete { snap, .. } => {
					self.value -= 1. / snap as f32
				},
				SliderType::Continuous { .. } => {
					self.value -= delta_sec as f32 * Self::SPEED;
				}
			}
			self.value = self.value.max(0.);
			self.state.press();
			changed = true;
		} else if minus_key.is_released() {
			self.state.release();
			changed = true;
		}
		changed
	}
}

impl Widget for Slider {
	fn update(&mut self, input: &Input, delta_sec: f64, _text_drawer: &TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();
		
		if input.mouse.left_button.is_pressed() {
			self.state.press();
			changed = true;
		} else if input.mouse.left_button.is_released() {
			self.state.release();
			changed = true;
		}
		
		match self.orientation {
			Orientation::Horizontal => {
				changed |= self.scroll_with_keys(delta_sec, input.keys_state.right, input.keys_state.left);
			},
			Orientation::Vertical => {
				changed |= self.scroll_with_keys(delta_sec, input.keys_state.up, input.keys_state.down);
			}
		}
		
		if input.mouse.left_button.is_pressed() || input.mouse.left_button.is_down() {
			let value = {
				let mouse_position = if self.has_camera {
					camera.transform.inverse() * input.mouse.position.cast()
				} else { input.mouse.position.cast() };
				
				let thumb_position = match self.orientation {
					Orientation::Horizontal => mouse_position.x - self.rect.left(),
					Orientation::Vertical => self.rect.bottom() + self.rect.height() - mouse_position.y,
				} - self.thickness() * 0.5;
				thumb_position.clamp(0.0, self.length()) as f32 / self.length() as f32
			};
			
			let value = match self.slider_type {
				SliderType::Discrete { snap, .. } => (value * snap as f32).round() / snap as f32,
				SliderType::Continuous { .. } => value,
			};
			
			if value != self.value {
				self.value = value;
				changed = true;
			}
		}
		
		changed
	}
	
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, focused: bool, hovered: bool) {
		let thumb_radius = self.thickness() * 0.5;
		let bar_radius = thumb_radius * 0.6;
		let margin = thumb_radius - bar_radius;
		let thumb_position = match self.orientation {
			Orientation::Horizontal => self.thumb_position() + thumb_radius,
			Orientation::Vertical => self.rect.height() - self.thumb_position() - thumb_radius,
		};
		let thumb_color = if self.state.is_pressed() || self.state.is_down() { self.style.thumb_pushed_color }
		else if hovered { self.style.thumb_hovered_color } else { self.style.thumb_color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };
		let camera = if self.has_camera { Some(camera) } else { None };
		
		let faces_nb = 7;
		let (mut empty_track, mut filled_track, track_center) = match self.orientation {
			Orientation::Horizontal => {
				let empty_track: Vec<Point2<f64>> = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 - 0.5);
					self.rect.mid_right() - Vector2::new(thumb_radius, 0.) + Vector2::new_polar(bar_radius, angle)
				}).collect();
				let filled_track: Vec<Point2<f64>> = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 + 0.5);
					self.rect.mid_left() + Vector2::new(thumb_radius, 0.) + Vector2::new_polar(bar_radius, angle)
				}).collect();
				
				let thumb_top = self.rect.top_left() + Vector2::new(thumb_position, -margin);
				let thumb_bottom = self.rect.bottom_left() + Vector2::new(thumb_position, margin);
				(empty_track, filled_track, vec![thumb_top, thumb_bottom])
			},
			Orientation::Vertical => {
				let empty_track: Vec<Point2<f64>> = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 + 1.0);
					self.rect.mid_bottom() + Vector2::new(0., thumb_radius) + Vector2::new_polar(bar_radius, angle)
				}).collect();
				let filled_track: Vec<Point2<f64>> = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64);
					self.rect.mid_top() - Vector2::new(0., thumb_radius) + Vector2::new_polar(bar_radius, angle)
				}).collect();
				
				let thumb_right = self.rect.bottom_right() + Vector2::new(-margin, thumb_position);
				let thumb_left = self.rect.bottom_left() + Vector2::new(margin, thumb_position);
				(empty_track, filled_track, vec![thumb_right, thumb_left])
			}
		};
		let mut full_track = empty_track.clone();
		full_track.extend(filled_track.clone());
		empty_track.extend(track_center.clone());
		filled_track.extend(track_center.iter().rev());
		
		fill_polygon(canvas, camera, self.style.empty_track_color, &empty_track);
		fill_polygon(canvas, camera, self.style.filled_track_color, &filled_track);
		draw_polygon(canvas, camera, self.style.border_color, &full_track);
		
		// Thumb
		let thumb_position = self.rect.bottom_left() + match self.orientation {
			Orientation::Horizontal => Vector2::new(thumb_position, thumb_radius),
			Orientation::Vertical => Vector2::new(thumb_radius, thumb_position),
		};
		
		if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_circle(canvas, camera, with_alpha(border_color, FOCUS_HALO_ALPHA), thumb_position, FOCUS_HALO_DELTA + thumb_radius);
		}
		fill_circle(canvas, camera, thumb_color, thumb_position, thumb_radius);
		draw_circle(canvas, camera, border_color, thumb_position, thumb_radius);
		
		let text_position = match self.orientation {
			Orientation::Horizontal => self.rect.position + Vector2::new(self.rect.width() * 0.5, self.rect.height() * 1.5),
			Orientation::Vertical => self.rect.position + Vector2::new(self.rect.width() * 0.5, self.rect.height() + self.rect.width() * 0.5)
		};
		match &self.slider_type {
			SliderType::Discrete { snap, display, .. } => {
				if let Some(format) = display {
					let text: String = format((self.value * *snap as f32).round() as u32);
					draw_text(canvas, camera, text_drawer, text_position, &text,
					          20.0, &TextStyle::default(), Align::Center);
				}
			}
			SliderType::Continuous { display, .. } => {
				if let Some(format) = display {
					let text = format(self.value);
					draw_text(canvas, camera, text_drawer, text_position, &text,
					          20.0, &TextStyle::default(), Align::Center);
				}
			}
		}
	}
	
	fn get_rect(&self) -> Rect { self.rect }
	fn get_rect_mut(&mut self) -> &mut Rect { &mut self.rect }
	
	fn has_camera(&self) -> bool { self.has_camera }
}
