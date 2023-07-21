use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::{draw_rect, draw_rounded_rect, fill_rect, fill_rounded_rect};
use crate::text::TextDrawer;
use crate::widgets::{Base, Widget, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use nalgebra::Vector2;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub struct BlankBoxStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: Option<f64>,
}

impl BlankBoxStyle {
	pub fn new(color: Color, corner_radius: Option<f64>) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			focused_color: Colors::BLACK,
			border_color: darker(color, 0.5),
			corner_radius,
		}
	}
}

pub struct BlankBox {
	base: Base,
	style: BlankBoxStyle,
}

impl BlankBox {
	pub fn new(rect: Rect, style: BlankBoxStyle) -> Self {
		Self { base: Base::new(rect), style }
	}
}

impl Widget for BlankBox {
	fn update(
		&mut self, input: &Input, _delta_sec: f64, _widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		_camera: Option<&Camera>,
	) -> bool {
		self.base.update(input, Vec::new())
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		let color = if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };

		if let Some(corner_radius) = self.style.corner_radius {
			if focused {
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
			if focused {
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

	fn get_base(&self) -> Base {
		self.base
	}

	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
