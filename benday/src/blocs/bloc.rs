use nalgebra::Vector2;
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::{Input, KeyState};
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, Widget, WidgetId, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, PUSH};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub struct NewBlocStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: f64,
}

impl NewBlocStyle {
	pub fn new(color: Color, corner_radius: f64) -> Self {
		Self {
			color,
			hovered_color: darker(color, HOVER),
			focused_color: Colors::BLACK,
			border_color: darker(color, 0.5),
			corner_radius,
		}
	}
}

pub struct NewBloc {
	base: Base,
	style: NewBlocStyle,
	grab_delta: Option<Vector2<f64>>,
	text_input_id: WidgetId,
}

impl NewBloc {
	const SHADOW: Vector2<f64> = Vector2::new(6., 8.);
	const TEXT_INPUT_SIZE: Vector2<f64> = Vector2::new(80., 20.);

	pub fn add(rect: Rect, style: NewBlocStyle, widgets_manager: &mut WidgetsManager) {
		widgets_manager.add(
			Box::new(TextInput::new(
				Rect::from(rect.center() - Self::TEXT_INPUT_SIZE * 0.5, Self::TEXT_INPUT_SIZE),
				TextInputStyle::new(paler(style.color, 0.2), 12., None),
				"text".to_string(),
			)),
			true,
		);
		widgets_manager.add(
			Box::new(Self { base: Base::new(rect), style, grab_delta: None, text_input_id: widgets_manager.last_id() }),
			true,
		);
		widgets_manager.put_on_top_cam(widgets_manager.last_id() - 1);
	}
}

impl Widget for NewBloc {
	fn update(
		&mut self, input: &Input, _delta_sec: f64, widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let camera = camera.unwrap();
		let mut changed = self.base.update(input, Vec::new());

		if self.base.state.is_pressed() {
			widgets_manager.put_on_top_cam(self.base.id);
			widgets_manager.put_on_top_cam(self.text_input_id);

			self.grab_delta = Some(self.base.rect.position - camera.transform().inverse() * input.mouse.position.cast());
			widgets_manager.get_mut::<TextInput>(self.text_input_id).unwrap().get_base_mut().rect.position -= Self::SHADOW;
		} else if self.base.state.is_released() {
			self.grab_delta = None;
			widgets_manager.get_mut::<TextInput>(self.text_input_id).unwrap().get_base_mut().rect.position += Self::SHADOW;
		} else if let Some(grab_delta) = self.grab_delta {
			if !input.mouse.delta.is_empty() {
				let new_position = camera.transform().inverse() * input.mouse.position.cast() + grab_delta;
				widgets_manager.get_mut::<TextInput>(self.text_input_id).unwrap().get_base_mut().rect.position +=
					new_position - self.base.rect.position;
				self.base.rect.position = new_position;
				changed = true;
			}
		}

		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		let color = if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused && !self.base.pushed() { self.style.focused_color } else { self.style.border_color };
		let rect = if self.base.pushed() { self.base.rect.translated(-Self::SHADOW) } else { self.base.rect };

		if self.base.pushed() {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(self.style.border_color, FOCUS_HALO_ALPHA),
				self.base.rect,
				self.style.corner_radius,
			);
		} else if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(self.style.focused_color, FOCUS_HALO_ALPHA),
				rect.enlarged(FOCUS_HALO_DELTA),
				FOCUS_HALO_DELTA + self.style.corner_radius,
			);
		}

		fill_rounded_rect(canvas, camera, color, rect, self.style.corner_radius);
		draw_rounded_rect(canvas, camera, border_color, rect, self.style.corner_radius);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
