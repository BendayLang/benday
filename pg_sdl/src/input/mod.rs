mod key_state;
mod mouse;

pub use key_state::{KeyState, KeysState, Shortcut};
use nalgebra::Vector2;
use sdl2::clipboard::ClipboardUtil;
use sdl2::event::WindowEvent;
use sdl2::EventPump;

pub struct Input {
	event_pump: sdl2::EventPump,
	pub window_closed: bool,
	pub keys_state: KeysState,
	pub mouse: mouse::Mouse,
	pub last_char: Option<char>,
	pub clipboard: ClipboardUtil,
	pub window_resized: Option<Vector2<u32>>,
}

impl Input {
	pub fn new(event_pump: EventPump, clipboard: ClipboardUtil) -> Self {
		Self {
			event_pump,
			window_closed: false,
			keys_state: KeysState::default(),
			mouse: mouse::Mouse::default(),
			last_char: None,
			clipboard,
			window_resized: None,
		}
	}

	/// should be called every frame
	pub fn get_events(&mut self) {
		self.last_char = None;
		self.window_resized = None;

		for key_state in self.keys_state.as_mut_array() {
			key_state.update()
		}

		self.mouse.update();

		for event in self.event_pump.poll_iter() {
			use sdl2::event::Event;
			self.mouse.get_event(event.clone());

			match event {
				Event::TextEditing { text, .. } => {
					println!("TextEditing {:?}", text);
				}
				Event::TextInput { text, .. } => {
					if text.chars().count() == 1 {
						self.last_char = text.chars().next();
					} else {
						panic!("TextInput event with more than one char {:?}", text);
					}
				}
				Event::Quit { .. } => self.window_closed = true,
				Event::KeyDown { keycode: Some(keycode), .. } => {
					self.keys_state.press_key(keycode);
				}
				Event::KeyUp { keycode: Some(keycode), .. } => {
					self.keys_state.release_key(keycode);
				}
				Event::Window { win_event: WindowEvent::SizeChanged(width, height), .. } => {
					self.window_resized = Some(Vector2::new(width as u32, height as u32));
				}
				_ => {}
			}
		}
	}

	pub fn shortcut_pressed(&self, shortcut: &Shortcut) -> bool {
		self.keys_state.shortcut_pressed(shortcut)
	}
}
