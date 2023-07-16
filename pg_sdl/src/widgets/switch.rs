use std::f64::consts::PI;
use crate::input::{Input, KeyState};
use crate::camera::Camera;
use crate::color::{darker, Colors};
use crate::primitives::{draw_circle, draw_polygon, draw_rounded_rect, fill_circle, fill_polygon, fill_rounded_rect};
use crate::text::TextDrawer;
use crate::widgets::{Orientation, Widget, HOVER, PUSH};
use crate::custom_rect::Rect;

use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;
use crate::vector2::Vector2Plus;

pub struct SwitchStyle {
	on_color: Color,
	off_color: Color,
	thumb_color: Color,
	thumb_hovered_color: Color,
	thumb_pushed_color: Color,
	thumb_focused_color: Color,
	border_color: Color,
}

impl Default for SwitchStyle {
	fn default() -> Self {
		Self {
			on_color: Colors::LIGHT_GREEN,
			off_color: Colors::LIGHT_GREY,
			thumb_color: Colors::LIGHTER_GREY,
			thumb_hovered_color: darker(Colors::LIGHTER_GREY, HOVER),
			thumb_pushed_color: darker(Colors::LIGHTER_GREY, PUSH),
			thumb_focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}

impl SwitchStyle {
	pub fn new(on_color: Color, off_color: Color) -> Self {
		Self {
			on_color,
			off_color,
			thumb_color: Colors::LIGHTER_GREY,
			thumb_hovered_color: darker(Colors::LIGHTER_GREY, HOVER),
			thumb_pushed_color: darker(Colors::LIGHTER_GREY, PUSH),
			thumb_focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}


/// A switch is a widget that can be toggled __on__ or __off__
pub struct Switch {
	rect: Rect,
	state: KeyState,
	has_camera: bool,
	style: SwitchStyle,
	orientation: Orientation,
	switched: bool,
}

impl Switch {
	pub fn new(rect: Rect, style: SwitchStyle, has_camera: bool) -> Self {
		let orientation = { if rect.width() > rect.height() { Orientation::Horizontal } else { Orientation::Vertical } };
		Self {
			rect,
			style,
			orientation,
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
			Orientation::Horizontal => self.rect.width() - self.rect.height(),
			Orientation::Vertical => self.rect.height() - self.rect.width(),
		}
	}
}

impl Widget for Switch {
	fn update(&mut self, input: &Input, _delta: f64, _text_drawer: &TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed() || input.keys_state.enter.is_pressed() {
			self.state.press();
			self.switched = !self.switched;
			changed = true;
		} else if input.mouse.left_button.is_released() || input.keys_state.enter.is_released() {
			self.state.release();
			changed = true;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, focused: bool, hovered: bool) {
		let camera = if self.has_camera { Some(camera) } else { None };

		let color = if self.switched { self.style.on_color } else { self.style.off_color };
		let border_color = if focused { self.style.thumb_focused_color } else { self.style.border_color };
		let thumb_color = if self.state.is_pressed() || self.state.is_down() { self.style.thumb_pushed_color }
		else if hovered { self.style.thumb_hovered_color } else { self.style.thumb_color };
		
		let thickness = match self.orientation {
			Orientation::Horizontal => self.rect.height(),
			Orientation::Vertical => self.rect.width(),
		};
		let radius = thickness * 0.5;
		
		let faces_nb = 9;
		let vertices = match self.orientation {
			Orientation::Horizontal => {
				let mut vertices = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 - 0.5);
					self.rect.mid_right() - Vector2::new(radius, 0.) + Vector2::new_polar(radius, angle)
				}).collect::<Vec<Point2<f64>>>();
				vertices.extend((0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 + 0.5);
					self.rect.mid_left() + Vector2::new(radius, 0.) + Vector2::new_polar(radius, angle)
				}).collect::<Vec<Point2<f64>>>());
				vertices
			},
			Orientation::Vertical => {
				let mut vertices = (0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64 + 1.0);
					self.rect.mid_bottom() + Vector2::new(0., radius) + Vector2::new_polar(radius, angle)
				}).collect::<Vec<Point2<f64>>>();
				vertices.extend((0..=faces_nb).map(|i| {
					let angle = PI * (i as f64 / faces_nb as f64);
					self.rect.mid_top() - Vector2::new(0., radius) + Vector2::new_polar(radius, angle)
				}).collect::<Vec<Point2<f64>>>());
				vertices
			}
		};
		
		fill_polygon(canvas, camera, color, &vertices);
		draw_polygon(canvas, camera, self.style.border_color, &vertices);

		let b = 0.8;

		// Thumb
		let dot_position = match self.orientation {
			Orientation::Horizontal => self.rect.mid_left() + Vector2::new(radius + self.thumb_position(), 0.),
			Orientation::Vertical => self.rect.mid_top() - Vector2::new(0., radius + self.thumb_position()),
		};
		
		fill_circle(canvas, camera, thumb_color, dot_position, b * radius);
		draw_circle(canvas, camera, border_color, dot_position, b * radius);
	}

	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera { camera.transform.inverse() * point } else { point };
		self.rect.collide_point(point)
	}
}
