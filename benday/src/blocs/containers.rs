use std::time::Duration;

use super::as_ast_node::AsAstNode;
use crate::blocs::bloc::Bloc;
use crate::blocs::FnRelativePosition;
use models::ast;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, Manager, Widget, WidgetId, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;

#[macro_export]
macro_rules! get_base_ {
	($self:ident, $manager:ident) => {
		$manager.get_widget(&$self.get_id()).get_base()
	};
	($self:expr, $manager:ident) => {
		$manager.get_widget(&$self.get_id()).get_base()
	};
}

pub struct Slot {
	text_input_id: WidgetId,
	child_id: Option<WidgetId>,
	fn_relative_position: FnRelativePosition,
}

impl Slot {
	const SIZE: Vector2<f64> = Vector2::new(80., 22.);
	const RADIUS: f64 = 4.;

	pub fn new(color: Color, placeholder: String, fn_relative_position: FnRelativePosition, manager: &mut Manager) -> Self {
		let text_input_id = manager.add_widget(
			Box::new(TextInput::new(
				Rect::from_origin(Self::SIZE),
				Some(Self::RADIUS),
				TextInputStyle::new(paler(color, 0.4), 12., true),
				placeholder,
			)),
			true,
		);
		Self { text_input_id, child_id: None, fn_relative_position }
	}

	pub fn get_relative_position(&self, bloc: &Bloc, manager: &Manager) -> Vector2<f64> {
		(self.fn_relative_position)(bloc, manager)
	}

	pub fn has_child(&self) -> bool {
		self.child_id.is_some()
	}

	pub fn set_child(&mut self, child_id: Option<WidgetId>) {
		self.child_id = child_id;
	}

	pub fn get_id(&self) -> &WidgetId {
		if let Some(child_id) = &self.child_id {
			child_id
		} else {
			&self.text_input_id
		}
	}

	pub fn get_text_input_id(&self) -> WidgetId {
		self.text_input_id
	}
}

impl AsAstNode for Slot {
	fn as_ast_node(&self, manager: &Manager) -> ast::Node {
		if let Some(child_id) = self.child_id {
			let value_bloc = manager.get::<Bloc>(&child_id);
			value_bloc.as_ast_node(manager)
		} else {
			let value_bloc = manager.get::<TextInput>(&self.text_input_id);
			ast::Node { id: self.text_input_id, data: ast::NodeData::RawText(value_bloc.get_text().to_string()) }
		}
	}
}

pub struct SequenceStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
}

impl SequenceStyle {
	pub fn new(color: Color) -> Self {
		Self { color, hovered_color: darker(color, HOVER), focused_color: Colors::BLACK, border_color: darker(color, 0.95) }
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

	pub fn add(color: Color, fn_relative_position: FnRelativePosition, manager: &mut Manager) -> WidgetId {
		let style = SequenceStyle::new(darker(color, 0.95));
		manager.add_widget(
			Box::new(Self {
				base: Base::new(Rect::from_origin(Self::SIZE), Some(Self::RADIUS), false),
				style,
				childs_ids: Vec::new(),
				fn_relative_position,
			}),
			true,
		)
	}

	pub fn get_relative_position(&self, bloc: &Bloc, manager: &Manager) -> Vector2<f64> {
		(self.fn_relative_position)(bloc, manager)
	}

	/// Met à jour la taille de la séquence
	pub fn get_updated_size(&self, manager: &Manager) -> Vector2<f64> {
		if self.childs_ids.is_empty() {
			Self::SIZE
		} else {
			let width = self
				.childs_ids
				.iter()
				.map(|child_id| manager.get::<Bloc>(child_id).get_base().rect.width())
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap();
			let height =
				(self.childs_ids.iter().map(|child_id| manager.get::<Bloc>(child_id).get_base().rect.height()).sum::<f64>())
					.max(Self::SIZE.y);
			let nb_blocs = self.childs_ids.len();
			Vector2::new(width, height) + Vector2::new(1, nb_blocs + 1).cast() * Self::GAP_HEIGHT
		}
	}

	pub fn get_updated_layout(&self, manager: &Manager) -> Vec<Point2<f64>> {
		let origin = self.base.rect.position;
		(0..self.childs_ids.len())
			.map(|place| {
				let y = Self::GAP_HEIGHT
					+ (0..place)
						.map(|i| manager.get::<Bloc>(&self.childs_ids[i]).get_base().rect.height() + Self::GAP_HEIGHT)
						.sum::<f64>();
				origin + Vector2::new(0., y)
			})
			.collect()
	}
	/// Returns a vec of the bloc's childs ids from leaf to root (including itself)
	pub fn get_recursive_childs(&self, manager: &Manager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.childs_ids.iter().for_each(|child_id| {
			childs.extend(manager.get::<Bloc>(child_id).get_recursive_childs(manager));
		});
		childs
	}

	/// Returns a vec of the bloc's childs ids, including widgets, from leaf to root (including itself)
	pub fn get_recursive_widget_childs(&self, manager: &Manager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.childs_ids.iter().rev().for_each(|child_id| {
			childs.extend(manager.get::<Bloc>(child_id).get_recursive_widget_childs(manager));
		});
		childs.push(*self.get_id());
		childs
	}

	pub fn get_childs_ids(&self) -> &Vec<WidgetId> {
		&self.childs_ids
	}
	pub fn get_childs_ids_mut(&mut self) -> &mut Vec<WidgetId> {
		&mut self.childs_ids
	}

	pub fn get_gap_rect(&self, place: usize, manager: &Manager) -> Rect {
		if self.childs_ids.is_empty() {
			self.base.rect
		} else {
			let bottom = if place == 0 {
				self.base.rect.bottom()
			} else {
				manager.get::<Bloc>(&self.childs_ids[place - 1]).get_base().rect.v_mid()
			};
			let top = if place == self.childs_ids.len() {
				self.base.rect.top()
			} else {
				manager.get::<Bloc>(&self.childs_ids[place]).get_base().rect.v_mid()
			};

			Rect::new(self.base.rect.left(), bottom, self.base.rect.width(), top - bottom)
		}
	}
}

impl Widget for Sequence {
	fn update(&mut self, input: &Input, _delta: Duration, _: &mut Manager, _: &mut TextDrawer, _: Option<&Camera>) -> bool {
		self.base.update(input, Vec::new())
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, _text_drawer: &mut TextDrawer, camera: Option<&Camera>) {
		let color = if self.is_hovered() { self.style.hovered_color } else { self.style.color };
		let border_color = if self.is_focused() { self.style.focused_color } else { self.style.border_color };

		if self.is_focused() {
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(border_color, FOCUS_HALO_ALPHA),
				self.base.rect.enlarged(FOCUS_HALO_DELTA),
				FOCUS_HALO_DELTA + self.base.radius.unwrap(),
			);
		}
		fill_rounded_rect(canvas, camera, color, self.base.rect, self.base.radius.unwrap());
		draw_rounded_rect(canvas, camera, border_color, self.base.rect, self.base.radius.unwrap());
	}

	fn get_base(&self) -> &Base {
		&self.base
	}

	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}

impl AsAstNode for Sequence {
	fn as_ast_node(&self, manager: &Manager) -> ast::Node {
		ast::Node {
			id: *self.get_id(),
			data: ast::NodeData::Sequence(
				self.childs_ids.iter().map(|id| manager.get::<Bloc>(id).as_ast_node(manager)).collect(),
			),
		}
	}
}
