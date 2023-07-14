use crate::input::{Input, KeyState};
use crate::widgets::{Orientation, Widget, HOVER, PUSH};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

use crate::camera::Camera;
use crate::color::{darker, Colors};
use crate::primitives::{draw_rounded_rect, fill_rounded_rect};
use crate::text::TextDrawer;

/// A switch is a widget that can be toggled __on__ or __off__
pub struct Switch {
	position: Point2<f64>,
	size: Vector2<f64>,
	on_color: Color,
	hovered_on_color: Color,
	off_color: Color,
	hovered_off_color: Color,
	thumb_color: Color,
	hovered_thumb_color: Color,
	orientation: Orientation,
	corner_radius: f64,
	hovered: bool,
	pub state: KeyState,
	switched: bool,
	has_camera: bool,
}

impl Switch {
	pub fn new(
		position: Point2<f64>, size: Vector2<f64>, on_color: Color, off_color: Color, corner_radius: f64, has_camera: bool,
	) -> Self {
		let orientation = {
			if size.x > size.y {
				Orientation::Horizontal
			} else {
				Orientation::Vertical
			}
		};
		let thumb_color = Colors::LIGHT_GREY;
		Self {
			position,
			size,
			on_color,
			hovered_on_color: darker(on_color, HOVER),
			off_color,
			hovered_off_color: darker(off_color, HOVER),
			thumb_color,
			hovered_thumb_color: darker(thumb_color, HOVER),
			orientation,
			corner_radius,
			hovered: false,
			state: KeyState::new(),
			switched: false,
			has_camera,
		}
	}

	pub fn set_switched(&mut self, switched: bool) {
		self.switched = switched;
	}

	pub fn is_switched(&self) -> bool {
		self.switched
	}

	fn thumb_position(&self) -> f64 {
		f64::from(self.switched) * self.length()
	}

	fn length(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.size.x - self.size.y,
			Orientation::Vertical => self.size.y - self.size.x,
		}
	}
}

impl Widget for Switch {
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

		if self.state.is_pressed() {
			self.switched = !self.switched;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool) {
		let b = 0.7;

		let color = {
			if self.switched {
				if self.hovered {
					self.hovered_on_color
				} else {
					self.on_color
				}
			} else {
				if self.hovered {
					self.hovered_off_color
				} else {
					self.off_color
				}
			}
		};
		let camera = if self.has_camera { Some(camera) } else { None };

		fill_rounded_rect(canvas, camera, color, self.position, self.size, self.corner_radius);
		draw_rounded_rect(canvas, camera, Colors::BLACK, self.position, self.size, self.corner_radius);

		let thickness = match self.orientation {
			Orientation::Horizontal => self.size.y,
			Orientation::Vertical => self.size.x,
		};
		let margin = thickness * (1.0 - b) / 2.0;
		let dot_width = thickness - 2.0 * margin; // (thickness * b) as u32;

		// Pad
		let (position, size): (Point2<f64>, Vector2<f64>) = match self.orientation {
			Orientation::Horizontal => (
				Point2::new(margin + self.position.x + self.thumb_position(), margin + todo!("top") as f64),
				Vector2::new(dot_width, dot_width),
			),
			Orientation::Vertical => (
				Point2::new(margin + self.position.x, margin + self.position.y - self.thumb_position() - thickness),
				Vector2::new(dot_width, dot_width),
			),
		};

		let radius = self.corner_radius * b;
		let color = if self.hovered { self.hovered_thumb_color } else { self.thumb_color };
		fill_rounded_rect(canvas, camera, color, position, size, radius);
		draw_rounded_rect(canvas, camera, Colors::BLACK, position, size, radius);
	}

	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera { camera.transform * point } else { point };
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}
}
