use std::time::Duration;
use as_any::Downcast;

use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::{Input, Shortcut};
use crate::primitives::{draw_hline, draw_rect, draw_rounded_rect, draw_text, draw_vline, fill_rect, fill_rounded_rect, get_text_size};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::{WidgetsManager, Base, Widget, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH};
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
	selected_option_color: Color,
	slider_color: Color,
	slider_hovered_color: Color,
	slider_pushed_color: Color,
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
			selected_option_color: darker(color, PUSH),
			slider_color: darker(color, 0.85),
			slider_hovered_color: darker(color, 0.85 * HOVER),
			slider_pushed_color: darker(color, 0.85 * PUSH),
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
			selected_option_color: darker(Colors::WHITE, PUSH),
			slider_color: Colors::LIGHT_GREY,
			slider_hovered_color: darker(Colors::LIGHT_GREY, HOVER),
			slider_pushed_color: darker(Colors::LIGHT_GREY, PUSH),
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
	Options { option_index: usize },
	Slider,
}

pub struct Select {
	base: Base,
	max_height: f64,
	style: SelectStyle,
	options: Vec<String>,
	selected_option: Option<usize>,
	text_input: TextInput,
	slider_value: f64,
	hovered_element: SelectElement,
	is_on_slider: Option<f64>,
	is_on_text_input: bool,
}

impl Select {
	const HEIGHT_MARGIN: f64 = 4.;
	const SLIDER_MARGIN: f64 = 2.;

	pub fn new(rect: Rect, style: SelectStyle, options: Vec<String>, placeholder: String) -> Self {
		let text_input_rect = Rect::from(rect.position, Vector2::new(rect.width(), style.font_size + 2. * Self::HEIGHT_MARGIN));
		let text_input_style = TextInputStyle::new(style.color, style.corner_radius, style.font_size, false);
		let mut select = Self {
			base: Base::new(rect),
			max_height: rect.height(),
			style,
			options,
			selected_option: None,
			text_input: TextInput::new(text_input_rect, text_input_style, placeholder),
			slider_value: 0.0,
			hovered_element: SelectElement::TextInput,
			is_on_slider: None,
			is_on_text_input: false,
		};
		select.base.rect.size.y = text_input_rect.height();
		select
	}

	fn option_height(&self) -> f64 {
		self.style.font_size + Self::HEIGHT_MARGIN
	}
	
	fn get_height(&self) -> f64 {
		self.max_height.min(self.option_height() * self.options.len() as f64 + self.text_input.get_base().rect.height())
	}
	
	fn is_max_height(&self) -> bool {
		self.max_height < self.option_height() * self.options.len() as f64 + self.text_input.get_base().rect.height()
	}

	fn get_bottom_rect(&self) -> Rect {
		let text_input_height = self.text_input.get_base().rect.height();
		Rect::from(
			self.base.rect.position + Vector2::new(0., text_input_height),
			self.base.rect.size - Vector2::new(0., text_input_height),
		)
	}
	
	fn get_options_zone_rect(&self) -> Rect {
		let text_input_height = self.text_input.get_base().rect.height();
		Rect::from(
			self.base.rect.position + Vector2::new(0., text_input_height),
			self.base.rect.size - Vector2::new(self.style.slider_width, text_input_height),
		)
	}
	
	fn get_option_rect(&self, option_index: usize) -> Rect {
		let width = self.base.rect.width() - if self.is_max_height() { self.style.slider_width } else { 0. };
		let height = self.option_height();
		let y = self.base.rect.bottom() + self.text_input.get_base().rect.height() +
			height * option_index as f64 - self.slider_value * self.options_course();
		Rect::new(self.base.rect.left(), y, width, height)
	}
	
	fn get_slider_rect(&self) -> Rect {
		let text_input_height = self.text_input.get_base().rect.height();
		Rect::from(
			self.base.rect.position + Vector2::new(self.base.rect.width() - self.style.slider_width, text_input_height),
			Vector2::new(self.style.slider_width, self.base.rect.height() - text_input_height),
		)
	}
	
	fn slider_course(&self) -> f64 {
		self.get_slider_rect().height() - self.slider_height() - 2. * Self::SLIDER_MARGIN
	}
	
