use super::button::ButtonStyle;
use crate::camera::Camera;
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect};
use crate::style::Align;
use crate::{
	color::{darker, Colors},
	input::{Input, KeyState},
	text::TextDrawer,
	text::TextStyle,
	widgets::Widget,
	widgets::{HOVER, PUSH},
};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// A rect is a stateless widget.
pub struct TextBox {
	position: Point2<f64>,
	size: Vector2<f64>,
	style: ButtonStyle,
	text: Option<String>,
	has_camera: bool,
}

impl TextBox {
	pub fn new(position: Point2<f64>, size: Vector2<f64>, style: ButtonStyle, text: Option<String>, has_camera: bool) -> Self {
		Self { position, size, style, text, has_camera }
	}
}

impl Widget for TextBox {
	fn update(&mut self, _input: &Input, _delta_sec: f64, _text_drawer: &TextDrawer, _camera: &Camera) -> bool {
		false
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool) {
		let color = self.style.color;
		let camera = if self.has_camera { Some(camera) } else { None };

		if let Some(corner_radius) = self.style.corner_radius {
			fill_rounded_rect(canvas, camera, color, self.position, self.size, corner_radius);
			draw_rounded_rect(canvas, camera, Colors::BLACK, self.position, self.size, corner_radius);
		} else {
			fill_rect(canvas, camera, color, self.position, self.size);
			draw_rect(canvas, camera, Colors::BLACK, self.position, self.size);
		};

		if selected {
			let position = Point2::new(self.position.x + 1.0, self.position.y + 1.0);
			let size = Vector2::new(self.size.x - 2.0, self.size.y - 2.0);
			if let Some(corner_radius) = self.style.corner_radius {
				draw_rounded_rect(canvas, camera, self.style.selected_color, position, size, corner_radius - 1.0);
				draw_rounded_rect(canvas, camera, self.style.selected_color, position, size, corner_radius - 2.0);
			} else {
				draw_rect(canvas, camera, self.style.selected_color, position, size);
			}
		}

		if let Some(text) = &self.text {
			draw_text(
				canvas,
				camera,
				text_drawer,
				self.style.text_style.color,
				self.position + self.size * 0.5,
				self.style.text_style.font_size as f64,
				text.clone(),
				Align::Center,
			);
		}
	}

	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera { camera.transform * point } else { point };
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}
}
