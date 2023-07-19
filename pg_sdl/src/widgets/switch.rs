use crate::camera::Camera;
use crate::color::{darker, with_alpha, Colors};
use crate::custom_rect::Rect;
use crate::input::Input;
use crate::primitives::{draw_circle, draw_polygon, fill_circle, fill_polygon};
use crate::text::TextDrawer;
use crate::vector2::Vector2Plus;
use crate::widgets::{Base, Orientation, Widget, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH};
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use std::f64::consts::PI;

pub struct SwitchStyle {
	on_color: Color,
	off_color: Color,
	thumb_color: Color,
	thumb_hovered_color: Color,
	thumb_pushed_color: Color,
	thumb_focused_color: Color,
	border_color: Color,
}

impl Default for SwitchStyle {
	fn default() -> Self {
		Self {
			on_color: Colors::LIGHT_GREEN,
			off_color: Colors::LIGHT_GREY,
			thumb_color: Colors::LIGHTER_GREY,
			thumb_hovered_color: darker(Colors::LIGHTER_GREY, HOVER),
			thumb_pushed_color: darker(Colors::LIGHTER_GREY, PUSH),
			thumb_focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}

impl SwitchStyle {
	pub fn new(on_color: Color, off_color: Color) -> Self {
		Self {
			on_color,
			off_color,
			thumb_color: Colors::LIGHTER_GREY,
			thumb_hovered_color: darker(Colors::LIGHTER_GREY, HOVER),
			thumb_pushed_color: darker(Colors::LIGHTER_GREY, PUSH),
			thumb_focused_color: Colors::BLUE,
			border_color: Colors::BLACK,
		}
	}
}

/// A switch is a widget that can be toggled __on__ or __off__
pub struct Switch {
	base: Base,
	style: SwitchStyle,
	orientation: Orientation,
	switched: bool,
}

impl Switch {
	pub fn new(rect: Rect, style: SwitchStyle) -> Self {
		let orientation = {
			if rect.width() > rect.height() {
				Orientation::Horizontal
			} else {
				Orientation::Vertical
			}
		};
		Self { base: Base::new(rect), style, orientation, switched: false }
	}

	pub fn set_switched(&mut self, switched: bool) {
		self.switched = switched;
	}

	pub fn is_switched(&self) -> bool {
		self.switched
	}

	fn thumb_position(&self) -> f64 {
		f64::from(self.switched) * self.length()
	}

	fn length(&self) -> f64 {
		match self.orientation {
			Orientation::Horizontal => self.base.rect.width() - self.base.rect.height(),
			Orientation::Vertical => self.base.rect.height() - self.base.rect.width(),
		}
	}
}

impl Widget for Switch {
	fn update(
		&mut self, input: &Input, _delta_sec: f64, _widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		_camera: Option<&Camera>,
	) -> bool {
		let changed = self.base.update(input, Vec::new());
		if self.base.state.is_released() {
			self.switched = !self.switched;
		}
		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		let color = if self.switched { self.style.on_color } else { self.style.off_color };
		let border_color = if focused { self.style.thumb_focused_color } else { self.style.border_color };
		let thumb_color = if self.base.is_pushed() {
			self.style.thumb_pushed_color
		} else if hovered {
			self.style.thumb_hovered_color
		} else {
			self.style.thumb_color
		};

		let thickness = match self.orientation {
			Orientation::Horizontal => self.base.rect.height(),
			Orientation::Vertical => self.base.rect.width(),
		};
		let radius = thickness * 0.5;

		let faces_nb = 9;
		let vertices = match self.orientation {
			Orientation::Horizontal => {
				let mut vertices = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64 - 0.5);
						self.base.rect.mid_right() - Vector2::new(radius, 0.) + Vector2::new_polar(radius, angle)
					})
					.collect::<Vec<Point2<f64>>>();
				vertices.extend(
					(0..=faces_nb)
						.map(|i| {
							let angle = PI * (i as f64 / faces_nb as f64 + 0.5);
							self.base.rect.mid_left() + Vector2::new(radius, 0.) + Vector2::new_polar(radius, angle)
						})
						.collect::<Vec<Point2<f64>>>(),
				);
				vertices
			}
			Orientation::Vertical => {
				let mut vertices = (0..=faces_nb)
					.map(|i| {
						let angle = PI * (i as f64 / faces_nb as f64 + 1.0);
						self.base.rect.mid_bottom() + Vector2::new(0., radius) + Vector2::new_polar(radius, angle)
					})
					.collect::<Vec<Point2<f64>>>();
				vertices.extend(
					(0..=faces_nb)
						.map(|i| {
							let angle = PI * (i as f64 / faces_nb as f64);
							self.base.rect.mid_top() - Vector2::new(0., radius) + Vector2::new_polar(radius, angle)
						})
						.collect::<Vec<Point2<f64>>>(),
				);
				vertices
			}
		};
		fill_polygon(canvas, camera, color, &vertices);
		draw_polygon(canvas, camera, self.style.border_color, &vertices);

		let b = 0.8;

		// Thumb
		let dot_position = match self.orientation {
			Orientation::Horizontal => self.base.rect.mid_left() + Vector2::new(radius + self.thumb_position(), 0.),
			Orientation::Vertical => self.base.rect.mid_top() - Vector2::new(0., radius + self.thumb_position()),
		};
		if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_circle(canvas, camera, with_alpha(border_color, FOCUS_HALO_ALPHA), dot_position, FOCUS_HALO_DELTA + b * radius);
		}
		fill_circle(canvas, camera, thumb_color, dot_position, b * radius);
		draw_circle(canvas, camera, border_color, dot_position, b * radius);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
