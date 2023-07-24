pub mod blank_box;
pub mod button;
pub mod draggable;
pub mod select;
pub mod slider;
pub mod switch;
pub mod text_input;
pub mod widgets_manager;

use crate::camera::Camera;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
pub use widgets_manager::WidgetsManager;
use as_any::{AsAny, Downcast};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use nalgebra::Point2;

pub const HOVER: f32 = 0.92;
pub const PUSH: f32 = 0.80;
pub const FOCUS_HALO_DELTA: f64 = 2.;
pub const FOCUS_HALO_ALPHA: u8 = 96;

pub enum Orientation {
	Horizontal,
	Vertical,
}

pub type WidgetId = u32;

#[derive(Copy, Clone, Debug)]
pub struct Base {
	pub id: WidgetId,
	pub rect: Rect,
	pub state: KeyState,
	visible: bool,
}

impl Default for Base {
	fn default() -> Self {
		Self { id: 0, state: KeyState::default(), visible: true, rect: Rect::new(0., 0., 200., 100.) }
	}
}

/// An struct that every widget must have
/// - id
/// - rect
/// - state
/// - visible
impl Base {
	pub fn new(rect: Rect) -> Self {
		Self { id: 0, rect, state: KeyState::default(), visible: true }
	}

	fn set_id(&mut self, id: WidgetId) {
		self.id = id;
	}

	pub fn update(&mut self, input: &Input, other_keys: Vec<KeyState>) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed()
			|| other_keys.iter().map(|key| key.is_pressed()).collect::<Vec<bool>>().contains(&true)
		{
			self.state.press();
			changed = true;
		} else if input.mouse.left_button.is_released()
			|| other_keys.iter().map(|key| key.is_released()).collect::<Vec<bool>>().contains(&true)
		{
			self.state.release();
			changed = true;
		}
		changed
	}

	pub fn is_pushed(&self) -> bool {
		self.state.is_pressed() || self.state.is_down()
	}

	pub fn is_visible(&self) -> bool {
		self.visible
	}

	pub fn set_visible(&mut self) {
		self.visible = true;
	}

	pub fn set_invisible(&mut self) {
		self.visible = false;
	}
}

/// A widget is a UI object that can be interacted with to take inputs from the user.
pub trait Widget: AsAny {
	/// Update the widget based on the inputs
	fn update(
		&mut self, input: &Input, delta: Duration, widgets_manager: &WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool;
	/// Draw the widget on the canvas
	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	);
	/// (Optional) This method is called when the widget is selected
	fn on_select(&mut self) {}
	/// (Optional) This method is called when the widget is unselected
	fn on_unselect(&mut self) {}
	/// (Optional, the default is the base rect) The hit-box of the Widget
	fn collide_point(&self, point: Point2<f64>) -> bool {
		self.get_base().rect.collide_point(point)
	}
	
	fn get_base(&self) -> Base;
	fn get_base_mut(&mut self) -> &mut Base;
}
