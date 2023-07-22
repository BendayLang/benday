use crate::camera::Camera;
use crate::color::Colors;
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::{draw_text, fill_rounded_rect};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::WidgetsManager;
use nalgebra::{Point2, Vector2};
use sdl2::mouse::MouseUtil;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use std::time::{Duration, Instant};

pub trait App {
	fn update(&mut self, delta: Duration, input: &Input, widgets_manager: &mut WidgetsManager, camera: &mut Camera) -> bool;
	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, widgets_manager: &WidgetsManager, camera: &Camera);
}

pub struct PgSdl<'ttf, 'texture> {
	mouse: MouseUtil,
	input: Input,
	canvas: Canvas<Window>,
	pub text_drawer: TextDrawer<'ttf, 'texture>,
	background_color: Color,
	widgets_manager: WidgetsManager,
	fps: Option<u32>,
	draw_fps: bool,
	camera: Camera,
}

impl<'ttf, 'texture> PgSdl<'ttf, 'texture> {
	pub fn init(
		window_title: &str, window_size: Vector2<u32>, fps: Option<u32>, draw_fps: bool, background_color: Color,
		widgets_manager: WidgetsManager,
	) -> Self {
		let sdl_context = sdl2::init().expect("SDL2 could not be initialized");

		let video_subsystem = sdl_context.video().expect("SDL video subsystem could not be initialized");

		video_subsystem.text_input().start();

		let window = video_subsystem
			.window(window_title, window_size.x, window_size.y)
			.position_centered()
			.resizable()
			.build()
			.expect("Window could not be created");

		let canvas = window.into_canvas().build().expect("Canvas could not be created");

		// TODO mettre ca en paramettre ?
		let camera = Camera::new(window_size, 6, 2.5, 5.0, -4000.0, 4000.0, -5000.0, 5000.0);

		let text_drawer = TextDrawer::new(canvas.texture_creator());

		PgSdl {
			mouse: sdl_context.mouse(),
			text_drawer,
			input: Input::new(sdl_context.event_pump().unwrap(), video_subsystem.clipboard()),
			widgets_manager,
			canvas,
			background_color,
			fps,
			draw_fps,
			camera,
		}
	}

	fn update<U>(&mut self, user_app: &mut U, delta: Duration) -> bool
	where
		U: App,
	{
		let mut change = false;
		if let Some(new_resolution) = self.input.window_resized {
			self.camera.resize(new_resolution);
			change = true;
		}
		change |= self.widgets_manager.update(&self.input, delta, &mut self.text_drawer, &self.camera);
		change |= user_app.update(delta, &self.input, &mut self.widgets_manager, &mut self.camera);
		change
	}

	fn draw<U>(&mut self, user_app: &U)
	where
		U: App,
	{
		self.canvas.set_draw_color(self.background_color);
		self.canvas.clear();
		user_app.draw(&mut self.canvas, &mut self.text_drawer, &self.widgets_manager, &self.camera);
	}

	fn draw_fps(&mut self, delta: Duration) {
		fill_rounded_rect(&mut self.canvas, None, Colors::WHITE, Rect::new(10.0, 2.0, 120.0, 32.0), 5.0);
		// self.text_drawer.draw_text(
		draw_text(
			&mut self.canvas,
			None,
			&mut self.text_drawer,
			Point2::new(65., 17.),
			&format!("FPS: {0:.0}", 1.0 / delta.as_secs_f32()),
			24.0,
			&TextStyle::default(),
			Align::Center,
		);
	}

	pub fn run<U>(&mut self, user_app: &mut U)
	where
		U: App,
	{
		let mut frame_instant: Instant;
		let mut frame_time = Duration::ZERO;

		self.input.get_events(); // permet au draw de savoir ou placer les widgets la première fois
		self.draw(user_app);

		loop {
			// Time control
			frame_instant = Instant::now();

			self.input.get_events();
			if self.input.window_closed {
				break;
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

			frame_time = frame_instant.elapsed();
		}
	}
}
