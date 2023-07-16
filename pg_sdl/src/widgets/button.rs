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
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use crate::camera::Camera;
use crate::color::with_alpha;
use crate::style::Align;
use crate::custom_rect::Rect;
use crate::widgets::{FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA};


pub struct ButtonStyle {
	color: Color,
	hovered_color: Color,
	pushed_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: Option<f64>,
	font_size: f64,
	text_style: TextStyle,
}

impl Default for ButtonStyle {
	fn default() -> Self {
		Self {
			color: Colors::LIGHTER_GREY,
			hovered_color: darker(Colors::LIGHTER_GREY, HOVER),
			pushed_color: darker(Colors::LIGHTER_GREY, PUSH),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			corner_radius: Some(7.0),
			font_size: 16.,
			text_style: TextStyle::default(),
		}
	}
}

impl ButtonStyle {
	pub fn new(color: Color, corner_radius: Option<f64>, font_size: f64) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			pushed_color: darker(color, PUSH),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			corner_radius,
			font_size,
			text_style: TextStyle::default(),
		}
	}
}

/// A button is a widget that it can be clicked.
pub struct Button {
	rect: Rect,
	state: KeyState,
	style: ButtonStyle,
	has_camera: bool,
	text: String,
}

impl Button {
	pub fn new(rect: Rect, text: String, style: ButtonStyle, has_camera: bool) -> Self {
		Self {
			rect,
			state: KeyState::new(),
			style,
			has_camera,
			text,
		}
	}
	pub fn set_text(&mut self, new_text: String) {
		self.text = new_text;
	}
	pub fn is_pressed(&self) -> bool { self.state.is_pressed() }
}

impl Widget for Button {
	fn update(&mut self, input: &Input, _delta_sec: f64, _text_drawer: &TextDrawer, _camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed() || input.keys_state.enter.is_pressed() {
			self.state.press();
			changed = true;
		} else if input.mouse.left_button.is_released() || input.keys_state.enter.is_released() {
			self.state.release();
			changed = true;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, focused: bool, hovered: bool) {
		let camera = if self.has_camera { Some(camera) } else { None };
		
		let color = if self.state.is_pressed() || self.state.is_down() {
			self.style.pushed_color } else if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };
		if let Some(corner_radius) = self.style.corner_radius {
			if focused {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(canvas, camera, with_alpha(border_color, FOCUS_HALO_ALPHA),
				                  self.rect.enlarged(FOCUS_HALO_DELTA), FOCUS_HALO_DELTA + corner_radius);
			}
			fill_rounded_rect(canvas, camera, color, self.rect, corner_radius);
			draw_rounded_rect(canvas, camera, border_color, self.rect, corner_radius);
			
		} else {
			if focused {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(canvas, camera, with_alpha(border_color, FOCUS_HALO_ALPHA),
				                  self.rect.enlarged(FOCUS_HALO_DELTA), FOCUS_HALO_DELTA);
			}
			fill_rect(canvas, camera, color, self.rect);
			draw_rect(canvas, camera, border_color, self.rect);
		};

		draw_text(canvas, camera, text_drawer, self.rect.center(), &self.text,
		          self.style.font_size, &self.style.text_style, Align::Center);
	}
	
	fn get_rect(&self) -> Rect { self.rect }
	fn get_rect_mut(&mut self) -> &mut Rect { &mut self.rect }
	
	fn has_camera(&self) -> bool { self.has_camera }
}
