pub mod blank_box;
pub mod button;
pub mod draggable;
pub mod select;
pub mod slider;
pub mod switch;
pub mod text_input;

use crate::camera::Camera;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
use as_any::{AsAny, Downcast};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fmt::Debug;

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
		&mut self, input: &Input, delta_sec: f64, widgets_manager: &mut WidgetsManager, text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool;
	/// Draw the widget on the canvas
	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	);

	fn get_base(&self) -> Base;
	fn get_base_mut(&mut self) -> &mut Base;
}

#[derive(Default)]
pub struct WidgetsManager {
	widgets: HashMap<WidgetId, Box<dyn Widget>>,
	no_cam_order: Vec<WidgetId>,
	cam_order: Vec<WidgetId>,
	id_counter: WidgetId,
	focused_widget: Option<WidgetId>,
	hovered_widget: Option<WidgetId>,
}

impl WidgetsManager {
	pub fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;

		// Update witch widget is focused (Mouse click)
		if input.mouse.left_button.is_pressed() {
			self.focused_widget = self.hovered_widget.as_ref().copied();
			changed = true;
		} else if input.keys_state.escape.is_pressed() && self.focused_widget.is_some() {
			self.focused_widget = None;
			changed = true;
		}
		// TAB to navigate between widgets
		else if input.keys_state.tab.is_pressed() {
			if let Some(focused_widget) = &self.focused_widget {
				// TODO pass the not visibles widgets when tab
				self.focused_widget = if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					Some(if focused_widget == &0 { self.id_counter - 1 } else { focused_widget - 1 })
				} else {
					Some(if focused_widget == &(self.id_counter - 1) { 0 } else { focused_widget + 1 })
				};
				changed = true;
			}
		}

		// Update witch widget is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() && !input.mouse.left_button.is_down() {
			let mut new_hovered_widget = None;
			let mouse_position = input.mouse.position.cast();
			// checks collisions with the widgets without camera first
			for id in self.no_cam_order.iter().rev() {
				let widget_base = self.widgets.get(id).unwrap().get_base();
				if widget_base.is_visible() && widget_base.rect.collide_point(mouse_position) {
					new_hovered_widget = Some(*id);
					break;
				}
			}
			// checks collisions with the widgets with camera if none without was hovered
			if new_hovered_widget.is_none() {
				let mouse_position = camera.transform().inverse() * mouse_position;
				for id in self.cam_order.iter().rev() {
					let widget_base = self.widgets.get(id).unwrap().get_base();
					if widget_base.is_visible() && widget_base.rect.collide_point(mouse_position) {
						new_hovered_widget = Some(*id);
						break;
					}
				}
			}
			if new_hovered_widget != self.hovered_widget {
				self.hovered_widget = new_hovered_widget;
				changed = true;
			}
		}

		// Update the focused widget (if there is one)
		if let Some(id) = self.focused_widget {
			let camera = if self.widget_has_no_camera(id) { None } else { Some(camera) };
			let mut focused_widget = self.widgets.remove(&id).unwrap();
			changed |= focused_widget.update(input, delta_sec, self, text_drawer, camera);
			self.widgets.insert(id, focused_widget);
		}

		changed
	}

	pub fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: &Camera) {
		// draws the widgets with camera first
		self.cam_order.iter().for_each(|id| {
			let widget = self.widgets.get(id).unwrap();
			if widget.get_base().is_visible() {
				let focused = Some(*id) == self.focused_widget;
				let hovered = Some(*id) == self.hovered_widget;
				widget.draw(canvas, text_drawer, Some(camera), focused, hovered);
			}
		});
		// draws the widgets without camera on top
		self.no_cam_order.iter().for_each(|id| {
			let widget = self.widgets.get(id).unwrap();
			if widget.get_base().is_visible() {
				let focused = Some(*id) == self.focused_widget;
				let hovered = Some(*id) == self.hovered_widget;
				widget.draw(canvas, text_drawer, None, focused, hovered);
			}
		});
	}

	/// Adds the given widget to the widgets manager and returns it's id
	pub fn add_widget(&mut self, widget: Box<dyn Widget>, has_camera: bool) -> WidgetId {
		let id = self.id_counter;
		self.widgets.insert(id, widget);
		self.widgets.get_mut(&id).unwrap().get_base_mut().set_id(id);
		if has_camera {
			self.cam_order.push(id)
		} else {
			self.no_cam_order.push(id)
		}
		self.id_counter += 1;
		id
	}

	#[allow(clippy::borrowed_box)]
	pub fn get_widget(&self, id: &WidgetId) -> Option<&Box<dyn Widget>> {
		self.widgets.get(id)
	}

	pub fn get_widget_mut(&mut self, id: &WidgetId) -> Option<&mut Box<dyn Widget>> {
		self.widgets.get_mut(id)
	}

	pub fn get<T: Widget>(&self, id: &WidgetId) -> Option<&T> {
		self.widgets.get(id).and_then(|w| w.as_ref().downcast_ref::<T>())
	}

	pub fn get_mut<T: Widget>(&mut self, id: &WidgetId) -> Option<&mut T> {
		self.widgets.get_mut(id).and_then(|w| w.as_mut().downcast_mut::<T>())
	}

	pub fn remove(&mut self, id: &WidgetId) -> Option<Box<dyn Widget>> {
		self.widgets.remove(id)
	}

	pub fn insert(&mut self, id: WidgetId, widget: Box<dyn Widget>) {
		self.widgets.insert(id, widget);
	}

	pub fn get_cam_order(&self) -> &Vec<WidgetId> {
		&self.cam_order
	}

	/// Puts the given widget on top of the others (the widget needs to not have a camera)
	pub fn put_on_top_no_cam(&mut self, id: &WidgetId) {
		let index = self.no_cam_order.iter().position(|i| i == id).unwrap();
		self.no_cam_order.remove(index);
		self.no_cam_order.push(*id);
	}
	/// Puts the given widget on top of the others (the widget needs to have a camera)
	pub fn put_on_top_cam(&mut self, id: &WidgetId) {
		let index = self.cam_order.iter().position(|i| i == id).unwrap();
		self.cam_order.remove(index);
		self.cam_order.push(*id);
	}

	fn widget_has_no_camera(&self, id: WidgetId) -> bool {
		self.no_cam_order.contains(&id)
	}

	/// Returns the id of the last added widget
	pub fn last_id(&self) -> WidgetId {
		self.id_counter - 1
	}

	pub fn focused_widget(&self) -> Option<WidgetId> {
		self.focused_widget
	}

	// TODO: remove this and replace with a macro that right all the code for us, and for every widget type
	/*
	pub fn get_mut_button(&mut self, id: WidgetId) -> &mut Button {
		if let Some(button) = self.get_mut(id) {
			button
		} else {
			panic!("Button '{}' not found", id);
		}
	}

	pub fn get_button(&self, id: WidgetId) -> &Button {
		if let Some(button) = self.get(id) {
			button
		} else {
			panic!("Button '{}' not found", id);
		}
	}

	pub fn get_mut_slider(&mut self, id: WidgetId) -> &mut Slider {
		if let Some(slider) = self.get_mut(id) {
			slider
		} else {
			panic!("Slider '{}' not found", id);
		}
	}

	pub fn get_slider(&self, id: WidgetId) -> &Slider {
		if let Some(slider) = self.get(id) {
			slider
		} else {
			panic!("Slider '{}' not found", id);
		}
	}
	 */
}
