use std::time::Duration;

use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::{Input, Shortcut};
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect, get_text_size};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::{Base, Widget, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use nalgebra::{Point2, Vector2};
use sdl2::mouse::SystemCursor::No;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

use super::button::{Button, ButtonStyle};
use super::text_input::{TextInput, TextInputStyle};
use super::WidgetId;

#[derive(Clone)]
pub struct SelectStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: Option<f64>,
	font_size: f64,
	slider_width: f64,
}

impl SelectStyle {
	pub fn new(color: Color, corner_radius: Option<f64>, font_size: f64) -> Self {
		Self {
			color: color,
			hovered_color: darker(color, HOVER),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			corner_radius,
			font_size,
			slider_width: font_size,
		}
	}
}

impl Default for SelectStyle {
	fn default() -> Self {
		Self {
			color: Colors::WHITE,
			hovered_color: darker(Colors::WHITE, HOVER),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			corner_radius: Some(4.0),
			font_size: 15.,
			slider_width: 15.,
		}
	}
}

#[derive(PartialEq)]
enum SelectElement {
	TextInput,
	Options { option: usize },
	Slider,
}

pub struct Select {
	base: Base,
	max_height: f64,
	style: SelectStyle,
	options: Vec<String>,
	selected_option: Option<usize>,
	text_input: TextInput,
	slider_value: f32,
	is_text_input_focused: bool,
	hovered_element: SelectElement,
}

impl Select {
	const HEIGHT_MARGIN: f64 = 4.;

	pub fn new(rect: Rect, style: SelectStyle, options: Vec<String>, placeholder: String) -> Self {
		let text_input_rect = Rect::from(rect.position, Vector2::new(rect.width(), style.font_size + 2. * Self::HEIGHT_MARGIN));
		let text_input_style = TextInputStyle::new(style.color, style.corner_radius, style.font_size);
		let mut select = Self {
			base: Base::new(rect),
			max_height: rect.height(),
			style,
			options,
			selected_option: None,
			text_input: TextInput::new(text_input_rect, text_input_style, placeholder),
			slider_value: 0.0,
			is_text_input_focused: true,
			hovered_element: SelectElement::TextInput,
		};
		select.base.rect.size.y = select.get_height();
		select
	}

	fn get_options_height(&self) -> f64 {
		(self.style.font_size + Self::HEIGHT_MARGIN) * self.options.len() as f64
	}
	fn get_height(&self) -> f64 {
		self.max_height.max(self.get_options_height() + self.text_input.get_base().rect.height())
	}

	fn get_options_rect(&self) -> Rect {
		let text_input_height = self.text_input.get_base().rect.height();
		Rect::from(
			self.base.rect.position + Vector2::new(0., text_input_height),
			self.base.rect.size - Vector2::new(self.style.slider_width, text_input_height),
		)
	}
	fn get_slider_rect(&self) -> Rect {
		let text_input_height = self.text_input.get_base().rect.height();
		Rect::from(
			self.base.rect.position + Vector2::new(self.base.rect.width() - self.style.slider_width, text_input_height),
			Vector2::new(self.style.slider_width, self.base.rect.height() - text_input_height),
		)
	}
}

impl Widget for Select {
	fn update(
		&mut self, input: &Input, delta: Duration, _widgets_manager: &mut WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;
		changed |= self.base.update(input, Vec::new());

		// Update witch element is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() && !input.mouse.left_button.is_down() {
			let mouse_position = input.mouse.position.cast();
			let mouse_position =
				if let Some(camera) = camera { camera.transform().inverse() * mouse_position } else { mouse_position };

			if self.get_options_rect().collide_point(mouse_position) {
				self.hovered_element = SelectElement::Options { option: 0 };
				changed = true;
			} else if self.get_slider_rect().collide_point(mouse_position) {
				self.hovered_element = SelectElement::Slider;
				changed = true;
			} else if self.text_input.get_base().rect.collide_point(mouse_position) {
				self.hovered_element = SelectElement::TextInput;
				changed = true;
			}
		}

		if self.is_text_input_focused {
			changed |= self.text_input.update(input, delta, _widgets_manager, text_drawer, camera);
		}

		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		// Box
		let background_color = if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };
		if let Some(corner_radius) = self.style.corner_radius {
			if focused {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA + corner_radius,
				);
			}
			fill_rounded_rect(canvas, camera, background_color, self.base.rect, corner_radius);
			draw_rounded_rect(canvas, camera, border_color, self.base.rect, corner_radius);
		} else {
			if focused {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA,
				);
			}
			fill_rect(canvas, camera, background_color, self.base.rect);
			draw_rect(canvas, camera, border_color, self.base.rect);
		}

		let text_input_focused = focused && self.is_text_input_focused;
		let text_input_hovered = hovered && self.hovered_element == SelectElement::TextInput;
		self.text_input.draw(canvas, text_drawer, camera, text_input_focused, text_input_hovered);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
