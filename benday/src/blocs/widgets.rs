use crate::blocs::containers::Slot;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{paler, Colors};
use pg_sdl::prelude::Align;
use pg_sdl::text::TextDrawer;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct TextBox {
	default_text: String,
	default_color: Color,
	text: String,
	color: Color,
	size: Vector2<f64>,
}

impl TextBox {
	pub fn new(size: Vector2<f64>, color: Color, default_text: String) -> Self {
		Self { default_text, default_color: paler(color, 0.2), text: String::new(), color: paler(color, 0.5), size }
	}
	pub fn get_size(&self) -> Vector2<f64> {
		self.size
	}
	pub fn get_text(&self) -> String {
		self.text.clone()
	}
	pub fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, position: Point2<f64>,
		selected: bool,
	) {
		let color = if selected { paler(self.color, 0.2) } else { self.color };
		camera.fill_rounded_rect(canvas, color, position, self.size, Slot::RADIUS);
		let position = position + Vector2::new(5.0, self.size.y * 0.5);
		if self.text.is_empty() {
			camera.draw_text(canvas, text_drawer, Colors::GREY, position, 12.0, self.default_text.clone(), Align::Left);
		} else {
			camera.draw_text(canvas, text_drawer, Colors::BLACK, position, 12.0, self.text.clone(), Align::Left);
		}
	}
}
