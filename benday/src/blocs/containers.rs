use super::as_ast_node::AsAstNode;
use crate::blocs::bloc::Bloc;
use models::ast;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, Widget, WidgetId, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub type FnRelativePosition = Box<dyn Fn(&Bloc, &WidgetsManager) -> Vector2<f64>>;

pub struct Slot {
	text_input_id: WidgetId,
	child_id: Option<WidgetId>,
	fn_relative_position: FnRelativePosition,
}

impl Slot {
	const SIZE: Vector2<f64> = Vector2::new(80., 22.);
	const RADIUS: f64 = 4.;

	pub fn new(
		color: Color, placeholder: String, fn_relative_position: FnRelativePosition, widgets_manager: &mut WidgetsManager,
	) -> Self {
		let text_input_id = widgets_manager.add_widget(
			Box::new(TextInput::new(
				Rect::from_origin(Self::SIZE),
				TextInputStyle::new(paler(color, 0.4), Some(Self::RADIUS), 12.),
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

impl AsAstNode for Slot {
	fn as_ast_node(&self, blocs: &Vec<WidgetId>, widgets_manager: &WidgetsManager) -> ast::Node {
		if let Some(child_id) = self.child_id {
			let value_bloc = widgets_manager.get::<Bloc>(&child_id).unwrap();
			value_bloc.as_ast_node(blocs, widgets_manager)
		} else {
			let value_bloc = widgets_manager.get::<TextInput>(&self.text_input_id).unwrap();
			ast::Node { id: self.text_input_id, data: ast::NodeData::RawText(value_bloc.get_text().to_string()) }
		}
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
	const RADIUS: f64 = 3.;
	const GAP_HEIGHT: f64 = 10.;

	pub fn add(color: Color, fn_relative_position: FnRelativePosition, widgets_manager: &mut WidgetsManager) -> WidgetId {
		let style = SequenceStyle::new(darker(color, 0.9), Self::RADIUS);
		widgets_manager.add_widget(
			Box::new(Self {
				base: Base::new(Rect::from_origin(Self::SIZE)),
				style,
				childs_ids: Vec::new(),
				fn_relative_position,
			}),
			true,
		)
	}

	pub fn get_relative_position(&self, bloc: &Bloc, widgets_manager: &WidgetsManager) -> Vector2<f64> {
		(self.fn_relative_position)(bloc, widgets_manager)
	}

	/// Met à jour la taille de la séquence
	pub fn get_updated_size(&self, widgets_manager: &WidgetsManager) -> Vector2<f64> {
		if self.childs_ids.is_empty() {
			Self::SIZE
		} else {
			let width = self
				.childs_ids
				.iter()
				.map(|child_id| widgets_manager.get::<Bloc>(child_id).unwrap().get_base().rect.width())
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap();
			let height = (self
				.childs_ids
				.iter()
				.map(|child_id| widgets_manager.get::<Bloc>(child_id).unwrap().get_base().rect.height())
				.sum::<f64>())
			.max(Self::SIZE.y);
			let nb_blocs = self.childs_ids.len();
			Vector2::new(width, height) + Vector2::new(1, nb_blocs + 1).cast() * Self::GAP_HEIGHT
		}
	}

	pub fn get_updated_layout(&self, widgets_manager: &WidgetsManager) -> Vec<Point2<f64>> {
		let origin = self.base.rect.position;
		self.childs_ids
			.iter()
			.enumerate()
			.map(|(place, child_id)| {
				let y = Self::GAP_HEIGHT + (0..place).map(|i| {
					widgets_manager.get::<Bloc>(&self.childs_ids[i]).unwrap().get_base().rect.height() + Self::GAP_HEIGHT
				}).sum::<f64>();
				origin + Vector2::new(0., y)
			})
			.collect()
	}
	/// Returns a vec of the bloc's childs ids from leaf to root (including itself)
	pub fn get_recursive_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.childs_ids.iter().for_each(|child_id| {
			childs.extend(widgets_manager.get::<Bloc>(child_id).unwrap().get_recursive_childs(widgets_manager));
		});
		childs
	}

	/// Returns a vec of the bloc's childs ids, including widgets, from leaf to root (including itself)
	pub fn get_recursive_widget_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.childs_ids.iter().for_each(|child_id| {
			childs.extend(widgets_manager.get::<Bloc>(child_id).unwrap().get_recursive_widget_childs(widgets_manager));
		});
		childs.push(self.base.id);
		childs
	}
	
	pub fn get_childs_ids(&self) -> &Vec<WidgetId> {
		&self.childs_ids
	}
	pub fn get_childs_ids_mut(&mut self) -> &mut Vec<WidgetId> {
		&mut self.childs_ids
	}

	pub fn get_gap_rect(&self, place: usize, widgets_manager: &WidgetsManager) -> Rect {
		if self.childs_ids.is_empty() {
			self.base.rect
		} else {
			let y = if place == 0 {
				self.base.rect.bottom()
			} else {
				widgets_manager.get::<Bloc>(&self.childs_ids[place - 1]).unwrap().get_base().rect.top()
			};
			Rect::new(self.base.rect.left(), y, self.base.rect.width(), Self::GAP_HEIGHT)
		}
	}
}

impl Widget for Sequence {
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

impl AsAstNode for Sequence {
	fn as_ast_node(&self, blocs: &Vec<WidgetId>, widgets_manager: &WidgetsManager) -> ast::Node {
		ast::Node {
			id: self.base.id,
			data: ast::NodeData::Sequence(
				self.childs_ids
					.iter()
					.map(|id| widgets_manager.get::<Bloc>(id).unwrap().as_ast_node(blocs, widgets_manager))
					.collect(),
			),
		}
	}
}
