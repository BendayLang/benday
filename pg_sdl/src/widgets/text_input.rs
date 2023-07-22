use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::{Input, Shortcut};
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect, get_text_size};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::{Base, Widget, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub struct TextInputStyle {
	background_color: Color,
	background_hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	carrot_color: Color,
	selection_color: Color,
	corner_radius: Option<f64>,
	font_size: f64,
	placeholder_style: TextStyle,
	text_style: TextStyle,
}

impl TextInputStyle {
	pub fn new(color: Color, corner_radius: Option<f64>, font_size: f64) -> Self {
		Self {
			background_color: color,
			background_hovered_color: darker(color, HOVER),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			carrot_color: Colors::DARK_GREY,
			selection_color: with_alpha(Colors::LIGHT_BLUE, 127),
			corner_radius,
			font_size,
			placeholder_style: TextStyle { color: Colors::GREY, ..Default::default() },
			text_style: TextStyle::default(),
		}
	}
}

impl Default for TextInputStyle {
	fn default() -> Self {
		Self {
			background_color: Colors::WHITE,
			background_hovered_color: darker(Colors::WHITE, HOVER),
			focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
			carrot_color: Colors::DARK_GREY,
			selection_color: with_alpha(Colors::LIGHT_BLUE, 127),
			corner_radius: Some(4.0),
			font_size: 15.,
			placeholder_style: TextStyle { color: Colors::GREY, ..Default::default() },
			text_style: TextStyle::default(),
		}
	}
}

pub struct TextInput {
	base: Base,
	style: TextInputStyle,
	placeholder: String,
	text: String,
	carrot_timer_sec: Duration,
	carrot_position: usize,
	/// The selected text (from, to)
	selection: (usize, usize),
	last_click: Instant,
}

impl TextInput {
	const LEFT_SHIFT: f64 = 5.0;
	const BLINKING_TIME_SEC: Duration = Duration::from_millis(400);

	pub fn new(rect: Rect, style: TextInputStyle, placeholder: String) -> Self {
		Self {
			base: Base::new(rect),
			style,
			placeholder,
			text: String::new(),
			carrot_timer_sec: Duration::ZERO,
			carrot_position: 0,
			selection: (0, 0),
			last_click: Instant::now(),
		}
	}

	pub fn get_text(&self) -> &str {
		&self.text
	}

	fn get_carrot_position(&self, text_drawer: &TextDrawer, mouse_position: Point2<i32>, camera: Option<&Camera>) -> usize {
		let mouse_x = ((if let Some(camera) = camera {
			camera.transform().inverse() * mouse_position.cast()
		} else {
			mouse_position.cast()
		})
		.x - self.base.rect.left()) as u32;
		let mut x: u32 = 0;
		for (i, c) in self.text.chars().enumerate() {
			let text_width =
				get_text_size(camera, text_drawer, &c.to_string(), self.style.font_size, &self.style.text_style).x as u32;
			x += text_width;
			if x >= mouse_x {
				return i;
			}
		}
		self.text.len()
	}

	fn is_carrot_visible(&self) -> bool {
		self.carrot_timer_sec < Self::BLINKING_TIME_SEC
	}
}

impl Widget for TextInput {
	#[allow(clippy::diverging_sub_expression)]
	fn update(
		&mut self, input: &Input, delta: Duration, _widgets_manager: &mut WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;
		changed |= self.base.update(input, Vec::new());

		// Carrot blinking
		self.carrot_timer_sec += delta;
		if Self::BLINKING_TIME_SEC < self.carrot_timer_sec && self.carrot_timer_sec < Self::BLINKING_TIME_SEC + delta {
			changed = true;
		}
		if self.carrot_timer_sec > Self::BLINKING_TIME_SEC.mul_f32(2.0) {
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}

		// Carrot movement
		let carrot_position_from_mouse = Some(self.get_carrot_position(text_drawer, input.mouse.position, camera));
		let mut new_carrot_position = None;
		if input.keys_state.left.is_pressed() {
			if self.carrot_position > 0 {
				if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					new_carrot_position = Some(self.carrot_position - 1);
				} else {
					self.carrot_position -= 1;
					self.selection = (self.carrot_position, self.carrot_position);
				}
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}
		if input.keys_state.right.is_pressed() {
			if self.carrot_position < self.text.len() {
				if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					new_carrot_position = Some(self.carrot_position + 1);
				} else {
					self.carrot_position += 1;
					self.selection = (self.carrot_position, self.carrot_position);
				}
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}

		if input.mouse.left_button.is_down() {
			new_carrot_position = carrot_position_from_mouse;
		}

		// if input.mouse.left_button.is_triple_pressed() {
		// 	self.selection = (0, self.text.len());
		// 	self.carrot_position = self.text.len();
		// 	changed = true;
		// } else if input.mouse.left_button.is_double_pressed() {
		// 	self.selection = todo!("selecting a single word");
		// 	self.carrot_position = todo!();
		// 	changed = true;

		// Mouse click
		if input.mouse.left_button.is_pressed() {
			if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
				new_carrot_position = carrot_position_from_mouse;
			} else {
				self.carrot_position = self.get_carrot_position(text_drawer, input.mouse.position, camera);
				self.selection = (self.carrot_position, self.carrot_position);
			}
			self.carrot_timer_sec = Duration::ZERO;
		}

		// Selection
		if let Some(new_carrot_position) = new_carrot_position {
			let (start, end) = self.selection;
			if new_carrot_position != self.carrot_position {
				self.selection = if new_carrot_position > self.carrot_position {
					if new_carrot_position > end {
						(start, new_carrot_position)
					} else {
						(new_carrot_position, end)
					}
				} else if new_carrot_position >= start {
					(start, new_carrot_position)
				} else {
					(new_carrot_position, end)
				};
				self.carrot_position = new_carrot_position;
				self.carrot_timer_sec = Duration::ZERO;
				changed = true;
			}
		}

		// Keyboard input
		// Clipboard
		if input.shortcut_pressed(&Shortcut::PASTE()) && input.clipboard.has_clipboard_text() {
			let (start, end) = self.selection;
			if start != end {
				self.text.drain(start..end);
				self.carrot_position = start;
			}
			let clipboard_text = input.clipboard.clipboard_text().unwrap();
			self.text.insert_str(self.carrot_position, &clipboard_text);
			self.carrot_position += clipboard_text.len();
			self.selection = (self.carrot_position, self.carrot_position);
			return true;
		}
		if input.shortcut_pressed(&Shortcut::COPY()) {
			let (start, end) = self.selection;
			if start != end {
				let text = self.text[start..end].to_string();
				input.clipboard.set_clipboard_text(&text).unwrap();
				return true;
			}
			input.clipboard.set_clipboard_text(&self.text).unwrap();
			return true;
		}
		if input.shortcut_pressed(&Shortcut::CUT()) {
			let (start, end) = self.selection;
			if start != end {
				let text = self.text.drain(start..end).collect::<String>();
				input.clipboard.set_clipboard_text(&text).unwrap();
				self.carrot_position = start;
				self.selection = (self.carrot_position, self.carrot_position);
				return true;
			}
			input.clipboard.set_clipboard_text(&self.text).unwrap();
			self.text.clear();
			self.carrot_position = 0;
			return true;
		}

		// Text input
		if let Some(c) = input.last_char {
			let (start, end) = self.selection;
			if start != end {
				self.text.drain(start..end);
				self.carrot_position = start;
			}

			self.text.insert(self.carrot_position, c);
			self.carrot_position += 1;
			self.selection = (self.carrot_position, self.carrot_position);
			changed = true;
		}
		if input.keys_state.backspace.is_pressed() {
			let (start, end) = self.selection;
			if start != end {
				self.text.drain(start..end);
				self.carrot_position = start;
			} else if self.carrot_position > 0 {
				if input.keys_state.lctrl.is_down() || input.keys_state.lctrl.is_down() {
					todo!(" (TODO implement ctrl + backspace) ")
				} else {
					self.text.remove(self.carrot_position - 1);
					self.carrot_position -= 1;
				}
			}
			self.selection = (self.carrot_position, self.carrot_position);
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		} else if input.keys_state.delete.is_pressed() {
			let (start, end) = self.selection;
			if start != end {
				self.text.drain(start..end);
				self.carrot_position = start;
				self.selection = (self.carrot_position, self.carrot_position);
			} else if self.carrot_position < self.text.len() {
				self.text.remove(self.carrot_position);
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}

		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		// Box
		let background_color = if hovered { self.style.background_hovered_color } else { self.style.background_color };
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

		if focused {
			// Selection
			let (start, end) = self.selection;
			if start != end {
				let selection_height = self.style.font_size * 1.3;
				let selection_x =
					get_text_size(camera, text_drawer, &self.text[..start], self.style.font_size, &self.style.text_style).x;
				let selection_width =
					get_text_size(camera, text_drawer, &self.text[start..end], self.style.font_size, &self.style.text_style).x;
				let rect = Rect::from(
					self.base.rect.mid_left() + Vector2::new(Self::LEFT_SHIFT + selection_x, -selection_height * 0.5),
					Vector2::new(selection_width, selection_height),
				);
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rect(canvas, camera, self.style.selection_color, rect);
			}
			// Carrot
			if self.is_carrot_visible() {
				let carrot_x = if self.carrot_position == 0 {
					0.
				} else {
					get_text_size(
						camera,
						text_drawer,
						&self.text[0..self.carrot_position],
						self.style.font_size,
						&self.style.text_style,
					)
					.x
				};
				let carrot_height = self.style.font_size * 1.2;
				let rect = Rect::from(
					self.base.rect.mid_left() + Vector2::new(Self::LEFT_SHIFT + carrot_x - 0.5, -carrot_height * 0.5),
					Vector2::new(1.5, carrot_height),
				);
				fill_rect(canvas, camera, self.style.carrot_color, rect);
			}
		}

		// Text
		let position = self.base.rect.mid_left() + Vector2::new(Self::LEFT_SHIFT, 0.);
		if self.text.is_empty() {
			draw_text(
				canvas,
				camera,
				text_drawer,
				position,
				&self.placeholder,
				self.style.font_size,
				&self.style.placeholder_style,
				Align::Left,
			);
		} else {
			draw_text(
				canvas,
				camera,
				text_drawer,
				position,
				&self.text,
				self.style.font_size,
				&self.style.text_style,
				Align::Left,
			);
		}
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
