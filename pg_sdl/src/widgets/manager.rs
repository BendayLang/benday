use crate::camera::Camera;
use crate::custom_rect::Rect;
use crate::input::{Input, KeyState};
use crate::text::TextDrawer;
use crate::widgets::{Widget, WidgetId};
use as_any::{AsAny, Downcast};
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::any::TypeId;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Clone)]
pub enum Command {
	UnfocusWidget,
	PutOnTopCam { id: WidgetId },
	PutOnTopNoCam { id: WidgetId },
}

#[derive(Default)]
pub struct Manager {
	widgets: HashMap<WidgetId, RefCell<Box<dyn Widget>>>,
	no_cam_order: Vec<WidgetId>,
	cam_order: Vec<WidgetId>,
	id_counter: WidgetId,
	focused_widget_id: Option<WidgetId>,
	hovered_widget_id: Option<WidgetId>,
	commands: Vec<Command>,
}

impl Manager {
	pub fn update(&mut self, input: &Input, delta: Duration, text_drawer: &mut TextDrawer, camera: &mut Camera) -> bool {
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
			self.update_focused_widget(new_focused_widget_id);
			changed = true;
		}

		// Update witch widget is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() && !input.mouse.left_button.is_down() {
			let mut new_hovered_widget = None;
			let mouse_position = input.mouse.position.cast();
			// checks collisions with the widgets without camera first
			for id in self.no_cam_order.iter().rev() {
				let widget = self.get_widget(id);
				if widget.is_visible() && widget.collide_point(mouse_position) {
					new_hovered_widget = Some(*id);
					break;
				}
			}
			// checks collisions with the widgets with camera if none without was hovered
			if new_hovered_widget.is_none() {
				let mouse_position = camera.transform().inverse() * mouse_position;
				for id in self.cam_order.iter().rev() {
					let widget = self.get_widget(id);
					if widget.is_visible() && widget.collide_point(mouse_position) {
						new_hovered_widget = Some(*id);
						break;
					}
				}
			}
			if new_hovered_widget != self.hovered_widget_id {
				self.update_hovered_widget(new_hovered_widget);
				changed = true;
			}
		}

		// Update the focused widget (if there is one)
		if let Some(focused_widget_id) = self.focused_widget_id {
			if !(self.hovered_widget_id == Some(focused_widget_id) && self.get_widget(&focused_widget_id).get_base().has_scroll) {
				changed |= camera.update(input, true);
			}

			let camera = if self.widget_has_no_camera(&focused_widget_id) { None } else { Some(&*camera) };
			self.commands.clear();

			let widget = self.widgets.remove(&focused_widget_id).unwrap();
			changed |= widget.borrow_mut().update(input, delta, self, text_drawer, camera);
			self.widgets.insert(focused_widget_id, widget);

			self.commands.clone().iter().for_each(|command| match command {
				Command::UnfocusWidget => {
					if let Some(focused_widget_id) = self.focused_widget_id {
						let widget = self.widgets.remove(&focused_widget_id).unwrap();
						widget.borrow_mut().on_unfocus(self);
						self.widgets.insert(focused_widget_id, widget);
						self.get_widget_mut(&focused_widget_id).get_base_mut().focused = false;
						self.focused_widget_id = None;
					}
				}
				Command::PutOnTopCam { id } => self.put_on_top_cam(id),
				Command::PutOnTopNoCam { id } => self.put_on_top_no_cam(id),
			});
		} else {
			changed |= camera.update(input, false);
		}

		changed
	}

	pub fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, camera: &Camera) {
		// draws the widgets with camera first
		self.cam_order.iter().for_each(|id| {
			let widget = self.get_widget(id);
			if widget.is_visible() {
				widget.draw(canvas, text_drawer, Some(camera))
			}
		});
		// draws the widgets without camera on top
		self.no_cam_order.iter().for_each(|id| {
			let widget = self.get_widget(id);
			if widget.is_visible() {
				widget.draw(canvas, text_drawer, None)
			}
		});
	}

	/// Adds the given widget to the widgets manager and returns it's id
	pub fn add_widget(&mut self, widget: Box<dyn Widget>, has_camera: bool) -> WidgetId {
		let id = self.id_counter;
		self.widgets.insert(id, RefCell::new(widget));
		self.get_widget_mut(&id).get_base_mut().set_id(id);
		if has_camera {
			self.cam_order.push(id)
		} else {
			self.no_cam_order.push(id)
		}
		self.id_counter += 1;
		id
	}

	pub fn focus_widget(&mut self, id: WidgetId) {
		self.update_focused_widget(Some(id));
	}

	pub fn remove_widget(&mut self, id: &WidgetId) {
		if self.widget_has_no_camera(id) {
			let index = self.no_cam_order.iter().position(|i| i == id).unwrap();
			self.no_cam_order.remove(index);
		} else {
			let index = self.cam_order.iter().position(|i| i == id).unwrap();
			self.cam_order.remove(index);
		}
		self.widgets.remove(id);
		if self.focused_widget_id == Some(*id) {
			self.focused_widget_id = None;
		}
		if self.hovered_widget_id == Some(*id) {
			self.hovered_widget_id = None;
		}
	}

	fn update_focused_widget(&mut self, new_focused_widget_id: Option<WidgetId>) {
		if let Some(old_focused_widget_id) = self.focused_widget_id {
			let unfocus_old_widget = |manager: &mut Self| {
				let widget = manager.widgets.remove(&old_focused_widget_id).unwrap();
				widget.borrow_mut().on_unfocus(manager);
				manager.widgets.insert(old_focused_widget_id, widget);
				manager.get_widget_mut(&old_focused_widget_id).get_base_mut().focused = false;
			};
			let unfocus_parent = |manager: &mut Self| {
				let option_parent_id = manager.get_widget(&old_focused_widget_id).get_base().parent_id.clone();
				if let Some(parent_id) = option_parent_id {
					let widget = manager.widgets.remove(&parent_id).unwrap();
					widget.borrow_mut().on_unfocus(manager);
					manager.widgets.insert(parent_id, widget);
					manager.get_widget_mut(&parent_id).get_base_mut().focused = false;
				}
			};

			if let Some(new_focused_widget_id) = new_focused_widget_id {
				if self.get_widget(&new_focused_widget_id).get_base().parent_id == Some(old_focused_widget_id) {
				} else if self.get_widget(&new_focused_widget_id).get_base().parent_id
					== self.get_widget(&old_focused_widget_id).get_base().parent_id
				{
					unfocus_old_widget(self);
				} else {
					unfocus_old_widget(self);
					unfocus_parent(self);
				}
			} else {
				unfocus_old_widget(self);
				unfocus_parent(self);
			}
			self.focused_widget_id = None;
		}

		if let Some(new_focused_widget_id) = new_focused_widget_id {
			let widget = self.widgets.remove(&new_focused_widget_id).unwrap();
			widget.borrow_mut().on_focus(self);
			self.widgets.insert(new_focused_widget_id, widget);
			self.get_widget_mut(&new_focused_widget_id).get_base_mut().focused = true;
			self.focused_widget_id = Some(new_focused_widget_id);

			let option_parent_id = self.get_widget(&new_focused_widget_id).get_base().parent_id.clone();
			if let Some(parent_id) = option_parent_id {
				let widget = self.widgets.remove(&parent_id).unwrap();
				widget.borrow_mut().on_focus(self);
				self.widgets.insert(parent_id, widget);
				self.get_widget_mut(&parent_id).get_base_mut().focused = true;
			}
		}
	}

	fn update_hovered_widget(&mut self, new_hovered_widget_id: Option<WidgetId>) {
		if let Some(old_hovered_widget_id) = self.hovered_widget_id {
			let unhover_old_widget = |manager: &mut Self| {
				let widget = manager.widgets.remove(&old_hovered_widget_id).unwrap();
				widget.borrow_mut().on_unhover(manager);
				manager.widgets.insert(old_hovered_widget_id, widget);
				manager.get_widget_mut(&old_hovered_widget_id).get_base_mut().hovered = false;
			};
			let unhover_parent = |manager: &mut Self| {
				let option_parent_id = manager.get_widget(&old_hovered_widget_id).get_base().parent_id.clone();
				if let Some(parent_id) = option_parent_id {
					let widget = manager.widgets.remove(&parent_id).unwrap();
					widget.borrow_mut().on_unhover(manager);
					manager.widgets.insert(parent_id, widget);
					manager.get_widget_mut(&parent_id).get_base_mut().hovered = false;
				}
			};

			if let Some(new_hovered_widget_id) = new_hovered_widget_id {
				if self.get_widget(&new_hovered_widget_id).get_base().parent_id == Some(old_hovered_widget_id) {
				} else if self.get_widget(&new_hovered_widget_id).get_base().parent_id
					== self.get_widget(&old_hovered_widget_id).get_base().parent_id
				{
					unhover_old_widget(self);
				} else {
					unhover_old_widget(self);
					unhover_parent(self);
				}
			} else {
				unhover_old_widget(self);
				unhover_parent(self);
			}
			self.hovered_widget_id = None;
		}

		if let Some(new_hovered_widget_id) = new_hovered_widget_id {
			let widget = self.widgets.remove(&new_hovered_widget_id).unwrap();
			widget.borrow_mut().on_hover(self);
			self.widgets.insert(new_hovered_widget_id, widget);
			self.get_widget_mut(&new_hovered_widget_id).get_base_mut().hovered = true;
			self.hovered_widget_id = Some(new_hovered_widget_id);

			let option_parent_id = self.get_widget(&new_hovered_widget_id).get_base().parent_id.clone();
			if let Some(parent_id) = option_parent_id {
				let widget = self.widgets.remove(&parent_id).unwrap();
				widget.borrow_mut().on_hover(self);
				self.widgets.insert(parent_id, widget);
				self.get_widget_mut(&parent_id).get_base_mut().hovered = true;
			}
		}
	}

	pub fn push_command(&mut self, command: Command) {
		self.commands.push(command);
	}

	pub fn get_widget(&self, id: &WidgetId) -> Ref<Box<dyn Widget>> {
		self.widgets.get(id).expect(&format!("The widget with id {id} does not exist")).borrow()
	}

	pub fn get_widget_mut(&self, id: &WidgetId) -> RefMut<Box<dyn Widget>> {
		self.widgets.get(id).expect(&format!("The widget with id {id} does not exist")).borrow_mut()
	}

	pub fn get<T: Widget>(&self, id: &WidgetId) -> Ref<T> {
		Ref::map(self.widgets.get(id).expect(&format!("The widget with id {id} does not exist")).borrow(), |x| {
			x.as_ref().downcast_ref::<T>().expect("Incorrect Widget type")
		})
	}

	pub fn get_mut<T: Widget>(&self, id: &WidgetId) -> RefMut<T> {
		RefMut::map(self.widgets.get(id).expect(&format!("The widget with id {id} does not exist")).borrow_mut(), |x| {
			x.as_mut().downcast_mut::<T>().expect("Incorrect Widget type")
		})
	}

	pub fn get_cam_order(&self) -> &Vec<WidgetId> {
		&self.cam_order
	}
	/// Puts the given widget on top of the others (the widget needs to have a camera)
	pub fn put_on_top_cam(&mut self, id: &WidgetId) {
		let index = self.cam_order.iter().position(|i| i == id).unwrap();
		self.cam_order.remove(index);
		self.cam_order.push(*id);
	}
	/// Puts the given widget on top of the others (the widget needs to not have a camera)
	fn put_on_top_no_cam(&mut self, id: &WidgetId) {
		let index = self.no_cam_order.iter().position(|i| i == id).unwrap();
		self.no_cam_order.remove(index);
		self.no_cam_order.push(*id);
	}

	fn widget_has_no_camera(&self, id: &WidgetId) -> bool {
		self.no_cam_order.contains(id)
	}

	/// Returns the id of the last added widget
	pub fn last_id(&self) -> WidgetId {
		self.id_counter - 1
	}

	pub fn focused_widget(&self) -> Option<WidgetId> {
		self.focused_widget_id
	}
}
