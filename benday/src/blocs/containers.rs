use nalgebra::{Point2, Vector2};
use pg_sdl::color::paler;
use pg_sdl::custom_rect::Rect;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, WidgetId, WidgetsManager};
use sdl2::pixels::Color;


#[derive(Clone)]
pub struct Slot {
	text_input_id: WidgetId,
	child_id: Option<WidgetId>,
}

impl Slot {
	const DEFAULT_SIZE: Vector2<f64> = Vector2::new(80., 20.);

	pub fn new(color: Color, placeholder: String, widgets_manager: &mut WidgetsManager) -> Self {
		let text_input_id = widgets_manager.add_widget(
			Box::new(TextInput::new(
				Rect::from(Point2::origin(), Self::DEFAULT_SIZE),
				TextInputStyle::new(paler(color, 0.2), None, 12.),
				placeholder,
			)),
			true,
		);
		Self { text_input_id, child_id: None }
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
