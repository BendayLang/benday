use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::vector2::Vector2Plus;
use crate::widgets::{Base, Orientation, Widget, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::ttf::FontStyle;
use std::f64::consts::{PI, TAU};

use crate::camera::Camera;
use crate::color::{darker, paler, with_alpha, Colors};
use crate::primitives::{
	draw_circle, draw_polygon, draw_rect, draw_rounded_rect, draw_text, fill_circle, fill_polygon, fill_rect, fill_rounded_rect,
};
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
	base: Base,
	/// Internal value of the slider (0.0 - 1.0)
	value: f32,
	style: SliderStyle,
	orientation: Orientation,
	slider_type: SliderType,
}

impl Slider {
	const SPEED: f32 = 2.5;

	pub fn new(rect: Rect, style: SliderStyle, slider_type: SliderType) -> Self {
		let orientation = if rect.width() > rect.height() { Orientation::Horizontal } else { Orientation::Vertical };
		Self {
			base: Base::new(rect),
			style,
			orientation,
			value: match slider_type {
				SliderType::Discrete { default_value, snap, .. } => default_value as f32 / snap as f32,
				SliderType::Continuous { default_value, .. } => default_value,
			},
			slider_type,
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

	fn thumb_position(&self) -> f64 {
		self.value as f64 * self.length()
	}

	fn length(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.base.rect.width() - self.base.rect.height(),
			Orientation::Vertical => self.base.rect.height() - self.base.rect.width(),
		}
	}

	fn thickness(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.base.rect.height(),
			Orientation::Vertical => self.base.rect.width(),
		}
	}

	fn scroll_with_keys(&mut self, delta_sec: f64, plus_key: KeyState, minus_key: KeyState) {
		if plus_key.is_pressed() && self.value != 1. {
			match self.slider_type {
				SliderType::Discrete { snap, .. } => self.value += 1. / snap as f32,
				SliderType::Continuous { .. } => {
					self.value += delta_sec as f32 * Self::SPEED;
				}
			}
			self.value = self.value.min(1.);
		}
		if minus_key.is_pressed() && self.value != 0. {
			match self.slider_type {
				SliderType::Discrete { snap, .. } => self.value -= 1. / snap as f32,
				SliderType::Continuous { .. } => {
					self.value -= delta_sec as f32 * Self::SPEED;
				}
			}
			self.value = self.value.max(0.);
		}
	}
}

impl Widget for Slider {
	fn update(
		&mut self, input: &Input, delta_sec: f64, _widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;

		match self.orientation {
			Orientation::Horizontal => {
				changed |= self.base.update(input, vec![input.keys_state.right, input.keys_state.left]);
				self.scroll_with_keys(delta_sec, input.keys_state.right, input.keys_state.left);
			}
			Orientation::Vertical => {
				changed |= self.base.update(input, vec![input.keys_state.up, input.keys_state.down]);
				self.scroll_with_keys(delta_sec, input.keys_state.up, input.keys_state.down);
			}
		}

		if input.mouse.left_button.is_pressed() || input.mouse.left_button.is_down() {
			let value = {
				let mouse_position = if let Some(camera) = camera {
					camera.transform().inverse() * input.mouse.position.cast()
				} else {
					input.mouse.position.cast()
				};

				let thumb_position = match self.orientation {
					Orientation::Horizontal => mouse_position.x - self.base.rect.left(),
					Orientation::Vertical => self.base.rect.bottom() + self.base.rect.height() - mouse_position.y,
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

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool) {
		let thumb_radius = self.thickness() * 0.5;
		let bar_radius = thumb_radius * 0.6;
		let margin = thumb_radius - bar_radius;
		let thumb_position = match self.orientation {
			Orientation::Horizontal => self.thumb_position() + thumb_radius,
			Orientation::Vertical => self.base.rect.height() - self.thumb_position() - thumb_radius,
		};
		let thumb_color = if self.base.is_pushed() {
			self.style.thumb_pushed_color
		} else if hovered {
			self.style.thumb_hovered_color
		} else {
			self.style.thumb_color
		};
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };

		let faces_nb = 7;
		let (mut empty_track, mut filled_track, track_center) = match self.orientation {
			Orientation::Horizontal => {
				let empty_track: Vec<Point2<f64>> = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64 - 0.5);
						self.base.rect.mid_right() - Vector2::new(thumb_radius, 0.) + Vector2::new_polar(bar_radius, angle)
					})
					.collect();
				let filled_track: Vec<Point2<f64>> = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64 + 0.5);
						self.base.rect.mid_left() + Vector2::new(thumb_radius, 0.) + Vector2::new_polar(bar_radius, angle)
					})
					.collect();

				let thumb_top = self.base.rect.top_left() + Vector2::new(thumb_position, -margin);
				let thumb_bottom = self.base.rect.bottom_left() + Vector2::new(thumb_position, margin);
				(empty_track, filled_track, vec![thumb_top, thumb_bottom])
			}
			Orientation::Vertical => {
				let empty_track: Vec<Point2<f64>> = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64 + 1.0);
						self.base.rect.mid_bottom() + Vector2::new(0., thumb_radius) + Vector2::new_polar(bar_radius, angle)
					})
					.collect();
				let filled_track: Vec<Point2<f64>> = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64);
						self.base.rect.mid_top() - Vector2::new(0., thumb_radius) + Vector2::new_polar(bar_radius, angle)
					})
					.collect();

				let thumb_right = self.base.rect.bottom_right() + Vector2::new(-margin, thumb_position);
				let thumb_left = self.base.rect.bottom_left() + Vector2::new(margin, thumb_position);
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
		let thumb_position = self.base.rect.bottom_left()
			+ match self.orientation {
				Orientation::Horizontal => Vector2::new(thumb_position, thumb_radius),
				Orientation::Vertical => Vector2::new(thumb_radius, thumb_position),
			};

		if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_circle(
				canvas,
				camera,
				with_alpha(border_color, FOCUS_HALO_ALPHA),
				thumb_position,
				FOCUS_HALO_DELTA + thumb_radius,
			);
		}
		fill_circle(canvas, camera, thumb_color, thumb_position, thumb_radius);
		draw_circle(canvas, camera, border_color, thumb_position, thumb_radius);

		let text_position = match self.orientation {
			Orientation::Horizontal => {
				self.base.rect.position + Vector2::new(self.base.rect.width() * 0.5, self.base.rect.height() * 1.5)
			}
			Orientation::Vertical => {
				self.base.rect.position
					+ Vector2::new(self.base.rect.width() * 0.5, self.base.rect.height() + self.base.rect.width() * 0.5)
			}
		};
		match &self.slider_type {
			SliderType::Discrete { snap, display, .. } => {
				if let Some(format) = display {
					let text: String = format((self.value * *snap as f32).round() as u32);
					draw_text(canvas, camera, text_drawer, text_position, &text, 20.0, &TextStyle::default(), Align::Center);
				}
			}
			SliderType::Continuous { display, .. } => {
				if let Some(format) = display {
					let text = format(self.value);
					draw_text(canvas, camera, text_drawer, text_position, &text, 20.0, &TextStyle::default(), Align::Center);
				}
			}
		}
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
