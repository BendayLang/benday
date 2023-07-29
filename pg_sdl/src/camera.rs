use crate::color::darker;
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::draw_text;
use crate::style::{Align, HAlign, VAlign};
use crate::text::{TextDrawer, TextStyle};
use nalgebra::{Point2, Similarity2, Translation2, Vector2};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

pub struct Camera {
	resolution: Vector2<u32>,
	transform: Similarity2<f64>,
	grab_delta: Option<Vector2<f64>>,
	scaling_factor: f64,
	min_scale: f64,
	max_scale: f64,
	top_limit: f64,
	bottom_limit: f64,
	left_limit: f64,
	right_limit: f64,
}

impl Camera {
	pub fn new(
		resolution: Vector2<u32>, doubling_steps: u8, zoom_in_limit: f64, zoom_out_limit: f64, top_limit: f64, bottom_limit: f64,
		left_limit: f64, right_limit: f64,
	) -> Self {
		Camera {
			resolution,
			transform: Similarity2::new(resolution.cast() * 0.5, 0.0, 1.0),
			grab_delta: None,
			scaling_factor: f64::powf(2.0, 1.0 / doubling_steps as f64),
			min_scale: 1.0 / zoom_out_limit,
			max_scale: zoom_in_limit,
			top_limit,
			bottom_limit,
			left_limit,
			right_limit,
		}
	}

	pub fn transform(&self) -> Similarity2<f64> {
		self.transform
	}
	pub fn scale(&self) -> f64 {
		self.transform.scaling()
	}

	fn translation(&self) -> Translation2<f64> {
		self.transform.isometry.translation
	}
	fn translation_mut(&mut self) -> &mut Translation2<f64> {
		&mut self.transform.isometry.translation
	}

	pub fn is_in_scope(&self, rect: Rect) -> bool {
		let camera_scope = Rect::from_origin(self.resolution.cast());
		camera_scope.collide_rect(rect)
	}

	/// Translates and scales the camera from the inputs
	pub fn update(&mut self, input: &Input, lock_translation: bool) -> bool {
		let mut changed = false;

		if input.mouse.left_button.is_pressed() {
			self.grab_delta = Some(self.translation().vector - input.mouse.position.coords.cast());
		} else if input.mouse.left_button.is_released() {
			self.grab_delta = None;
		} else if let Some(grab_delta) = self.grab_delta {
			if !lock_translation {
				let mouse_position = input.mouse.position.coords.cast();
				changed |= self.translate(mouse_position + grab_delta - self.translation().vector, Some(mouse_position));
			}
		}

		let scaling = self.scaling_factor.powf(input.mouse.wheel as f64);
		let center = input.mouse.position.coords.cast();
		changed |= self.change_scale(scaling, center);

		changed
	}

	/// Translates the camera while restricting it within it limits.
	fn translate(&mut self, delta: Vector2<f64>, mouse_position: Option<Vector2<f64>>) -> bool {
		let old_translation = self.translation();
		self.transform.append_translation_mut(&Translation2::from(delta));

		let start = self.transform.inverse() * Point2::origin(); // Top Left
		let end = self.transform.inverse() * Point2::from(self.resolution.cast()); // Bottom Right

		if start.x < self.left_limit {
			self.translation_mut().x = -self.left_limit * self.scale();
			if let Some(mouse_position) = mouse_position {
				self.grab_delta = Some(self.translation().vector - mouse_position);
			}
		}
		if start.y < self.top_limit {
			self.translation_mut().y = -self.top_limit * self.scale();
			if let Some(mouse_position) = mouse_position {
				self.grab_delta = Some(self.translation().vector - mouse_position);
			}
		}
		if end.x > self.right_limit {
			self.translation_mut().x = -self.right_limit * self.scale() + self.resolution.x as f64;
			if let Some(mouse_position) = mouse_position {
				self.grab_delta = Some(self.translation().vector - mouse_position);
			}
		}
		if end.y > self.bottom_limit {
			self.translation_mut().y = -self.bottom_limit * self.scale() + self.resolution.y as f64;
			if let Some(mouse_position) = mouse_position {
				self.grab_delta = Some(self.translation().vector - mouse_position);
			}
		}
		self.translation() != old_translation
	}

	/// Scales the camera by 'scaling' while restricting it within it limits.
	fn change_scale(&mut self, scaling: f64, center: Vector2<f64>) -> bool {
		if scaling == 1.0 {
			return false;
		}
		if self.min_scale > self.scale() * scaling {
			if self.scale() <= self.min_scale {
				return false;
			}
			let adjusted_scaling = self.min_scale / self.scale();
			self.transform.append_scaling_mut(adjusted_scaling);
			self.translate((1.0 - adjusted_scaling) * center, None);
			true
		} else if self.max_scale < self.scale() * scaling {
			if self.scale() >= self.max_scale {
				return false;
			}
			let adjusted_scaling = self.max_scale / self.scale();
			self.transform.append_scaling_mut(adjusted_scaling);
			self.translate((1.0 - adjusted_scaling) * center, None);
			true
		} else {
			self.transform.append_scaling_mut(scaling);
			self.translate((1.0 - scaling) * center, None);
			true
		}
	}

	pub fn resize(&mut self, new_resolution: Vector2<u32>) {
		self.translate((new_resolution.cast() - self.resolution.cast()) * 0.5, None);
		self.resolution = new_resolution;
	}

