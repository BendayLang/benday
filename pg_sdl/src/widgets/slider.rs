use nalgebra::{Point2, Vector2};
use crate::input::{Input, KeyState};
use crate::widgets::{HOVER, PUSH, Widget};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;

use crate::primitives::{draw_rounded_rect, fill_rounded_rect};
use sdl2::video::Window;
use crate::camera::Camera;
use crate::color::{Colors, darker, paler};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};

pub enum Orientation {
	Horizontal,
	Vertical,
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
	position: Point2<f64>,
	size: Vector2<f64>,
	color: Color,
	hovered_color: Color,
	back_color: Color,
	hovered_back_color: Color,
	thumb_color: Color,
	hovered_thumb_color: Color,
	pushed_thumb_color: Color,
	orientation: Orientation,
	corner_radius: u16,
	hovered: bool,
	pub state: KeyState,
	/// Internal value of the slider (0.0 - 1.0)
	value: f32,
	slider_type: SliderType,
	has_camera: bool
}

impl Slider {
	pub fn new(position: Point2<f64>, size: Vector2<f64>, color: Color, corner_radius: u16, slider_type: SliderType, has_camera: bool) -> Self {
		let orientation = {
			if size.x > size.y {
				Orientation::Horizontal
			} else {
				Orientation::Vertical
			}
		};
		let thumb_color = Colors::LIGHT_GREY;
		let back_color = darker(paler(color, 0.5), 0.9);
		Self {
			position,
			size,
			color,
			hovered_color: darker(color, HOVER),
			back_color,
			hovered_back_color: darker(back_color, HOVER),
			thumb_color,
			hovered_thumb_color: darker(thumb_color, HOVER),
			pushed_thumb_color: darker(thumb_color, PUSH),
			orientation,
			corner_radius,
			hovered: false,
			state: KeyState::new(),
			value: match slider_type {
				SliderType::Discrete { default_value, snap, .. } => default_value as f32 / snap as f32,
				SliderType::Continuous { default_value, .. } => default_value,
			},
			slider_type,
			has_camera,
		}
	}

	/// Renvoie la valeur du slider comme un u32 si le slider est discret, sinon comme un f32
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

	pub fn set_value(&mut self, value: f32) {
		self.value = value;
	}

	fn thumb_position(&self) -> f64 {
		self.value as f64 * self.length()
	}

	fn length(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.size.x - self.size.y,
			Orientation::Vertical => self.size.y - self.size.x,
		}
	}

	fn thickness(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.size.y,
			Orientation::Vertical => self.size.x,
		}
	}
}

impl Widget for Slider {
	fn update(&mut self, input: &Input, _delta: f64, _text_drawer: &TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed() && self.hovered {
			self.state.press();
			changed = true;
		} else if self.state.is_down() && input.mouse.left_button.is_released() {
			self.state.release();
			changed = true;
		}

		if self.state.is_pressed() | self.state.is_down() {
			let value = {
				let point = input.mouse.position.cast::<f64>();
				let thumb_position = match self.orientation {
					Orientation::Horizontal => point.x - self.position.x,
					Orientation::Vertical => self.position.y - point.y,
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

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool) {
		let b: f32 = 0.7;

		// Back bar
		let margin = (self.thickness() as f32 * (1.0 - b)* 0.5) as u32;
		/* TODO
		let (back_rect, rect) = match self.orientation {
			Orientation::Horizontal => (
				Rect::new(
					self.position.x + (self.thumb_position() + self.thickness() * 0.5),
					self.position.y + self.size.y + margin as i32,
					self.size.x - self.thumb_position() as u32 - self.thickness() * 0.5 - margin,
					self.size.y as f32 * b
				),
				Rect::new(
					self.position.x + margin as i32,
					self.position.y + self.size.y + margin as i32,
					self.thumb_position() + self.thickness()* 0.5 - margin,
					self.size.y as f32 * b
				),
			),
			Orientation::Vertical => (
				Rect::new(
					self.position.x + margin as i32,
					self.position.y + self.size.y + margin as i32,
					self.size.x as f32 * b as u32,
					self.size.y - self.thumb_position() - self.thickness()* 0.5 - margin
				),
				Rect::new(
					self.position.x + margin as i32,
					self.position.y - (self.thumb_position() + self.thickness()* 0.5) as i32,
					self.size.x as f32 * b as u32,
					self.thumb_position() + self.thickness()* 0.5 - margin
				),
			),
		};

		let color =
			if self.hovered | self.state.is_pressed() | self.state.is_down() { self.hovered_back_color } else { self.back_color };
		fill_rounded_rect(canvas, back_rect, color, self.corner_radius);
		draw_rounded_rect(canvas, back_rect, Colors::BLACK, self.corner_radius);
		let color = if self.hovered | self.state.is_pressed() | self.state.is_down() { self.hovered_color } else { self.color };
		fill_rounded_rect(canvas, rect, color, self.corner_radius);
		draw_rounded_rect(canvas, rect, Colors::BLACK, self.corner_radius);

		// Pad
		let rect = match self.orientation {
			Orientation::Horizontal => {
				Rect::new(self.position.x + self.thumb_position() as i32, self.position.y + self.size.y, self.thickness(), self.thickness())
			}
			Orientation::Vertical => Rect::new(
				self.position.x,
				self.position.y - self.thumb_position() as i32 - self.thickness() as i32,
				self.thickness(),
				self.thickness()
			),
		};

		let color = if self.state.is_pressed() | self.state.is_down() {
			self.pushed_thumb_color
		} else if self.hovered {
			self.hovered_thumb_color
		} else {
			self.thumb_color
		};
		fill_rounded_rect(canvas, rect, color, self.corner_radius);
		draw_rounded_rect(canvas, rect, Colors::BLACK, self.corner_radius);

		match &self.slider_type {
			SliderType::Discrete { snap, display, .. } => {
				if let Some(format) = display {
					let text: String = format((self.value * *snap as f32).round() as u32);
					text_drawer.draw(
						canvas,
						rect.center(),
						&TextStyle::new(20, None, Color::BLACK, FontStyle::NORMAL),
						&text,
						Align::Center,
					);
				}
			}
			SliderType::Continuous { display, .. } => {
				if let Some(format) = display {
					let text = format(self.value);
					text_drawer.draw(
						canvas,
						rect.center(),
						&TextStyle::new(20, None, Color::BLACK, FontStyle::NORMAL),
						&text,
						Align::Center,
					);
				}
			}
		}
		 */
	}
	
	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera{ camera.transform * point } else { point };
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}
}
