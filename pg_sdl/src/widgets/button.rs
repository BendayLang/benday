use nalgebra::{Point2, Vector2};
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect};
use crate::{
	color::{darker, Colors},
	input::{Input, KeyState},
	text::TextDrawer,
	widgets::Widget,
	widgets::{HOVER, PUSH},
	text::TextStyle,
};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::camera::Camera;
use crate::style::Align;


pub struct ButtonStyle {
	color: Color,
	hovered_color: Color,
	pushed_color: Color,
	contour_color: Color,
	selected_color: Color,
	corner_radius: Option<f64>,
	text_style: TextStyle,
}

impl Default for ButtonStyle {
	fn default() -> Self {
		Self {
			color: Colors::LIGHT_BLUE,
			hovered_color: darker(Colors::LIGHT_BLUE, HOVER),
			pushed_color: darker(Colors::LIGHT_BLUE, PUSH),
			selected_color: Colors::DARK_BLUE,
			contour_color: Colors::BLACK,
			corner_radius: Some(7.0),
			text_style: TextStyle::default(),
		}
	}
}

/// A button is a widget that it can be clicked.
pub struct Button {
	position: Point2<f64>,
	size: Vector2<f64>,
	style: ButtonStyle,
	text: String,
	pub state: KeyState,
	has_camera: bool,
}

impl Button {
	pub fn new(position: Point2<f64>, size: Vector2<f64>, style: ButtonStyle, text: String, has_camera: bool) -> Self {
		Self {
			position,
			size,
			style,
			text,
			state: KeyState::new(),
			has_camera,
		}
	}
	pub fn set_text(&mut self, new_text: String) {
		self.text = new_text;
	}
}

impl Widget for Button {
	fn update(&mut self, input: &Input, _delta_sec: f64, _text_drawer: &TextDrawer, _camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed() {
			self.state.press();
			changed = true;
		} else if self.state.is_down() && input.mouse.left_button.is_released() {
			self.state.release();
			changed = true;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool) {
		let color = if self.state.is_pressed() | self.state.is_down() { self.style.pushed_color
		} else if hovered { self.style.hovered_color
		} else { self.style.color };
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

		draw_text(canvas, camera, text_drawer, self.style.text_style.color, self.position + self.size * 0.5,
		          self.style.text_style.font_size as f64, self.text.clone(), Align::Center);
	}
	
	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera{ camera.transform * point } else { point };
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}
}