	fn slider_height(&self) -> f64 {
		let height = self.get_slider_rect().height();
		height * height / (self.option_height() * self.options.len() as f64)
	}
	
	fn set_slider_value(&mut self, value: f64, mouse_y: f64) {
		self.slider_value = value;
		if self.slider_value < 0. {
			self.slider_value = 0.;
			self.is_on_slider = Some(self.get_slider_rect().y() - mouse_y + self.slider_value * self.slider_course());
		}
		if self.slider_value > 1. {
			self.slider_value = 1.;
			self.is_on_slider = Some(self.get_slider_rect().y() - mouse_y + self.slider_value * self.slider_course());
		}
	}
	
	fn options_course(&self) -> f64 {
		self.option_height() * self.options.len() as f64 - self.get_options_zone_rect().height()
	}
}

impl Widget for Select {
	fn update(
		&mut self, input: &Input, delta: Duration, widgets_manager: &WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;
		changed |= self.base.update(input, vec![input.keys_state.up, input.keys_state.down]);
		
		if input.keys_state.up.is_pressed() {
			if let Some(option_index) = self.selected_option {
				self.selected_option = Some(if option_index == 0 { self.options.len() - 1 } else { option_index - 1 });
			} else {
				self.selected_option = Some(0);
			}
		}
		if input.keys_state.down.is_pressed() {
			if let Some(option_index) = self.selected_option {
				self.selected_option = Some(if option_index == self.options.len() - 1 { 0 } else { option_index + 1 });
			} else {
				self.selected_option = Some(0);
			}
		}
		if input.keys_state.enter.is_pressed() {
			if let Some(option_index) = self.selected_option {
				if self.text_input.get_text() == self.options[option_index] {
					// widgets_manager.unselect_widget();
				} else {
					self.text_input.set_text(self.options[option_index].clone());
				}
			}
		}

		// Update witch element is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() && !input.mouse.left_button.is_down() {
			let mut new_hovered_element = None;
			let mouse_position = if let Some(camera) = camera {
					camera.transform().inverse() * input.mouse.position.cast()
			} else { input.mouse.position.cast() };
			
			if self.get_options_zone_rect().collide_point(mouse_position) {
				let y = mouse_position.y - self.get_options_zone_rect().bottom() + self.slider_value * self.options_course();
				let option_index = (y / self.option_height()).floor() as usize;
				new_hovered_element = Some(SelectElement::Options { option_index });
			} else if self.get_slider_rect().collide_point(mouse_position) {
				new_hovered_element = Some(SelectElement::Slider);
			} else if self.text_input.collide_point(mouse_position) {
				new_hovered_element = Some(SelectElement::TextInput);
			}
			if let Some(new_hovered_element) = new_hovered_element {
				self.hovered_element = new_hovered_element;
				changed = true;
			}
		}
		
		// Mouse click
		if input.mouse.left_button.is_pressed() {
			match self.hovered_element {
				SelectElement::TextInput => {
					self.selected_option = None;
				},
				SelectElement::Options { option_index } => {
					self.selected_option = Some(option_index);
					self.text_input.set_text(self.options[option_index].clone());
					self.is_on_text_input = false;
				},
				SelectElement::Slider => {
					let mouse_y = if let Some(camera) = camera {
							(camera.transform().inverse() * input.mouse.position.cast()).y
					} else { input.mouse.position.cast().y };
					let course = self.slider_value * self.slider_course();
					let value = mouse_y - self.get_slider_rect().y() - self.slider_height() * 0.5;
					if value < course - self.slider_height() * 0.5 || course + self.slider_height() * 0.5 < value {
						self.set_slider_value(value / self.slider_course(), mouse_y);
					}
					self.is_on_slider = Some(self.get_slider_rect().y() - mouse_y + self.slider_value * self.slider_course());
					self.is_on_text_input = false;
				}
			}
		}
		else if input.mouse.left_button.is_released() {
			self.is_on_slider = None;
			self.is_on_text_input = true;
		}
		
