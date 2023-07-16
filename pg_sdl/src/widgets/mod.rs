pub mod button;
pub mod slider;
pub mod switch;
pub mod text_input;

use crate::camera;
use crate::camera::Camera;
use crate::color::Colors;
use crate::input::Input;
use crate::text::TextDrawer;
use as_any::{AsAny, Downcast};
pub use button::Button;
use nalgebra::{Complex, Point2, RealField, Similarity, Similarity2, Transform2, Unit, Vector2};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
pub use slider::Slider;
pub use slider::SliderType;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Add, Mul};
pub use text_input::{TextInput, TextInputStyle};

const HOVER: f32 = 0.92;
const PUSH: f32 = 0.80;


pub enum Orientation {
	Horizontal,
	Vertical,
}

type WidgetId = u32;

/// A widget is a UI object that can be interacted with to take inputs from the user.
pub trait Widget: AsAny {
	/// Update the widget based on the inputs
	fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &TextDrawer, camera: &Camera) -> bool;
	/// Draw the widget on the canvas
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, focused: bool, hovered: bool);
	/// Returns if a point collides with the widget
	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool;
}

pub struct WidgetsManager {
	id_counter: WidgetId,
	widgets: HashMap<WidgetId, Box<dyn Widget>>,
	focused_widget: Option<WidgetId>,
	hovered_widget: Option<WidgetId>,
}

impl WidgetsManager {
	pub fn new() -> Self {
		WidgetsManager { id_counter: 0, widgets: HashMap::new(), focused_widget: None, hovered_widget: None }
	}
	
	pub fn add(&mut self, widget: Box<dyn Widget>) {
		self.widgets.insert(self.id_counter, widget);
		self.id_counter += 1;
	}
	
	pub fn get<T: Widget>(&self, id: WidgetId) -> Option<&T> {
		self.widgets.get(&id).and_then(|w| w.as_ref().downcast_ref::<T>())
	}
	
	pub fn get_mut<T: Widget>(&mut self, id: WidgetId) -> Option<&mut T> {
		self.widgets.get_mut(&id).and_then(|w| w.as_mut().downcast_mut::<T>())
	}
	
	pub fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;
		
		// Update witch widget is focused (Mouse click)
		if input.mouse.left_button.is_pressed() {
			self.focused_widget = if let Some(name) = &self.hovered_widget { Some(name.clone()) } else { None };
			changed = true;
		}
		else if input.keys_state.escape.is_pressed() && self.focused_widget.is_some() {
			self.focused_widget = None;
			changed = true;
		}
		else if input.keys_state.tab.is_pressed() {
			if let Some(focused_widget) = &self.focused_widget {
				self.focused_widget = if input.keys_state.lshift.is_down() || input.keys_state.rshift.is_down() {
					Some(if focused_widget == &0 { self.id_counter - 1 } else {focused_widget - 1})
				}
				else { Some(if focused_widget == &(self.id_counter - 1) { 0 } else {focused_widget + 1}) };
				changed = true;
			}
		}
		
		// Update witch widget is hovered (Mouse movement)
		if !input.mouse.delta.is_empty() {
			let mut new_hovered_widget = None;
			let mouse_position = input.mouse.position.cast();
			for (name, widget) in &self.widgets {
				if widget.collide_point(mouse_position, camera) {
					new_hovered_widget = Some(name.clone());
					break;
				}
			}
			
			if new_hovered_widget != self.hovered_widget {
				self.hovered_widget = new_hovered_widget;
				changed = true;
			}
		}
		// Update the focused widget
		if let Some(focused_widget) = &self.focused_widget {
			changed |= self.widgets.get_mut(focused_widget).unwrap().update(input, delta_sec, text_drawer, camera);
		}
		
		changed
	}
	
	pub fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera) {
		(0..self.id_counter).for_each(|id| {
			let focused = Some(id) == self.focused_widget;
			let hovered = Some(id) == self.hovered_widget;
			self.widgets.get(&id).unwrap().draw(canvas, text_drawer, camera, focused, hovered);
		});
	}
	
	pub fn is_widget_focused(&self) -> bool { self.focused_widget.is_some() }
	
	// TODO: remove this and replace with a macro that right all the code for us
	// and for every widget type
	
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
}
