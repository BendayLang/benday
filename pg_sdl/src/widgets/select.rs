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

use super::button::{Button, ButtonStyle};
use super::text_input::{TextInput, TextInputStyle};
use super::WidgetId;

#[derive(Default)]
pub struct SelectStyle {}

pub struct Select {
	base: Base,
	style: SelectStyle,
	options: Vec<String>,
	selected: usize,
	text_input: TextInput,
	drop_button: Button,
}

impl Select {
	#[allow(non_snake_case)]
	fn DEFAULT_RECT() -> Rect {
		Rect::new(0., 0., 200., 50.)
	}
}

impl Default for Select {
	fn default() -> Self {
		let rect = Self::DEFAULT_RECT();
		let drop_button_size: Vector2<f64> = Vector2::new(50., 50.);
		let drop_button_rect = Rect::new(
			rect.width() - drop_button_size.x,
			rect.height() - drop_button_size.y,
			drop_button_size.x,
			drop_button_size.y,
		);
		let mut text_input = TextInput::new(rect, TextInputStyle::default(), "select".to_string());
		text_input.get_base_mut().id = WidgetId::MAX;
		let mut drop_button = Button::new(drop_button_rect, ButtonStyle::default(), ">".to_string());
		drop_button.get_base_mut().id = WidgetId::MAX - 1;
		Self { base: Base::new(rect), style: SelectStyle::default(), options: vec![], selected: 0, text_input, drop_button }
	}
}

impl Widget for Select {
	fn update(
		&mut self, input: &Input, delta_sec: f64, widgets_manager: &mut WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let mut changed = false;
		changed |= self.base.update(input, Vec::new());
		changed |= self.text_input.update(input, delta_sec, widgets_manager, text_drawer, camera);
		changed |= self.drop_button.update(input, delta_sec, widgets_manager, text_drawer, camera);

		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		self.text_input.draw(canvas, text_drawer, camera, focused, hovered);
		self.drop_button.draw(canvas, text_drawer, camera, focused, hovered);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
