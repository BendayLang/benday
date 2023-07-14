pub mod button;
pub mod slider;
pub mod switch;
pub mod text_box;
use crate::camera;
use crate::camera::Camera;
use crate::color::Colors;
use crate::input::Input;
use crate::text::TextDrawer;
use as_any::{AsAny, Downcast};
pub use button::Button;
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
pub use slider::Slider;
pub use slider::SliderType;
use std::collections::HashMap;
pub use text_box::{TextBox, TextBoxStyle};

const HOVER: f32 = 0.94;
const PUSH: f32 = 0.80;

pub enum Orientation {
	Horizontal,
	Vertical,
}

const SELECTED_COLOR: Color = Colors::LIGHT_BLUE;

/// A widget is a UI object that can be interacted with to take inputs from the user.
pub trait Widget: AsAny {
	/// Update the widget based on the inputs
	fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &TextDrawer, camera: &Camera) -> bool;
	/// Draw the widget on the canvas
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, selected: bool, hovered: bool);
	/// Returns if a point collides with the widget
	fn collide_point(&self, point: Point2<f64>, camera: &Camera) -> bool;
}

pub struct WidgetsManager {
	widgets: HashMap<String, Box<dyn Widget>>,
	selected_widget: Option<String>,
	hovered_widget: Option<String>,
}

impl WidgetsManager {
	pub fn new() -> Self {
		WidgetsManager { widgets: HashMap::new(), selected_widget: None, hovered_widget: None }
	}

	pub fn add(&mut self, name: &str, widget: Box<dyn Widget>) {
		self.widgets.insert(name.to_string(), widget);
	}

	pub fn get<T: Widget>(&self, name: &str) -> Option<&T> {
		self.widgets.get(name).and_then(|w| w.as_ref().downcast_ref::<T>())
	}

	pub fn get_mut<T: Widget>(&mut self, name: &str) -> Option<&mut T> {
		self.widgets.get_mut(name).and_then(|w| w.as_mut().downcast_mut::<T>())
	}

	pub fn update(&mut self, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera) -> bool {
		let mut changed = false;

		// Update witch widget is selected (Mouse click)
		if input.mouse.left_button.is_pressed() {
			self.selected_widget = if let Some(name) = &self.hovered_widget { Some(name.clone()) } else { None };
			changed = true;
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
		// Update the selected widget
		if let Some(selected_widget) = &self.selected_widget {
			changed |= self.widgets.get_mut(selected_widget).unwrap().update(input, delta_sec, text_drawer, camera);
		}

		changed
	}

	pub fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera) {
		for (name, widget) in &self.widgets {
			let selected = Some(name.clone()) == self.selected_widget;
			let hovered = Some(name.clone()) == self.hovered_widget;
			widget.draw(canvas, text_drawer, camera, selected, hovered);
		}
	}

	pub fn is_widget_selected(&self) -> bool {
		self.selected_widget.is_some()
	}

	// TODO: remove this and replace with a macro that right all the code for us
	// and for every widget type

	pub fn get_mut_button(&mut self, name: &str) -> &mut Button {
		if let Some(button) = self.get_mut(name) {
			button
		} else {
			panic!("Button '{}' not found", name);
		}
	}

	pub fn get_button(&self, name: &str) -> &Button {
		if let Some(button) = self.get(name) {
			button
		} else {
			panic!("Button '{}' not found", name);
		}
	}

	pub fn get_mut_slider(&mut self, name: &str) -> &mut Slider {
		if let Some(slider) = self.get_mut(name) {
			slider
		} else {
			panic!("Slider '{}' not found", name);
		}
	}

	pub fn get_slider(&self, name: &str) -> &Slider {
		if let Some(slider) = self.get(name) {
			slider
		} else {
			panic!("Slider '{}' not found", name);
		}
	}
}