		if input.mouse.delta.y != 0 {
			if let Some(grab_delta_y) = self.is_on_slider {
				let mouse_y = if let Some(camera) = camera {
						(camera.transform().inverse() * input.mouse.position.cast()).y
				} else { input.mouse.position.cast().y };
				self.set_slider_value((mouse_y - self.get_slider_rect().y() + grab_delta_y) / self.slider_course(), mouse_y);
				changed = true;
			}
		}
		if self.is_on_text_input {
			changed |= self.text_input.update(input, delta, widgets_manager, text_drawer, camera);
		}

		changed
	}
	
	fn on_select(&mut self) {
		self.base.rect.size.y = self.get_height();
	}
	
	fn on_unselect(&mut self) {
		self.base.rect.size.y = self.text_input.get_base().rect.height();
		self.is_on_text_input = false;
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		if focused {
			// Box
			let bottom_rect = self.get_bottom_rect();
			if let Some(corner_radius) = self.style.corner_radius {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(self.style.focused_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA + corner_radius,
				);
				fill_rounded_rect(canvas, camera, self.style.color, bottom_rect, corner_radius);
			} else {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(self.style.focused_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA,
				);
				fill_rect(canvas, camera, self.style.color, bottom_rect);
			}
			
			// Options
			if hovered {
				if let SelectElement::Options { option_index } = self.hovered_element {
					if let Some(corner_radius) = self.style.corner_radius {
						fill_rounded_rect(canvas, camera, self.style.hovered_color, self.get_option_rect(option_index), corner_radius);
					} else {
						fill_rect(canvas, camera, self.style.hovered_color, self.get_option_rect(option_index));
					}
				}
			}
			if let Some(option_index) = self.selected_option {
				if let Some(corner_radius) = self.style.corner_radius {
					fill_rounded_rect(canvas, camera, self.style.selected_option_color, self.get_option_rect(option_index), corner_radius);
				} else {
					fill_rect(canvas, camera, self.style.selected_option_color, self.get_option_rect(option_index));
				}
			}
			let options_rect = if self.is_max_height() { self.get_options_zone_rect() } else { self.get_bottom_rect() };
			let x1 = options_rect.left();
			let x2 = options_rect.right();
			let bottom = options_rect.bottom() - self.slider_value * self.options_course();
			self.options.iter().enumerate().for_each(|(option_index, option)| {
				let position = Point2::new(x1 + Self::HEIGHT_MARGIN, bottom + self.option_height() * (option_index as f64 + 0.5));
				draw_text(canvas, camera, text_drawer, position, option, self.style.font_size, &TextStyle::default(), Align::Left);
			});
			(1..self.options.len()).for_each(|i| {
				let y = bottom + self.option_height() * i as f64;
				draw_hline(canvas, camera, Colors::GREY, x1, x2, y - 1.0);
				draw_hline(canvas, camera, Colors::GREY, x1, x2, y);
			});
			
			// Slider
			if self.is_max_height() {
				let slider_color = if focused && self.is_on_slider.is_some() {
					self.style.slider_pushed_color
				} else if hovered && self.hovered_element == SelectElement::Slider {
					self.style.slider_hovered_color
				} else { self.style.slider_color };
				let mut slider_rect = self.get_slider_rect().enlarged(-Self::SLIDER_MARGIN);
				slider_rect.size.y = self.slider_height();
				slider_rect.position.y += self.slider_value * self.slider_course();
				if let Some(corner_radius) = self.style.corner_radius {
					fill_rounded_rect(canvas, camera, slider_color, slider_rect, corner_radius);
					draw_rounded_rect(canvas, camera, self.style.border_color, slider_rect, corner_radius);
				} else {
					fill_rect(canvas, camera, slider_color, slider_rect);
					draw_rect(canvas, camera, self.style.border_color, slider_rect);
				}
			}
			
			// Contour
			if let Some(corner_radius) = self.style.corner_radius {
				draw_rounded_rect(canvas, camera, self.style.border_color, bottom_rect, corner_radius);
			} else {
				draw_rect(canvas, camera, self.style.border_color, bottom_rect);
			}
		}
		
		// Text input
		let text_input_hovered = hovered && self.hovered_element == SelectElement::TextInput;
		self.text_input.draw(canvas, text_drawer, camera, self.is_on_text_input, text_input_hovered);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
