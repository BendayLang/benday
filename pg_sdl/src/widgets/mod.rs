pub mod blank_box;
pub mod button;
pub mod draggable;
pub mod manager;
pub mod select;
pub mod slider;
pub mod switch;
pub mod text_input;

use crate::camera::Camera;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
use as_any::{AsAny, Downcast};
pub use manager::Manager;
use nalgebra::Point2;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

pub const HOVER: f32 = 0.92;
pub const PUSH: f32 = 0.80;
pub const FOCUS_HALO_DELTA: f64 = 2.;
pub const FOCUS_HALO_ALPHA: u8 = 96;

pub enum Orientation {
	Horizontal,
	Vertical,
}

pub type WidgetId = u32;

pub type SignalHandler = HashMap<String, Box<dyn Fn(Box<&mut dyn Widget>)>>;

pub struct Signal {
	receivers: Vec<WidgetId>,
	name: String,
}

pub struct Base {
	id: WidgetId,
	pub rect: Rect,
	pub state: KeyState,
	focused: bool,
	hovered: bool,
	visible: bool,
	pub parent_id: Option<WidgetId>,
	has_scroll: bool,
}

/// An struct that every widget must have
/// - id
/// - rect
/// - state
/// - visible
impl Base {
	pub fn new(rect: Rect, has_scroll: bool) -> Self {
		Self {
			id: 0,
			rect,
			state: KeyState::default(),
			focused: false,
			hovered: false,
			visible: true,
			parent_id: None,
			has_scroll,
		}
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
}

/// A widget is a UI object that can be interacted with to take inputs from the user.
pub trait Widget: AsAny {
	/// Update the widget based on the inputs
	fn update(
		&mut self, input: &Input, delta: Duration, manager: &mut Manager, text_drawer: &TextDrawer, camera: Option<&Camera>,
	) -> bool;
	/// Draw the widget on the canvas
	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, camera: Option<&Camera>);

	fn get_base(&self) -> &Base;

	fn get_base_mut(&mut self) -> &mut Base;

	/// Returns the id (used in the manager) of the widget
	fn get_id(&self) -> &WidgetId {
		&self.get_base().id
	}

	/// (Optional) This method is called when the widget is focused
	fn on_focus(&mut self, manager: &mut Manager) {}
	/// (Optional) This method is called when the widget is unfocused
	fn on_unfocus(&mut self, manager: &mut Manager) {}
	/// (Optional) This method is called when the widget begins being hovered
	fn on_hover(&mut self, manager: &mut Manager) {}
	/// (Optional) This method is called when the widget ends being hovered
	fn on_unhover(&mut self, manager: &mut Manager) {}
	/// (Optional, the default is the base rect) The hit-box of the Widget
	fn collide_point(&self, point: Point2<f64>) -> bool {
		self.get_base().rect.collide_point(point)
	}
	/// (Should not be overwritten) Indicates if the widget is visible
	fn is_visible(&self) -> bool {
		self.get_base().visible
	}
	/// (Should not be overwritten) Makes the widget visible
	fn set_visible(&mut self) {
		self.get_base_mut().visible = true;
	}
	/// (Should not be overwritten) Makes the widget invisible
	fn set_invisible(&mut self) {
		self.get_base_mut().visible = false;
	}
	/// (Should not be overwritten) Indicates if the widget is currently focused
	fn is_focused(&self) -> bool {
		self.get_base().focused
	}
	/// (Should not be overwritten) Indicates if the widget is currently hovered
	fn is_hovered(&self) -> bool {
		self.get_base().hovered
	}
}
