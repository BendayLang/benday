use crate::camera::Camera;
use crate::color::Colors;
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::{draw_text, fill_rounded_rect};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::Manager;
use nalgebra::{Point2, Vector2};
use sdl2::mouse::MouseUtil;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use std::time::{Duration, Instant};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

pub trait App {
	fn update(&mut self, delta: Duration, input: &Input, manager: &mut Manager, camera: &mut Camera) -> bool;
	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, manager: &Manager, camera: &Camera);
}

pub struct PgSdl<'ttf, 'texture> {
	mouse: MouseUtil,
	pub text_drawer: TextDrawer<'ttf, 'texture>,
	input: Input,
	manager: Manager,
	window_canvas: Canvas<Window>,
	canvas_surface: Canvas<Surface<'ttf>>,
	background_color: Color,
	fps: Option<u32>,
	draw_fps: bool,
	camera: Camera,
}

impl<'ttf, 'texture> PgSdl<'ttf, 'texture> {
	pub fn init(
		window_title: &str, window_size: Vector2<u32>, fps: Option<u32>, draw_fps: bool, background_color: Color, manager: Manager,
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

		let window_canvas = window.into_canvas().build().expect("Canvas could not be created");
		let surface = Surface::new(window_size.x, window_size.y, PixelFormatEnum::RGBA32).unwrap();
		let canvas_surface = surface.into_canvas().unwrap();

		// TODO mettre ca en paramettre ?
		let camera = Camera::new(window_size, 6, 2.5, 5.0, -4000.0, 4000.0, -5000.0, 5000.0);

		let text_drawer = TextDrawer::new(canvas_surface.texture_creator());

		PgSdl {
			mouse: sdl_context.mouse(),
			text_drawer,
			input: Input::new(sdl_context.event_pump().unwrap(), video_subsystem.clipboard()),
			manager,
			window_canvas,
			canvas_surface,
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
		change |= self.manager.update(&self.input, delta, &mut self.text_drawer, &self.camera);
		change |= user_app.update(delta, &self.input, &mut self.manager, &mut self.camera);
		change
	}

	fn draw<U>(&mut self, user_app: &U, frame_time: Duration)
	where
		U: App,
	{
		self.canvas_surface.set_draw_color(self.background_color);
		self.canvas_surface.clear();
		user_app.draw(&mut self.canvas_surface, &mut self.text_drawer, &self.manager, &self.camera);
	}

	pub fn run<U>(&mut self, user_app: &mut U)
	where
		U: App,
	{
		let mut frame_instant: Instant;
		let mut frame_time = Duration::ZERO;

		self.input.get_events(); // permet au draw de savoir ou placer les widgets la première fois
		self.draw(user_app, frame_time);

		loop {
			// Time control
			frame_instant = Instant::now();

			self.input.get_events();
			if self.input.window_closed {
				break;
			}
			if let Some(new_size) = self.input.window_resized {
				// TODO ca marche pas
				self.canvas_surface.set_logical_size(new_size.x, new_size.y).unwrap();
			}

			// Update
			// Draw
			if self.update(user_app, frame_time) {
				self.draw(user_app, frame_time);
			}
			
			// fps
			if self.draw_fps {
				fill_rounded_rect(&mut self.canvas_surface, None, Colors::WHITE, Rect::new(10.0, 2.0, 120.0, 32.0), 5.0);
				draw_text(
					&mut self.canvas_surface,
					None,
					&mut self.text_drawer,
					Point2::new(65., 17.),
					&format!("FPS: {0:.0}", 1.0 / frame_time.as_secs_f32()),
					24.0,
					&TextStyle::default(),
					Align::Center,
				);
			}

			// Render to screen
			let texture_creator = self.window_canvas.texture_creator();
			let texture = texture_creator.create_texture_from_surface(self.canvas_surface.surface()).unwrap();
			let (width, height) = self.window_canvas.window().size();
			let target = Rect::from_origin(Vector2::new(width, height).cast()).into_rect();
			self.window_canvas.copy(&texture, None, Some(target)).unwrap();
			self.window_canvas.present();

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
