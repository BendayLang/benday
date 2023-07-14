use nalgebra::{Point2, Vector2};
use crate::primitives::{draw_rect, draw_rounded_rect, fill_rect, fill_rounded_rect};
use crate::input::{KeyState, KeysState, Shortcut, Input};
use crate::widgets::{HOVER, PUSH, SELECTED_COLOR, Widget};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use crate::camera::Camera;
use crate::color::{Colors, darker, paler};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};

pub struct TextBoxStyle {
	background_color: Color,
	background_hovered_color: Color,
	background_pushed_color: Color,
	contour_color: Color,
	corner_radius: Option<f64>,
	text_style: TextStyle,
}

impl Default for TextBoxStyle {
	fn default() -> Self {
		Self {
			background_color: Colors::WHITE,
			background_hovered_color: darker(Colors::WHITE, HOVER),
			background_pushed_color: darker(Colors::WHITE, PUSH),
			contour_color: Colors::BLACK,
			corner_radius: Some(4.0),
			text_style: TextStyle::default(),
		}
	}
}

pub struct TextBox {
	position: Point2<f64>,
	size: Vector2<f64>,
	style: TextBoxStyle,
	pub content: String,
	carrot_timer_sec: f64,
	carrot_position: usize,
	selection: Option<(usize, usize)>,
	is_selecting: bool,
	pub state: KeyState,
	has_camera: bool,
}

impl TextBox {
	const LEFT_SHIFT: f64 = 5.0;
	const BLINKING_TIME_SEC: f64 = 0.4;

	pub fn new(position: Point2<f64>, size: Vector2<f64>, style: Option<TextBoxStyle>, default_text: Option<String>, has_camera: bool) -> Self {
		let carrot_position = match default_text {
			Some(ref text) => text.len(),
			None => 0,
		};
		Self {
			position,
			size,
			style: style.unwrap_or_default(),
			content: default_text.unwrap_or_default(),
			state: KeyState::new(),
			carrot_timer_sec: 0.0,
			carrot_position,
			selection: None,
			is_selecting: false,
			has_camera,
		}
	}

	fn get_carrot_position_from_mouse(&self, text_drawer: &TextDrawer, mouse_x: f64) -> Option<usize> {
		let mut x: u32 = 0;
		for (i, c) in self.content.chars().enumerate() {
			let text_width = text_drawer.text_size(&self.style.text_style, &c.to_string()).0;
			x += text_width;
			if x >= mouse_x as u32 {
				return Some(i);
			}
		}
		return None;
	}

	fn is_carrot_visible(&self) -> bool {
		self.carrot_timer_sec < Self::BLINKING_TIME_SEC
	}
}

impl Widget for TextBox {
	fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		// Carrot blinking
		self.carrot_timer_sec += delta_sec;
		if Self::BLINKING_TIME_SEC < self.carrot_timer_sec && self.carrot_timer_sec < Self::BLINKING_TIME_SEC + delta_sec {
			changed = true;
		}
		if self.carrot_timer_sec > 2.0 * Self::BLINKING_TIME_SEC {
			self.carrot_timer_sec = 0.0;
			changed = true;
		}

		// Mouse click
		if input.mouse.left_button_double_clicked() {
			self.selection = Some((0, self.content.len()));
			changed = true;
		} else if input.mouse.left_button.is_pressed() {
			self.selection = None;

			// Carrot position
			let mouse_x = input.mouse.position.x as f64 - self.position.x;
			self.carrot_position =
				if let Some(new_carrot_position) = self.get_carrot_position_from_mouse(text_drawer, mouse_x) {
					new_carrot_position
				} else {
					self.content.len()
				};

			// Selection
			self.state.press();
			self.is_selecting = true;
			self.carrot_timer_sec = 0.0;
			changed = true;
		} else if input.mouse.left_button.is_down() && self.is_selecting {
			// Selection
			let mouse_x = input.mouse.position.x as f64 - self.position.x;
			let new_carrot_position =
				if let Some(new_carrot_position) = self.get_carrot_position_from_mouse(text_drawer, mouse_x) {
					new_carrot_position
				} else {
					self.content.len()
				};
			if new_carrot_position != self.carrot_position {
				if self.carrot_position > new_carrot_position {
					self.selection = Some((new_carrot_position, self.carrot_position));
				} else {
					self.selection = Some((self.carrot_position, new_carrot_position));
				}
				changed = true;
			}
		}

		if self.is_selecting && input.mouse.left_button.is_released() {
			self.is_selecting = false;
			changed = true;
		}

		if self.state.is_down() && input.mouse.left_button.is_released() {
			self.state.release();
			changed = true;
		}

