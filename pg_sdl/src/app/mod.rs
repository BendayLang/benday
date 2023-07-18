use crate::camera::Camera;
use crate::color::Colors;
use crate::input::Input;
use crate::primitives::{draw_text, fill_rounded_rect};
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::widgets::{Widget, WidgetsManager};
use crate::custom_rect::Rect;
use nalgebra::{Point2, Vector2};
use ndarray::AssignElem;
use sdl2::mouse::{Cursor, MouseUtil, SystemCursor};
use sdl2::ttf::FontStyle;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use std::collections::HashMap;
use std::time::{Duration, Instant};


pub trait App {
	fn update(&mut self, delta_sec: f64, input: &Input, widgets_manager: &mut WidgetsManager, camera: &mut Camera) -> bool;
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
	pub fn init(window_title: &str, window_size: Vector2<u32>, fps: Option<u32>, draw_fps: bool,
	            background_color: Color, widgets_manager: WidgetsManager) -> Self {
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
		
		PgSdl {
			mouse: sdl_context.mouse(),
			text_drawer: TextDrawer::new(canvas.texture_creator()),
			input: Input::new(sdl_context, video_subsystem.clipboard()),
			widgets_manager,
			canvas,
			background_color,
			fps,
			draw_fps,
			camera,
		}
	}
	
	fn update<U>(&mut self, user_app: &mut U, delta_sec: f64) -> bool where U: App, {
		let mut change = false;
		if let Some(new_resolution) = self.input.window_resized {
			self.camera.resize(new_resolution);
			change = true;
		}
		change |= self.widgets_manager.update(&self.input, delta_sec, &mut self.text_drawer, &self.camera);
		change |= user_app.update(delta_sec, &self.input, &mut self.widgets_manager, &mut self.camera);
		change
	}
	
	fn draw<U>(&mut self, user_app: &U) where U: App, {
		self.canvas.set_draw_color(self.background_color);
		self.canvas.clear();
		user_app.draw(&mut self.canvas, &mut self.text_drawer, &self.camera);
		self.widgets_manager.draw(&mut self.canvas, &self.text_drawer, &self.camera);
	}
	
	fn draw_fps(&mut self, delta_sec: f64) {
		fill_rounded_rect(&mut self.canvas, None, Colors::WHITE, Rect::new(10.0, 2.0, 120.0, 32.0), 5.0);
		draw_text(&mut self.canvas, None, &self.text_drawer, Point2::new(65., 17.), &format!("FPS: {0:.0}", 1.0 / delta_sec),
		          24.0, &TextStyle::default(), Align::Center);
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
}
