use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use pg_sdl::camera::Camera;
use pg_sdl::color::{Colors, darker, with_alpha};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::{Input, KeyState};
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::{FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH, Widget};


pub struct DraggableBlocStyle {
	color: Color,
	hovered_color: Color,
	pushed_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: f64,
}

impl DraggableBlocStyle {
	pub fn new(color: Color, corner_radius: f64) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			pushed_color: darker(color, PUSH),
			focused_color: Colors::BLACK,
			border_color: darker(color, 0.5),
			corner_radius,
		}
	}
}

pub struct DraggableBloc {
	rect: Rect,
	state: KeyState,
	style: DraggableBlocStyle,
	has_camera: bool,
}

impl DraggableBloc {
	pub fn new(rect: Rect, style: DraggableBlocStyle) -> Self {
		Self {
			rect,
			state: KeyState::new(),
			style,
			has_camera: true,
		}
	}
}

impl Widget for DraggableBloc {
	fn update(&mut self, input: &Input, _delta_sec: f64, _text_drawer: &TextDrawer, _camera: &Camera) -> bool {
		let mut changed = false;
		self.state.update();

		if input.mouse.left_button.is_pressed() {
			self.state.press();
			changed = true;
		} else if input.mouse.left_button.is_released() {
			self.state.release();
			changed = true;
		}

		changed
	}
	
	fn draw(&self, canvas: &mut Canvas<Window>, _text_drawer: &TextDrawer, camera: &Camera, focused: bool, hovered: bool) {
		let camera = if self.has_camera { Some(camera) } else { None };
		
		let color = if self.state.is_pressed() || self.state.is_down() {
			self.style.pushed_color } else if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };
		
		if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(canvas, camera, with_alpha(border_color, FOCUS_HALO_ALPHA),
			                  self.rect.enlarged(FOCUS_HALO_DELTA), FOCUS_HALO_DELTA + self.style.corner_radius);
		}
		fill_rounded_rect(canvas, camera, color, self.rect, self.style.corner_radius);
		draw_rounded_rect(canvas, camera, border_color, self.rect, self.style.corner_radius);
	}
	
	fn get_rect(&self) -> Rect { self.rect }
	fn get_rect_mut(&mut self) -> &mut Rect { &mut self.rect }
	fn has_camera(&self) -> bool { self.has_camera }
}
