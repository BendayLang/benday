use crate::input::key_state::ChadKeyState;
use nalgebra::{Point2, Vector2};
use sdl2::mouse::MouseButton;

use super::KeyState;

#[derive(Default)]
pub struct Mouse {
	pub position: Point2<i32>,
	pub delta: Vector2<i32>,
	// left_button_last_release: Instant,
	pub left_button: ChadKeyState,
	pub right_button: KeyState,
	pub middle_button: KeyState,
	pub wheel: i32,
}

impl Mouse {
	pub fn update(&mut self) {
		self.delta = Vector2::zeros();
		self.wheel = 0;
		self.left_button.update();
		self.right_button.update();
		self.middle_button.update();
	}

	pub fn get_event(&mut self, event: sdl2::event::Event) {
		use sdl2::event::Event;
		match event {
			Event::MouseMotion { x, y, xrel, yrel, .. } => {
				self.position = Point2::new(x, y);
				self.delta = Vector2::new(xrel, yrel);
			}
			Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
				MouseButton::Left => self.left_button.press(),
				MouseButton::Right => self.right_button = KeyState::Pressed,
				MouseButton::Middle => self.middle_button = KeyState::Pressed,
				MouseButton::Unknown | MouseButton::X1 | MouseButton::X2 => todo!(),
			},
			Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
				MouseButton::Left => self.left_button.release(),
				MouseButton::Right => self.right_button = KeyState::Released,
				MouseButton::Middle => self.middle_button = KeyState::Released,
				MouseButton::Unknown | MouseButton::X1 | MouseButton::X2 => todo!(),
			},
			Event::MouseWheel { y, .. } => {
				self.wheel = y;
			}
			_ => {}
		}
	}
}
