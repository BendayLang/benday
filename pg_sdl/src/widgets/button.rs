use std::time::Duration;

use crate::camera::Camera;
use crate::color::with_alpha;
use crate::custom_rect::Rect;
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect};
use crate::style::Align;
use crate::widgets::{Base, Manager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA};
use crate::{
	color::{darker, Colors},
	input::Input,
	text::TextDrawer,
	text::TextStyle,
	widgets::Widget,
	widgets::{HOVER, PUSH},
};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

#[derive(Clone)]
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
			text_style: TextStyle::new(None, Colors::DARK_GREY, FontStyle::NORMAL),
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
			text_style: TextStyle::new(None, darker(color, 0.5), FontStyle::NORMAL),
		}
	}
}

/// A button is a widget that it can be clicked.
pub struct Button {
	base: Base,
	style: ButtonStyle,
	text: String,
}

impl Button {
	pub fn new(rect: Rect, style: ButtonStyle, text: String) -> Self {
		Self { base: Base::new(rect), style, text }
	}

	pub fn get_text(&self) -> &String {
		&self.text
	}
	pub fn set_text(&mut self, new_text: String) {
		self.text = new_text;
	}

	pub fn is_pressed(&self) -> bool {
		self.base.state.is_pressed()
	}
}

impl Widget for Button {
	fn update(&mut self, input: &Input, _delta: Duration, _: &mut Manager, _: &TextDrawer, _camera: Option<&Camera>) -> bool {
		self.base.update(input, vec![input.keys_state.enter])
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, camera: Option<&Camera>) {
		let color = if self.base.is_pushed() {
			self.style.pushed_color
		} else if self.is_hovered() {
			self.style.hovered_color
		} else {
			self.style.color
		};
		let border_color = if self.is_focused() { self.style.focused_color } else { self.style.border_color };

		if let Some(corner_radius) = self.style.corner_radius {
			if self.is_focused() {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA + corner_radius,
				);
			}
			fill_rounded_rect(canvas, camera, color, self.base.rect, corner_radius);
			draw_rounded_rect(canvas, camera, border_color, self.base.rect, corner_radius);
		} else {
			if self.is_focused() {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA,
				);
			}
			fill_rect(canvas, camera, color, self.base.rect);
			draw_rect(canvas, camera, border_color, self.base.rect);
		}

		draw_text(
			canvas,
			camera,
			text_drawer,
			self.base.rect.center(),
			&self.text,
			self.style.font_size,
			&self.style.text_style,
			Align::Center,
		);
	}

	fn get_base(&self) -> &Base {
		&self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
