use nalgebra::{Point2, Vector2};
use pg_sdl::color::{Colors, darker, paler, with_alpha};
use pg_sdl::custom_rect::Rect;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER, Widget, WidgetId, WidgetsManager};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use pg_sdl::camera::Camera;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use crate::blocs::bloc::Bloc;


pub type FnRelativePosition = Box<dyn Fn(&Bloc, &WidgetsManager) -> Vector2<f64>>;

pub struct Slot {
	text_input_id: WidgetId,
	child_id: Option<WidgetId>,
	fn_relative_position: FnRelativePosition,
}

impl Slot {
	const DEFAULT_SIZE: Vector2<f64> = Vector2::new(80., 22.);

	pub fn new(color: Color, placeholder: String, fn_relative_position: FnRelativePosition, widgets_manager: &mut WidgetsManager) -> Self {
		let text_input_id = widgets_manager.add_widget(
			Box::new(TextInput::new(
				Rect::from_origin(Self::DEFAULT_SIZE),
				TextInputStyle::new(paler(color, 0.4), Some(4.), 12.),
				placeholder,
			)),
			true,
		);
		Self { text_input_id, child_id: None, fn_relative_position }
	}
	
	pub fn get_relative_position(&self, bloc: &Bloc, widgets_manager: &WidgetsManager) -> Vector2<f64> {
		(self.fn_relative_position)(bloc, widgets_manager)
	}

	pub fn has_child(&self) -> bool {
		self.child_id.is_some()
	}

	pub fn set_child(&mut self, child_id: Option<WidgetId>) {
		self.child_id = child_id;
	}

	pub fn get_id(&self) -> WidgetId {
		if let Some(child_id) = self.child_id {
			child_id
		} else {
			self.text_input_id
		}
	}

	pub fn get_text_input_id(&self) -> WidgetId {
		self.text_input_id
	}

	pub fn get_base(&self, widgets_manager: &WidgetsManager) -> Base {
		widgets_manager.get_widget(&self.get_id()).unwrap().get_base()
	}
	pub fn get_base_mut<'a>(&'a self, widgets_manager: &'a mut WidgetsManager) -> &mut Base {
		widgets_manager.get_widget_mut(&self.get_id()).unwrap().get_base_mut()
	}
}


pub struct SequenceStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: f64,
}

impl SequenceStyle {
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


pub struct Sequence {
	base: Base,
	style: SequenceStyle,
	childs_ids: Vec<WidgetId>,
	fn_relative_position: FnRelativePosition,
}

impl Sequence {
	const SIZE: Vector2<f64> = Vector2::new(50., 40.);
	
	pub fn new(style: SequenceStyle, fn_relative_position: FnRelativePosition) -> Self{
		Self {
			base: Base::new(Rect::from_origin(Vector2::zeros())),
			style,
			childs_ids: Vec::new(),
			fn_relative_position,
		}
	}
	
	pub fn get_relative_position(&self, bloc: &Bloc, widgets_manager: &WidgetsManager) -> Vector2<f64> {
		(self.fn_relative_position)(bloc, widgets_manager)
	}
}

impl Widget for Sequence {
	fn update(&mut self, input: &Input, _delta_sec: f64, _widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer, _camera: Option<&Camera>) -> bool {
		self.base.update(input, Vec::new())
	}
	
	fn draw(&self, canvas: &mut Canvas<Window>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool) {
		let color = if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused { self.style.focused_color } else { self.style.border_color };

		
		if focused {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(border_color, FOCUS_HALO_ALPHA),
				self.base.rect.enlarged(FOCUS_HALO_DELTA),
				FOCUS_HALO_DELTA + self.style.corner_radius,
			);
		}
		fill_rounded_rect(canvas, camera, color, self.base.rect, self.style.corner_radius);
		draw_rounded_rect(canvas, camera, border_color, self.base.rect, self.style.corner_radius);
	}
	
	fn get_base(&self) -> Base {
		self.base
	}
	
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
