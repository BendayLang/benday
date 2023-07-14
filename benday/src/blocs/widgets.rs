use crate::blocs::containers::Slot;
use crate::blocs::Bloc;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{paler, Colors};
use pg_sdl::input::{Input, Shortcut};
use pg_sdl::primitives::{draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub struct BaseWidgetS {
	pub position: Point2<f64>,
	pub size: Vector2<f64>,
}

impl BaseWidgetS {
	pub fn new(position: Point2<f64>, size: Vector2<f64>) -> Self {
		Self { position, size }
	}
}

/// A widget is a UI object that can be interacted with to take inputs from the user.
pub trait WidgetS {
	/// Update the widget based on the inputs
	fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool;
	/// Draw the widget on the canvas
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool);

	fn get_base_widget(&self) -> &BaseWidgetS;

	fn get_base_widget_mut(&mut self) -> &mut BaseWidgetS;
}

pub struct TextBoxS {
	base_widget: BaseWidgetS,
	default_text: String,
	default_color: Color,
	color: Color,
	content: String,
	carrot_timer_sec: f64,
	carrot_position: usize,
	selection: Option<(usize, usize)>,
}

impl TextBoxS {
	const FONT_SIZE: f64 = 12.0;
	const LEFT_SHIFT: i32 = 5;
	const BLINKING_TIME_SEC: f64 = 0.4;

	pub fn new(base_widget: BaseWidgetS, color: Color, default_text: String) -> Self {
		Self {
			base_widget,
			default_text,
			default_color: paler(color, 0.2),
			color: paler(color, 0.5),
			content: String::new(),
			carrot_timer_sec: 0.0,
			carrot_position: 0,
			selection: None,
		}
	}
	pub fn get_text(&self) -> String {
		self.content.clone()
	}

	fn get_carrot_position(&self, mouse_position: Point2<i32>, text_drawer: &mut TextDrawer, camera: &Camera) -> Option<usize> {
		let mouse_x = mouse_position.x - self.base_widget.position.x as i32;
		let mut x: u32 = 0;
		for (i, c) in self.content.chars().enumerate() {
			let text_style = &TextStyle { font_size: (camera.scale() * Self::FONT_SIZE) as u16, ..Default::default() };
			let text_width = text_drawer.text_size(&text_style, &c.to_string()).0;
			x += text_width;
			if x >= mouse_x as u32 {
				return Some(i);
			}
		}
		return None;
	}
}

impl WidgetS for TextBoxS {
	fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;

		// Carrot blinking
		self.carrot_timer_sec += delta_sec;
		if Self::BLINKING_TIME_SEC < self.carrot_timer_sec && self.carrot_timer_sec < Self::BLINKING_TIME_SEC + delta_sec {
			changed = true;
		}
		if self.carrot_timer_sec > 2.0 * Self::BLINKING_TIME_SEC {
			self.carrot_timer_sec = 0.0;
			changed = true;
		}

		if input.mouse.left_button_double_clicked() {
			// Mouse double click
			self.selection = Some((0, self.content.len()));
			changed = true;
		} else if input.mouse.left_button.is_pressed() {
			// Mouse click
			self.selection = None;

			// Carrot position
			self.carrot_position =
				if let Some(new_carrot_position) = self.get_carrot_position(input.mouse.position, text_drawer, camera) {
					new_carrot_position
				} else {
					self.content.len()
				};

			// Selection
			self.carrot_timer_sec = 0.0;
			changed = true;
		} else if input.mouse.left_button.is_down() {
			// Selection
			let new_carrot_position =
				if let Some(new_carrot_position) = self.get_carrot_position(input.mouse.position, text_drawer, camera) {
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
		let color = if selected { self.default_color } else { self.color };
		fill_rounded_rect(canvas, Some(camera), color, self.base_widget.position, self.base_widget.size, Slot::RADIUS);
		let text_position = self.base_widget.position + Vector2::new(5.0, self.base_widget.size.y * 0.5);
		if self.content.is_empty() {
			draw_text(
				canvas,
				Some(camera),
				text_drawer,
				Colors::GREY,
				text_position,
				Self::FONT_SIZE,
				self.default_text.clone(),
				Align::Left,
			);
		} else {
			draw_text(
				canvas,
				Some(camera),
				text_drawer,
				Colors::BLACK,
				text_position,
				Self::FONT_SIZE,
				self.content.clone(),
				Align::Left,
			);
		}
		if selected {
			draw_rounded_rect(
				canvas,
				Some(camera),
				Colors::BLACK,
				self.base_widget.position,
				self.base_widget.size,
				Slot::RADIUS,
			);
		}
		if hovered {
			let hovered_color = Color::from((0, 0, 0, Bloc::HOVER_ALPHA));
			canvas.set_blend_mode(BlendMode::Mod);
			fill_rounded_rect(
				canvas,
				Some(camera),
				hovered_color,
				self.base_widget.position,
				self.base_widget.size,
				Slot::RADIUS,
			);
			canvas.set_blend_mode(BlendMode::None);
		}
	}

	fn get_base_widget(&self) -> &BaseWidgetS {
		&self.base_widget
	}

	fn get_base_widget_mut(&mut self) -> &mut BaseWidgetS {
		&mut self.base_widget
	}
}
