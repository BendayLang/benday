use std::time::Duration;

use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::{draw_rect, draw_rounded_rect, fill_rect, fill_rounded_rect};
use crate::text::TextDrawer;
use crate::widgets::manager::Command;
use crate::widgets::{Base, Manager, Widget, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH};
use nalgebra::Vector2;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;
use sdl2::video::Window;

pub struct DraggableStyle {
	color: Color,
	hovered_color: Color,
	pushed_color: Color,
	focused_color: Color,
	border_color: Color,
}

impl DraggableStyle {
	pub fn new(color: Color) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			pushed_color: darker(color, PUSH),
			focused_color: Colors::BLACK,
			border_color: darker(color, 0.5),
		}
	}
}

pub struct Draggable {
	base: Base,
	style: DraggableStyle,
	grab_delta: Option<Vector2<f64>>,
}

impl Draggable {
	const SHADOW: Vector2<f64> = Vector2::new(6.0, 8.0);

	pub fn new(rect: Rect, corner_radius: Option<f64>, style: DraggableStyle) -> Self {
		Self { base: Base::new(rect, corner_radius, false), style, grab_delta: None }
	}
}

impl Widget for Draggable {
	fn update(
		&mut self, input: &Input, _delta: Duration, manager: &mut Manager, _: &mut TextDrawer, camera: Option<&Camera>,
	) -> bool {
		let mut changed = self.base.update(input, Vec::new());

		if self.base.state.is_pressed() {
			if camera.is_some() {
				manager.push_command(Command::PutOnTopCam { id: *self.get_id() });
			} else {
				manager.push_command(Command::PutOnTopNoCam { id: *self.get_id() });
			};
			self.grab_delta = if let Some(camera) = camera {
				Some(self.base.rect.position - camera.transform().inverse() * input.mouse.position.cast())
			} else {
				Some(self.base.rect.position - input.mouse.position.cast())
			};
		} else if self.base.state.is_released() {
			self.grab_delta = None;
		} else if let Some(grab_delta) = self.grab_delta {
			if !input.mouse.delta.is_empty() {
				let mouse_position = if let Some(camera) = camera {
					camera.transform().inverse() * input.mouse.position.cast()
				} else {
					input.mouse.position.cast()
				};
				self.base.rect.position = mouse_position + grab_delta;
				changed = true;
			}
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>) {
		let color = if self.base.is_pushed() {
			self.style.pushed_color
		} else if self.is_hovered() {
			self.style.hovered_color
		} else {
			self.style.color
		};
		let border_color =
			if self.is_focused() && !self.base.is_pushed() { self.style.focused_color } else { self.style.border_color };
		let rect = if self.base.is_pushed() { self.base.rect.translated(-Self::SHADOW) } else { self.base.rect };

		if let Some(corner_radius) = self.base.radius {
			if self.is_focused() {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA + corner_radius,
				);
			}
			fill_rounded_rect(canvas, camera, color, self.base.rect, corner_radius);
			draw_rounded_rect(canvas, camera, border_color, self.base.rect, corner_radius);
		} else {
			if self.is_focused() {
				canvas.set_blend_mode(BlendMode::Blend);
				fill_rounded_rect(
					canvas,
					camera,
					with_alpha(border_color, FOCUS_HALO_ALPHA),
					self.base.rect.enlarged(FOCUS_HALO_DELTA),
					FOCUS_HALO_DELTA,
				);
			}
			fill_rect(canvas, camera, color, self.base.rect);
			draw_rect(canvas, camera, border_color, self.base.rect);
		}
	}

	fn get_base(&self) -> &Base {
		&self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
