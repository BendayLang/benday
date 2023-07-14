use crate::widgets::{Widget, WidgetsManager};
use ndarray::AssignElem;
use sdl2::mouse::{Cursor, MouseUtil, SystemCursor};
use sdl2::ttf::FontStyle;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use nalgebra::{Point2, Vector2};
use sdl2::rect::{Point, Rect};
use crate::camera::Camera;
use crate::primitives::fill_rounded_rect;
use crate::color::Colors;
use crate::input::Input;
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};

pub trait App {
	fn update(&mut self, delta_sec: f64, input: &Input, widget_change: bool, widgets_manager: &mut WidgetsManager, camera: &mut Camera) -> bool;
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera);
}

pub struct PgSdl {
	mouse: MouseUtil,
	input: Input,
	canvas: Canvas<Window>,
	text_drawer: TextDrawer,
	background_color: Color,
	widgets_manager: WidgetsManager,
	fps: Option<u32>,
	draw_fps: bool,
	camera: Camera,
}

impl PgSdl {
	pub fn init(
		window_title: &str, window_width: u32, window_height: u32, fps: Option<u32>, draw_fps: bool, background_color: Color,
	) -> Self {
		let sdl_context = sdl2::init().expect("SDL2 could not be initialized");

		let video_subsystem = sdl_context.video().expect("SDL video subsystem could not be initialized");

		video_subsystem.text_input().start();

		let window = video_subsystem
			.window(window_title, window_width, window_height)
			.position_centered()
			.resizable()
			.build()
			.expect("Window could not be created");

		let canvas = window.into_canvas().build().expect("Canvas could not be created");

		// TODO mettre ca en paramettre ?
		let resolution = Vector2::new(window_width, window_height);
		let camera = Camera::new(resolution, 6, 2.5, 5.0, -4000.0, 4000.0, -5000.0, 5000.0);

		PgSdl {
			mouse: sdl_context.mouse(),
			text_drawer: TextDrawer::new(canvas.texture_creator()),
			input: Input::new(sdl_context, video_subsystem.clipboard()),
			widgets_manager: WidgetsManager::new(),
			canvas,
			background_color,
			fps,
			draw_fps,
			camera,
		}
	}

	fn draw_fps(&mut self, delta_sec: f64) {
		fill_rounded_rect(&mut self.canvas, None, Colors::WHITE, Point2::new(10.0, 2.0), Vector2::new(120.0, 32.0), 5.0);
		self.text_drawer.draw(
			&mut self.canvas,
			Point::new(65, 17),
			&TextStyle::new(24, None, Color::BLACK, FontStyle::NORMAL),
			&format!("FPS: {0:.0}", 1.0 / delta_sec),
			Align::Center,
		);
	}

	fn draw<U>(&mut self, user_app: &U)
	where
		U: App,
	{
		self.canvas.set_draw_color(self.background_color);
		self.canvas.clear();
		user_app.draw(&mut self.canvas, &mut self.text_drawer, &self.camera);
		self.widgets_manager.draw(&mut self.canvas, &self.text_drawer, &self.camera);
	}

	fn update<U>(&mut self, user_app: &mut U, delta_sec: f64) -> bool
	where
		U: App,
	{
		if let Some(new_resolution) = self.input.window_resized {
			self.camera.resize(new_resolution)
		}
		let widget_change = self.widgets_manager.update(&self.input, delta_sec, &mut self.text_drawer, &self.camera);
		let app_change = user_app.update(delta_sec, &self.input, widget_change, &mut self.widgets_manager, &mut self.camera);
		widget_change || app_change
	}

	pub fn run<U>(&mut self, user_app: &mut U)
	where
		U: App,
	{
		let mut frame_instant: Instant;
		let mut frame_time: f64 = 0.02;

		self.input.get_events(); // permet au draw de savoir ou placer les widgets la première fois
		self.draw(user_app);

		'running: loop {
			// Time control
			frame_instant = Instant::now();

			self.input.get_events();
			if self.input.window_closed {
				break 'running;
			}

			// Update
			// Draw
			if self.update(user_app, frame_time) {
				self.draw(user_app);
			}

			// FPS
			if self.draw_fps {
				self.draw_fps(frame_time);
			}

			// Render to screen
			self.canvas.present();

			// Sleep
			if let Some(fps) = &self.fps {
				let to_sleep = 1.0 / *fps as f64 - frame_instant.elapsed().as_secs_f64();
				if to_sleep > 0.0 {
					std::thread::sleep(Duration::from_secs_f64(to_sleep));
				}
			}

			frame_time = frame_instant.elapsed().as_secs_f64();
		}
	}

	pub fn add_widget(&mut self, name: &str, widget: Box<dyn Widget>) -> &mut Self {
		self.widgets_manager.add(name, widget);
		self
	}

	pub fn add_widgets(&mut self, widgets: HashMap<&str, Box<dyn Widget>>) {
		for (name, widget) in widgets {
			self.widgets_manager.add(name, widget);
		}
	}

	pub fn change_mouse_cursor(&mut self) {
		let cursor = Cursor::from_system(SystemCursor::WaitArrow).expect("mouse cursor loading error");
		cursor.set();
		// TODO ca marche pas
	}
}
