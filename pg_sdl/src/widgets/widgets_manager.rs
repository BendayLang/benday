use std::any::TypeId;
use std::cell::{Cell, Ref, RefCell, RefMut};
use crate::camera::Camera;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
use crate::widgets::{Widget, WidgetId};
use as_any::{AsAny, Downcast};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::time::Duration;


#[derive(Default)]
pub struct WidgetsManager {
	widgets: HashMap<WidgetId, RefCell<Box<dyn Widget>>>,
	no_cam_order: RefCell<Vec<WidgetId>>,
	cam_order: RefCell<Vec<WidgetId>>,
	id_counter: WidgetId,
	focused_widget_id: Option<WidgetId>,
	hovered_widget_id: Option<WidgetId>,
}

impl WidgetsManager {
	pub fn update(&mut self, input: &Input, delta: Duration, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;

		let mut new_focused_widget_id = self.focused_widget_id.as_ref().copied();
		// Update witch widget is focused (Mouse click)
		if input.mouse.left_button.is_pressed() {
			if self.focused_widget_id != self.hovered_widget_id {
				new_focused_widget_id = self.hovered_widget_id.as_ref().copied();
			}
		// Escape key pressed
		} else if input.keys_state.escape.is_pressed() {
			new_focused_widget_id = None;
		}
		// TAB to navigate between widgets
		else if input.keys_state.tab.is_pressed() {
			if let Some(focused_widget) = &self.focused_widget_id {
				// TODO pass the not visibles widgets when tab
				new_focused_widget_id = if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					Some(if focused_widget == &0 { self.id_counter - 1 } else { focused_widget - 1 })
				} else {
					Some(if focused_widget == &(self.id_counter - 1) { 0 } else { focused_widget + 1 })
				};
			}
		}
		
		if new_focused_widget_id != self.focused_widget_id {
			self.unselect_widget();
			if let Some(new_focused_widget_id) = new_focused_widget_id {
				self.select_widget(new_focused_widget_id);
			}
			changed = true;
		}
		
		// Update witch widget is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() && !input.mouse.left_button.is_down() {
			let mut new_hovered_widget = None;
			let mouse_position = input.mouse.position.cast();
			// checks collisions with the widgets without camera first
			for id in self.no_cam_order.borrow().iter().rev() {
				let widget = self.get_widget(id);
				if widget.get_base().is_visible() && widget.collide_point(mouse_position) {
					new_hovered_widget = Some(*id);
					break;
				}
			}
			// checks collisions with the widgets with camera if none without was hovered
			if new_hovered_widget.is_none() {
				let mouse_position = camera.transform().inverse() * mouse_position;
				for id in self.cam_order.borrow().iter().rev() {
					let widget = self.get_widget(id);
					if widget.get_base().is_visible() && widget.collide_point(mouse_position) {
						new_hovered_widget = Some(*id);
						break;
					}
				}
			}
			if new_hovered_widget != self.hovered_widget_id {
				self.hovered_widget_id = new_hovered_widget;
				changed = true;
			}
		}

		// Update the focused widget (if there is one)
		if let Some(id) = self.focused_widget_id {
			let camera = if self.widget_has_no_camera(id) { None } else { Some(camera) };
			changed |= self.get_widget_mut(&id).update(input, delta, self, text_drawer, camera);
		}

		changed
	}

	pub fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: &Camera) {
		// draws the widgets with camera first
		self.cam_order.borrow().iter().for_each(|id| {
			let widget = self.get_widget(id);
			if widget.get_base().is_visible() {
				let focused = Some(*id) == self.focused_widget_id;
				let hovered = Some(*id) == self.hovered_widget_id;
				widget.draw(canvas, text_drawer, Some(camera), focused, hovered);
			}
		});
		// draws the widgets without camera on top
		self.no_cam_order.borrow().iter().for_each(|id| {
			let widget = self.get_widget(id);
			if widget.get_base().is_visible() {
				let focused = Some(*id) == self.focused_widget_id;
				let hovered = Some(*id) == self.hovered_widget_id;
				widget.draw(canvas, text_drawer, None, focused, hovered);
			}
		});
	}

	/// Adds the given widget to the widgets manager and returns it's id
	pub fn add_widget(&mut self, widget: Box<dyn Widget>, has_camera: bool) -> WidgetId {
		let id = self.id_counter;
		self.widgets.insert(id, RefCell::new(widget));
		self.get_widget_mut(&id).get_base_mut().set_id(id);
		if has_camera {
			self.cam_order.borrow_mut().push(id)
		} else {
			self.no_cam_order.borrow_mut().push(id)
		}
		self.id_counter += 1;
		id
	}
	
	pub fn select_widget(&mut self, id: WidgetId) {
		self.get_widget_mut(&id).on_select();
		self.focused_widget_id = Some(id);
	}
	
	pub fn unselect_widget(&mut self) {
		if let Some(focused_widget_id) = self.focused_widget_id {
			self.get_widget_mut(&focused_widget_id).on_unselect();
			self.focused_widget_id = None;
		}
	}

	pub fn get_widget(&self, id: &WidgetId) -> Ref<Box<dyn Widget>> {
		self.widgets.get(id).unwrap().borrow()
	}

	pub fn get_widget_mut(&self, id: &WidgetId) -> RefMut<Box<dyn Widget>> {
		self.widgets.get(id).unwrap().borrow_mut()
	}

	
	pub fn get<T: Widget>(&self, id: &WidgetId) -> Ref<T> {
		Ref::map(
			self.widgets.get(id).unwrap().borrow(),
			|x| x.as_ref().downcast_ref::<T>().unwrap()
		)
	}

	pub fn get_mut<T: Widget>(&self, id: &WidgetId) -> RefMut<T> {
		RefMut::map(
			self.widgets.get(id).unwrap().borrow_mut(),
			|x| x.as_mut().downcast_mut::<T>().unwrap()
		)
	}

	pub fn get_cam_order(&self) -> Ref<Vec<WidgetId>> {
		self.cam_order.borrow()
	}
	/// Puts the given widget on top of the others (the widget needs to have a camera)
	pub fn put_on_top_cam(&self, id: &WidgetId) {
		let index = self.cam_order.borrow().iter().position(|i| i == id).unwrap();
		self.cam_order.borrow_mut().remove(index);
		self.cam_order.borrow_mut().push(*id);
	}
	/// Puts the given widget on top of the others (the widget needs to not have a camera)
	pub fn put_on_top_no_cam(&self, id: &WidgetId) {
		let index = self.no_cam_order.borrow().iter().position(|i| i == id).unwrap();
		self.no_cam_order.borrow_mut().remove(index);
		self.no_cam_order.borrow_mut().push(*id);
	}

	fn widget_has_no_camera(&self, id: WidgetId) -> bool {
		self.no_cam_order.borrow().contains(&id)
	}

	/// Returns the id of the last added widget
	pub fn last_id(&self) -> WidgetId {
		self.id_counter - 1
	}

	pub fn focused_widget(&self) -> Option<WidgetId> {
		self.focused_widget_id
	}
}
