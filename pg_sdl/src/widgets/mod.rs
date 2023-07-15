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
use sdl2::rect::Rect;
pub use text_input::{TextInput, TextInputStyle};

const HOVER: f32 = 0.92;
const PUSH: f32 = 0.80;


#[derive(Clone, Debug)]
pub struct MyRect<T: Clone + PartialEq + Debug + 'static> {
	position: Point2<T>,
	size: Vector2<T>,
}

impl<T: Copy + Clone + PartialEq + Debug + Add<T, Output = T> + Mul<f64, Output = T>> MyRect<T> { // + 'static
	pub fn new(x: T, y: T, width: T, height: T) -> MyRect<T> {
		Self { position: Point2::new(x, y), size: Vector2::new(width, height) }
	}
	pub fn from(position: Point2<T>, size: Vector2<T>) -> MyRect<T> {
		Self { position, size }
	}
	
	pub fn x(&self) -> T { self.position.x }
	pub fn y(&self) -> T { self.position.y }
	pub fn width(&self) -> T { self.size.x }
	pub fn height(&self) -> T { self.size.y }
	
	pub fn top(&self) -> T { self.position.y + self.size.y }
	pub fn v_mid(&self) -> T { self.position.y + self.size.y * 0.5}
	pub fn bottom(&self) -> T { self.position.y }
	
	pub fn left(&self) -> T { self.position.x }
	pub fn h_mid(&self) -> T { self.position.x + self.size.x * 0.5 }
	pub fn right(&self) -> T { self.position.x + self.size.x }
	
	pub fn top_left(&self) -> Point2<T> { self.position + Vector2::new(self.size.x * 0., self.size.y) }
	pub fn mid_top(&self) -> Point2<T> { self.position + Vector2::new(self.size.x * 0.5, self.size.y) }
	pub fn top_right(&self) -> Point2<T> { self.position + self.size }
	pub fn mid_left(&self) -> Point2<T> { self.position + Vector2::new(0., self.size.y * 0.5) }
	pub fn center(&self) -> Point2<T> { self.position + self.size * 0.5 }
	pub fn mid_right(&self) -> Point2<T> { self.position + Vector2::new(self.size.x, self.size.y * 0.5) }
	pub fn bottom_left(&self) -> Point2<T> { self.position }
	pub fn mid_bottom(&self) -> Point2<T> { self.position + Vector2::new(self.size.x * 0.5, 0.) }
	pub fn bottom_right(&self) -> Point2<T> { self.position + Vector2::new(self.size.x, 0.) }
	
	pub fn collide_point(&self, point: Point2<T>) -> bool {
		self.left() < point.x && point.x < self.right() && self.bottom() < point.y && point.y < self.top()
	}
	pub fn collide_rect(&self, rect: &MyRect<T>) -> bool {
		self.left() < rect.right() && rect.left() < self.right() && self.bottom() < rect.top() && rect.bottom() < self.top()
	}
}

impl<T: Clone + PartialEq + Debug + 'static + RealField> Mul<Transform2<T>> for MyRect<T> {
	type Output = Self;
	
	fn mul(self, rhs: Transform2<T>) -> Self::Output {
		Self::from(rhs * self.position, rhs * self.size)
	}
}

impl<T: Clone + PartialEq + Debug + 'static + RealField> Mul<MyRect<T>> for Similarity<T, Unit<Complex<T>>, 2> {
	type Output = MyRect<T>;
	
	fn mul(self, rhs: MyRect<T>) -> Self::Output {
		MyRect::from(self * rhs.position, self * rhs.size)
	}
}

impl<T: Clone + PartialEq + Debug + 'static> Into<Rect> for MyRect<T>{
	fn into(self) -> Rect {
		Rect::new(self.x() as i32, self.y() as i32, self.width() as u32, self.height() as u32)
	}
}


pub enum Orientation {
	Horizontal,
	Vertical,
}


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
