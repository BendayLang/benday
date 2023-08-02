use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::{Input, Shortcut};
use crate::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rect, fill_rounded_rect, get_text_size};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::{Base, Manager, Widget, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;
use sdl2::video::Window;

pub struct TextInputStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Option<Color>,
	border_color: Color,
	carrot_color: Color,
	selection_color: Color,
	pub font_size: f64,
	placeholder_style: TextStyle,
	text_style: TextStyle,
}

impl TextInputStyle {
	pub fn new(color: Color, font_size: f64, focus: bool) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			focused_color: if focus { Some(Colors::BLUE) } else { None },
			border_color: Colors::BLACK,
			carrot_color: Colors::DARK_GREY,
			selection_color: with_alpha(Colors::LIGHT_BLUE, 127),
			font_size,
			placeholder_style: TextStyle { color: Colors::GREY, ..Default::default() },
			text_style: TextStyle::default(),
		}
	}
}

impl Default for TextInputStyle {
	fn default() -> Self {
		Self {
			color: Colors::WHITE,
			hovered_color: darker(Colors::WHITE, HOVER),
			focused_color: Some(Colors::BLUE),
			border_color: Colors::BLACK,
			carrot_color: Colors::DARK_GREY,
			selection_color: with_alpha(Colors::LIGHT_BLUE, 127),
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
	selection: Option<(usize, usize)>,
	last_click: Instant,
	click_count: u8,
	click_position: usize,
}

impl TextInput {
	const LEFT_SHIFT: f64 = 5.0;
	const BLINKING_TIME_SEC: Duration = Duration::from_millis(400);
	const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(300);

	pub fn new(rect: Rect, corner_radius: Option<f64>, style: TextInputStyle, placeholder: String) -> Self {
		Self {
			base: Base::new(rect, corner_radius, false),
			style,
			placeholder,
			text: String::new(),
			carrot_timer_sec: Duration::ZERO,
			carrot_position: 0,
			selection: None,
			last_click: Instant::now(),
			click_count: 0,
			click_position: 0,
		}
	}
	
	pub fn get_style(&self) -> &TextInputStyle {
		&self.style
	}

	pub fn get_text(&self) -> &str {
		&self.text
	}

	pub fn set_text(&mut self, text: String) {
		self.text = text;
		self.selection = None;
		self.carrot_position = self.text.len();
	}

	fn get_carrot_position(&self, text_drawer: &mut TextDrawer, mouse_position: Point2<i32>, camera: Option<&Camera>) -> usize {
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

#[derive(PartialEq, Eq)]
enum CharType {
	Space,
	AlphaNum,
	Other,
}

impl CharType {
	pub fn from_char(c: char) -> Self {
		if c.is_alphanumeric() {
			CharType::AlphaNum
		} else if c.is_whitespace() {
			CharType::Space
		} else {
			CharType::Other
		}
	}
}

fn get_word_position(text: &str, mut position: usize) -> (usize, usize) {
	if text.is_empty() {
		return (0, 0);
	}
	if position >= text.len() {
		position = text.len() - 1;
	}
	let c: char = text.chars().nth(position).expect("position caca");
	let char_type = CharType::from_char(c);

	let mut end = position + 1;
	loop {
		if end >= text.len() {
			break;
		}
		let c: char = text.chars().nth(end).expect("forward oups");
		if CharType::from_char(c) != char_type {
			break;
		}
		end += 1;
	}

	let mut start = if position == 0 { 0 } else { position - 1 };
	loop {
		if start == 0 {
			break;
		}
		let c: char = text.chars().nth(start).expect("back oups");
		if CharType::from_char(c) != char_type {
			start += 1;
			break;
		}
		start -= 1;
	}

	(start, end)
}

impl Widget for TextInput {
	#[allow(clippy::diverging_sub_expression)]
	fn update(
		&mut self, input: &Input, delta: Duration, _: &mut Manager, text_drawer: &mut TextDrawer, camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;
		let now = Instant::now();
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
		let mouse_carrot_position = self.get_carrot_position(text_drawer, input.mouse.position, camera);
		let mut new_carrot_position = None;
		if input.keys_state.left.is_pressed() && self.carrot_position > 0 {
			let n: usize = if input.keys_state.lctrl.is_down() || input.keys_state.rctrl.is_down() {
				let pos = get_word_position(&self.text, self.carrot_position - 1);
				pos.0
			} else {
				self.carrot_position - 1
			};
			if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
				new_carrot_position = Some(n);
			} else {
				self.carrot_position = n;
				self.selection = None;
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}
		if input.keys_state.right.is_pressed() && self.carrot_position < self.text.len() {
			let n: usize = if input.keys_state.lctrl.is_down() || input.keys_state.rctrl.is_down() {
				let pos = get_word_position(&self.text, self.carrot_position);
				pos.1
			} else {
				self.carrot_position + 1
			};
			if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
				new_carrot_position = Some(n);
			} else {
				self.carrot_position = n;
				self.selection = None;
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}

		if input.mouse.left_button.is_down() && self.click_count < 2 {
			new_carrot_position = Some(mouse_carrot_position);
		}

		if now.duration_since(self.last_click) > Self::DOUBLE_CLICK_TIME {
			self.click_count = 0;
		}

		// Mouse click
		if input.mouse.left_button.is_pressed() {
			let mouse_carrot_position = self.get_carrot_position(text_drawer, input.mouse.position, camera);
			self.click_count += 1;
			self.last_click = now;
			self.carrot_timer_sec = Duration::ZERO;

			// check same position

			if mouse_carrot_position != self.click_position {
				self.click_count = 1;
				self.click_position = mouse_carrot_position;
			}

			match self.click_count {
				0 => unreachable!(),
				1 => {
					if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
						new_carrot_position = Some(mouse_carrot_position);
					} else {
						self.carrot_position = mouse_carrot_position;
						self.selection = None;
					}
				}
				2 => {
					let (start, end) = get_word_position(&self.text, mouse_carrot_position);
					self.selection = Some((start, end));
					self.carrot_position = end;
					new_carrot_position = None;
					changed = true;
				}
				_ => {
					self.selection = Some((0, self.text.len()));
					self.carrot_position = self.text.len();
					new_carrot_position = None;
					changed = true;
				}
			}
		}

		// Selection
		if let Some(new_carrot_position) = new_carrot_position {
			let (start, end) =
				if let Some((start, end)) = self.selection { (start, end) } else { (self.carrot_position, self.carrot_position) };
			if new_carrot_position != self.carrot_position {
				self.selection = Some(if new_carrot_position > self.carrot_position {
					if new_carrot_position > end {
						(start, new_carrot_position)
					} else {
						(new_carrot_position, end)
					}
				} else if new_carrot_position >= start {
					(start, new_carrot_position)
				} else {
					(new_carrot_position, end)
				});
				self.carrot_position = new_carrot_position;
				self.carrot_timer_sec = Duration::ZERO;
				changed = true;
			}
		}

		// Keyboard input
		// Clipboard
		if input.shortcut_pressed(&Shortcut::PASTE()) && input.clipboard.has_clipboard_text() {
			if let Some((start, end)) = self.selection {
				self.text.drain(start..end);
				self.carrot_position = start;
			}
			let clipboard_text = input.clipboard.clipboard_text().unwrap();
			self.text.insert_str(self.carrot_position, &clipboard_text);
			self.carrot_position += clipboard_text.len();
			self.selection = None;
			return true;
		}
		if input.shortcut_pressed(&Shortcut::COPY()) {
			if let Some((start, end)) = self.selection {
				let text = self.text[start..end].to_string();
				input.clipboard.set_clipboard_text(&text).unwrap();
				return true;
			}
			input.clipboard.set_clipboard_text(&self.text).unwrap();
			return true;
		}
		if input.shortcut_pressed(&Shortcut::CUT()) {
			if let Some((start, end)) = self.selection {
				let text = self.text.drain(start..end).collect::<String>();
				input.clipboard.set_clipboard_text(&text).unwrap();
				self.carrot_position = start;
				self.selection = None;
				return true;
			}
			input.clipboard.set_clipboard_text(&self.text).unwrap();
			self.text.clear();
			self.carrot_position = 0;
			return true;
		}

		// Text input
		if let Some(c) = input.last_char {
			if let Some((start, end)) = self.selection {
				self.text.drain(start..end);
				self.carrot_position = start;
			}

			self.text.insert(self.carrot_position, c);
			self.carrot_position += 1;
			self.selection = None;
			changed = true;
		}

		// Remove input (backspace / delete)
		if input.keys_state.backspace.is_pressed() {
			if let Some((start, end)) = self.selection {
				self.text.drain(start..end);
				self.carrot_position = start;
			} else if self.carrot_position > 0 {
				if input.keys_state.lctrl.is_down() || input.keys_state.lctrl.is_down() {
					let pos = get_word_position(&self.text, self.carrot_position - 1);
					self.text.drain(pos.0..self.carrot_position);
					self.carrot_position = pos.0;
				} else {
					self.text.remove(self.carrot_position - 1);
					self.carrot_position -= 1;
				}
			}
			self.selection = None;
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		} else if input.keys_state.delete.is_pressed() {
			if let Some((start, end)) = self.selection {
				self.text.drain(start..end);
				self.carrot_position = start;
				self.selection = None;
			} else if self.carrot_position < self.text.len() {
				if input.keys_state.lctrl.is_down() || input.keys_state.lctrl.is_down() {
					let pos = get_word_position(&self.text, self.carrot_position);
					self.text.drain(self.carrot_position..pos.1);
				} else {
					self.text.remove(self.carrot_position);
				}
			}
			self.carrot_timer_sec = Duration::ZERO;
			changed = true;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, camera: Option<&Camera>) {
		// Box
		let background_color = if self.is_hovered() { self.style.hovered_color } else { self.style.color };
		let mut border_color = self.style.border_color;
		if self.is_focused() {
			if let Some(focused_color) = self.style.focused_color {
				border_color = focused_color
			}
		};

		if let Some(corner_radius) = self.base.radius {
			if self.is_focused() && self.style.focused_color.is_some() {
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
			if self.is_focused() && self.style.focused_color.is_some() {
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

		if self.is_focused() {
			// Selection
			if let Some((start, end)) = self.selection {
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

	fn get_base(&self) -> &Base {
		&self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