	/// Draws a grid
	pub fn draw_grid(
		&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, color: Color, axes: bool, graduations: bool,
	) {
		let max_depth = 2;

		let p = (self.scale().log(5.0) + 1.4).floor();
		let global_scale = 5_f64.powf(p) / 100.0;
		let global_unit = |depth: i16| 5_f64.powf(depth as f64 - p) * 100.0;

		// Alignment
		let origin = self.transform * Point2::origin();
		let v_align = if origin.y.is_sign_negative() {
			VAlign::Top
		} else if (origin.y as u32).lt(&self.resolution.y) {
			VAlign::Center
		} else {
			VAlign::Bottom
		};
		let h_align = if origin.x.is_sign_negative() {
			HAlign::Left
		} else if (origin.x as u32).lt(&self.resolution.x) {
			HAlign::Center
		} else {
			HAlign::Right
		};
		let alignment = Align::from_align(
			if h_align == HAlign::Center { HAlign::Left } else { h_align },
			if v_align == VAlign::Center { VAlign::Top } else { v_align },
		);

		let x_transform = |x_th: i32, scale: f64| (self.scale() / scale * x_th as f64 + self.translation().x) as i16;
		let y_transform = |y_th: i32, scale: f64| (self.scale() / scale * y_th as f64 + self.translation().y) as i16;

		// Grid
		(0..=max_depth).for_each(|depth| {
			let line_color = darker(
				color,
				match depth {
					0 => 0.96,
					1 => 0.88,
					_ => 0.80,
				},
			);
			let scale = global_scale * 5_f64.powf(-depth as f64);
			let transform = self.transform.inverse().append_scaling(scale);

			let start = (transform * Point2::origin()).map(|v| v.ceil() as i32); // Top Left
			let end = (transform * Point2::from(self.resolution.cast())).map(|v| v.ceil() as i32); // Bottom Right

			(start.x..end.x).for_each(|x_th| {
				if (x_th % 5 != 0) | (depth == max_depth) {
					DrawRenderer::vline(canvas, x_transform(x_th, scale), 0, self.resolution.y as i16 - 1, line_color).unwrap();
				}
			});
			(start.y..end.y).for_each(|y_th| {
				if (y_th % 5 != 0) | (depth == max_depth) {
					DrawRenderer::hline(canvas, 0, self.resolution.x as i16 - 1, y_transform(y_th, scale), line_color).unwrap();
				}
			});
		});

		let axes_color = darker(color, 0.3);

		if axes {
			let x = match h_align {
				HAlign::Left => 0,
				HAlign::Center => origin.x as u32,
				HAlign::Right => self.resolution.x - 1,
			};
			let y = match v_align {
				VAlign::Top => 0,
				VAlign::Center => origin.y as u32,
				VAlign::Bottom => self.resolution.y - 1,
			};
			DrawRenderer::vline(canvas, x as i16, 0, self.resolution.y as i16 - 1, axes_color).unwrap();
			DrawRenderer::hline(canvas, 0, self.resolution.x as i16 - 1, y as i16, axes_color).unwrap();
		}

		if graduations {
			(1..=max_depth).for_each(|depth| {
				let scale = global_scale * 5_f64.powf(-depth as f64);
				let unit = global_unit(depth);

				let transform = self.transform.inverse().append_scaling(scale);

				let start = (transform * Point2::origin()).map(|v| v.ceil() as i32); // Top Left
				let end = (transform * Point2::from(self.resolution.cast())).map(|v| v.ceil() as i32); // Bottom Right

				let n = 8 * depth;
				let (x1, x2) = match h_align {
					HAlign::Left => (-n, n),
					HAlign::Center => (origin.x as i16 - n, origin.x as i16 + n),
					HAlign::Right => (self.resolution.x as i16 - 1 - n, self.resolution.x as i16 - 1 + n),
				};
				let (y1, y2) = match v_align {
					VAlign::Top => (-n, n),
					VAlign::Center => (origin.y as i16 - n, origin.y as i16 + n),
					VAlign::Bottom => (self.resolution.y as i16 - 1 - n, self.resolution.y as i16 - 1 + n),
				};

				let font_size = 16.;
				let font_style = if depth == 1 { FontStyle::NORMAL } else { FontStyle::BOLD };
				let text_style = TextStyle::new(None, axes_color, font_style);

				(start.x..end.x).for_each(|x_th| {
					if (x_th % 5 != 0) | (depth == max_depth) {
						let x = x_transform(x_th, scale);
						DrawRenderer::vline(canvas, x, y1, y2, axes_color).unwrap();

						let position = Point2::new(x as f64, (y1 as f64 + y2 as f64) / 2.);
						let text = &format!("{}", x_th as f64 * unit);
						draw_text(canvas, Some(self), text_drawer, position, text, font_size, &text_style, alignment);
					}
				});
				(start.y..end.y).for_each(|y_th| {
					if (y_th % 5 != 0) | (depth == max_depth) {
						let y = y_transform(y_th, scale);
						DrawRenderer::hline(canvas, x1, x2, y, axes_color).unwrap();

						let position = Point2::new((x1 as f64 + x2 as f64) / 2., y as f64);
						let text = &format!("{}", y_th as f64 * unit);
						draw_text(canvas, Some(self), text_drawer, position, text, font_size, &text_style, alignment);
					}
				});
			});
		}
	}
}
