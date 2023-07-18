pub mod button;
pub mod draggable;
pub mod slider;
pub mod switch;
pub mod text_input;

use crate::camera;
use crate::camera::Camera;
use crate::color::Colors;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
use as_any::{AsAny, Downcast};
use nalgebra::{Complex, Point2, RealField, Similarity, Similarity2, Transform2, Unit, Vector2};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Add, Mul};

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
}

/// An struct that every widget must have
/// - id
/// - rect
/// - state
impl Base {
	pub fn new(rect: Rect) -> Self {
		Self { id: 0, rect, state: KeyState::new() }
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

	pub fn pushed(&self) -> bool {
		self.state.is_pressed() || self.state.is_down()
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

pub struct WidgetsManager {
	widgets: HashMap<WidgetId, Box<dyn Widget>>,
	no_cam_order: Vec<WidgetId>,
	cam_order: Vec<WidgetId>,
	id_counter: WidgetId,
	focused_widget: Option<WidgetId>,
	hovered_widget: Option<WidgetId>,
}

impl WidgetsManager {
	pub fn new() -> Self {
		WidgetsManager {
			widgets: HashMap::new(),
			no_cam_order: Vec::new(),
			cam_order: Vec::new(),
			id_counter: 0,
			focused_widget: None,
			hovered_widget: None,
		}
	}

	pub fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;

		// Update witch widget is focused (Mouse click)
		if input.mouse.left_button.is_pressed() {
			self.focused_widget = if let Some(name) = &self.hovered_widget { Some(name.clone()) } else { None };
			changed = true;
		} else if input.keys_state.escape.is_pressed() && self.focused_widget.is_some() {
			self.focused_widget = None;
			changed = true;
		} else if input.keys_state.tab.is_pressed() {
			if let Some(focused_widget) = &self.focused_widget {
				self.focused_widget = if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					Some(if focused_widget == &0 { self.id_counter - 1 } else { focused_widget - 1 })
				} else {
					Some(if focused_widget == &(self.id_counter - 1) { 0 } else { focused_widget + 1 })
				};
				changed = true;
			}
		}

		// Update witch widget is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() {
			let mut new_hovered_widget = None;
			let mouse_position = input.mouse.position.cast();
			// checks collisions with the widgets without camera first
			for id in self.no_cam_order.iter().rev() {
				if self.widgets.get(id).unwrap().get_base().rect.collide_point(mouse_position) {
					new_hovered_widget = Some(id.clone());
					break;
				}
			}
			// checks collisions with the widgets with camera if none without was hovered
			if new_hovered_widget.is_none() {
				for id in self.cam_order.iter().rev() {
					if self.widgets.get(id).unwrap().get_base().rect.collide_point(camera.transform().inverse() * mouse_position)
					{
						new_hovered_widget = Some(id.clone());
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
			let mut focused_widget = self.widgets.remove(&id).unwrap();
			let camera = if self.widget_has_no_camera(id) { None } else { Some(camera) };
			changed |= focused_widget.update(input, delta_sec, self, text_drawer, camera);
			self.widgets.insert(id, focused_widget);
		}

		changed
	}

	pub fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: &Camera) {
		// draws the widgets with camera first
		self.cam_order.iter().for_each(|id| {
			let focused = Some(*id) == self.focused_widget;
			let hovered = Some(*id) == self.hovered_widget;
			self.widgets.get(id).unwrap().draw(canvas, text_drawer, Some(camera), focused, hovered);
		});
		// draws the widgets without camera on top
		self.no_cam_order.iter().for_each(|id| {
			let focused = Some(*id) == self.focused_widget;
			let hovered = Some(*id) == self.hovered_widget;
			self.widgets.get(id).unwrap().draw(canvas, text_drawer, None, focused, hovered);
		});
	}

	pub fn add(&mut self, widget: Box<dyn Widget>, has_camera: bool) {
		self.widgets.insert(self.id_counter, widget);
		self.widgets.get_mut(&self.id_counter).unwrap().get_base_mut().set_id(self.id_counter);
		if has_camera {
			self.cam_order.push(self.id_counter)
		} else {
			self.no_cam_order.push(self.id_counter)
		}
		self.id_counter += 1;
	}

	pub fn get<T: Widget>(&self, id: WidgetId) -> Option<&T> {
		self.widgets.get(&id).and_then(|w| w.as_ref().downcast_ref::<T>())
	}

	pub fn get_mut<T: Widget>(&mut self, id: WidgetId) -> Option<&mut T> {
		self.widgets.get_mut(&id).and_then(|w| w.as_mut().downcast_mut::<T>())
	}

	/// Puts the given widget on top of the others (the widget needs to not have a camera)
	pub fn put_on_top_no_cam(&mut self, id: WidgetId) {
		let index = self.no_cam_order.iter().position(|i| i == &id).unwrap();
		self.no_cam_order.remove(index);
		self.no_cam_order.push(id);
	}
	/// Puts the given widget on top of the others (the widget needs to have a camera)
	pub fn put_on_top_cam(&mut self, id: WidgetId) {
		let index = self.cam_order.iter().position(|i| i == &id).unwrap();
		self.cam_order.remove(index);
		self.cam_order.push(id);
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