		// Keyboard input
		// Clipboard
		if input.shortcut_pressed(&Shortcut::PASTE()) && input.clipboard.has_clipboard_text() {
			if self.selection.is_some() {
				let (start, end) = self.selection.unwrap();
				self.content.drain(start..end);
				self.carrot_position = start;
				self.selection = None;
			}
			let clipboard_text = input.clipboard.clipboard_text().unwrap();
			self.content.insert_str(self.carrot_position, &clipboard_text);
			self.carrot_position = self.carrot_position + clipboard_text.len();
			return true;
		}
		if input.shortcut_pressed(&Shortcut::COPY()) {
			if self.selection.is_some() {
				let (start, end) = self.selection.unwrap();
				let text = self.content[start..end].to_string();
				input.clipboard.set_clipboard_text(&text).unwrap();
				return true;
			}
			input.clipboard.set_clipboard_text(&self.content).unwrap();
			return true;
		}
		if input.shortcut_pressed(&Shortcut::CUT()) {
			if self.selection.is_some() {
				let (start, end) = self.selection.unwrap();
				let text = self.content.drain(start..end).collect::<String>();
				input.clipboard.set_clipboard_text(&text).unwrap();
				self.carrot_position = start;
				self.selection = None;
				return true;
			}
			input.clipboard.set_clipboard_text(&self.content).unwrap();
			self.content.clear();
			self.carrot_position = 0;
			return true;
		}

		// Text input
		if let Some(c) = input.last_char {
			if let Some((start, end)) = self.selection {
				self.content.drain(start..end);
				self.carrot_position = start;
				self.selection = None;
			}
			changed = true;
			self.content.insert(self.carrot_position, c);
			if self.carrot_position < self.content.len() {
				self.carrot_position += 1;
			}
		}
		if input.keys_state.backspace.is_pressed() {
			if self.selection.is_some() {
				let (start, end) = self.selection.unwrap();
				self.content.drain(start..end);
				self.carrot_position = start;
				self.selection = None;
			} else if self.carrot_position > 0 {
				self.content.remove(self.carrot_position - 1);
				self.carrot_position -= 1;
			}
			self.carrot_timer_sec = 0.0;
			changed = true;
		}

		// Carrot movement
		if input.keys_state.left.is_pressed() {
			if self.carrot_position > 0 {
				self.carrot_position -= 1;
			}
			self.carrot_timer_sec = 0.0;
			changed = true;
		}
		if input.keys_state.right.is_pressed() {
			if self.carrot_position < self.content.len() {
				self.carrot_position += 1;
			}
			self.carrot_timer_sec = 0.0;
			changed = true;
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool) {
		// Box
		let background_color = if hovered { self.style.background_hovered_color } else { self.style.background_color };
		
		let camera = if self.has_camera { Some(camera) } else { None };
		if let Some(corner_radius) = self.style.corner_radius {
			fill_rounded_rect(canvas, camera, background_color, self.position, self.size, corner_radius);
			draw_rounded_rect(canvas, camera, self.style.contour_color, self.position, self.size, corner_radius);
		} else {
			fill_rect(canvas, camera, background_color, self.position, self.size);
			draw_rect(canvas, camera, self.style.contour_color, self.position, self.size);
		}

		if selected {
			let position = Point2::new(self.position.x + 1.0, self.position.y + 1.0);
			let size = Vector2::new(self.size.x - 2.0, self.size.y - 2.0);
			if let Some(corner_radius) = self.style.corner_radius {
				draw_rounded_rect(canvas, camera, SELECTED_COLOR, position, size, corner_radius - 1.0);
			} else {
				draw_rect(canvas, camera, SELECTED_COLOR, position, size);
			}
			// Selection
			if let Some(selection) = self.selection {
				let position = Point2::new(
					self.position.x + 5.0 + text_drawer.text_size(&self.style.text_style, &self.content[..selection.0]).0 as f64,
					self.position.y + 5.0);
				let size = Vector2::new(
					text_drawer.text_size(&self.style.text_style, &self.content[selection.0..selection.1]).0 as f64,
					self.size.y - 10.0,
				);
				let mut selection_color = Colors::LIGHT_BLUE;
				selection_color.a = 100;
				canvas.set_blend_mode(BlendMode::Mod);
				fill_rect(canvas, camera, selection_color, position, size);
				canvas.set_blend_mode(BlendMode::None);
			}
		}

		// Text
		if !self.content.is_empty() {
			text_drawer.draw(
				canvas,
				Point2::new(self.position.x + Self::LEFT_SHIFT, self.size.y * 0.5 + self.position.y),
				&self.style.text_style,
				&self.content,
				Align::Left,
			);
		}

		// Carrot
		if selected && self.is_carrot_visible() {
			let carrot_x_position = if self.carrot_position != 0 {
				text_drawer.text_size(&self.style.text_style, &self.content[..self.carrot_position]).0 as f64
			} else {
				0.0
			};

			let position = Point2::new(self.position.x + 5.0 + carrot_x_position, self.position.y + 5.0);
			let size = Vector2::new(1.0, self.size.y - 10.0);
			fill_rect(canvas, camera, Colors::BLACK, position, size);
		}
	}
	
	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool {
		let point = if self.has_camera{ camera.transform * point } else { point };
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}
}
