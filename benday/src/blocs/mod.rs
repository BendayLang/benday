pub mod as_ast_node;
pub mod bloc;
pub mod containers;

use crate::blocs::bloc::{Bloc, BlocStyle};
use crate::blocs::containers::{FnRelativePosition, Sequence, Slot};
use nalgebra::{Point2, Vector2};
use pg_sdl::color::{paler, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::widgets::button::{Button, ButtonStyle};
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Base, Widget, WidgetId, WidgetsManager};

#[derive(PartialEq, Debug, Clone)]
pub enum BlocContainer {
	Slot { nth_slot: usize },
	Sequence { nth_sequence: usize, place: usize },
}

#[derive(PartialEq, Debug, Clone)]
pub struct Container {
	pub bloc_id: WidgetId,
	pub bloc_container: BlocContainer,
}

pub enum BlocType {
	VariableAssignment,
	IfElse,
	While,
	FunctionCall, // widget 1 = name. slots = params
	FunctionDeclaration,
	Sequence,
}

const WIDGET_SIZE: Vector2<f64> = Vector2::new(80., 20.);
const MARGIN: f64 = 12.;
const INNER_MARGIN: f64 = 6.;
const RADIUS: f64 = 12.;

pub fn new_variable_assignment_bloc(position: Point2<f64>, widgets_manager: &mut WidgetsManager) -> Bloc {
	let bloc_type = BlocType::VariableAssignment;
	let color = Colors::LIGHT_VIOLET;
	let style = BlocStyle::new(color, RADIUS);

	let widgets_ids = vec![widgets_manager.add_widget(
		Box::new(TextInput::new(
			Rect::from_origin(WIDGET_SIZE),
			TextInputStyle::new(paler(color, 0.2), None, 12.),
			"name".to_string(),
		)),
		true,
	)];
	let widgets_relative_positions = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager, _| {
		let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
		let slot_height = bloc.slots[0].get_base(widgets_manager).rect.height();
		Vector2::new(MARGIN, MARGIN + (slot_height - widget_height) * 0.5)
	});

	let mut slots = Vec::new();
	(0..1).for_each(|nth_slot| {
		let fn_relative_position = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
			let widget_width = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.width();
			Vector2::new(MARGIN + widget_width + INNER_MARGIN, MARGIN)
		});
		slots.push(Slot::new(color, "value".to_string(), fn_relative_position, widgets_manager));
	});

	let sequences = Vec::new();

	let get_size = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let widget_width = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.width();
		let slot_size = bloc.slots[0].get_base(widgets_manager).rect.size;
		Vector2::new(2. * MARGIN + widget_width + INNER_MARGIN + slot_size.x, 2. * MARGIN + slot_size.y)
	});

	Bloc::new(position, style, widgets_ids, widgets_relative_positions, slots, sequences, get_size, bloc_type)
}

pub fn new_if_else_bloc(position: Point2<f64>, widgets_manager: &mut WidgetsManager) -> Bloc {
	let bloc_type = BlocType::IfElse;
	let color = Colors::LIGHT_ORANGE;
	let style = BlocStyle::new(color, RADIUS);

	let widgets_ids = Vec::new();
	let widgets_relative_positions = Box::new(|_: &Bloc, _: &WidgetsManager, _| Vector2::zeros());

	let mut slots = Vec::new();
	(0..1).for_each(|nth_slot| {
		let fn_relative_position = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
			let slot_0_height = bloc.slots[0].get_base(widgets_manager).rect.height();
			let sequence_0_height = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.height();
			let max_height = slot_0_height.max(sequence_0_height);
			Vector2::new(MARGIN, MARGIN + (max_height - slot_0_height) * 0.5)
		});
		slots.push(Slot::new(color, "condition".to_string(), fn_relative_position, widgets_manager));
	});

	let mut sequences_ids = Vec::new();
	(0..1).for_each(|nth_slot| {
		let fn_relative_position = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
			let slot_0_size = bloc.slots[0].get_base(widgets_manager).rect.size;
			let sequence_0_height = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.height();
			let max_height = slot_0_size.y.max(sequence_0_height);
			Vector2::new(MARGIN + slot_0_size.x + INNER_MARGIN, MARGIN + (max_height - sequence_0_height) * 0.5)
		});
		sequences_ids.push(Sequence::add(color, fn_relative_position, widgets_manager));
	});

	let get_size = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let slot_0_size = bloc.slots[0].get_base(widgets_manager).rect.size;
		let sequence_0_size = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.size;
		let height = slot_0_size.y.max(sequence_0_size.y);
		Vector2::new(2. * MARGIN + slot_0_size.x + INNER_MARGIN + sequence_0_size.x, 2. * MARGIN + height)
	});

	Bloc::new(position, style, widgets_ids, widgets_relative_positions, slots, sequences_ids, get_size, bloc_type)
}

pub fn new_function_call_bloc(position: Point2<f64>, widgets_manager: &mut WidgetsManager) -> Bloc {
	let bloc_type = BlocType::FunctionCall;
	let color = Colors::LIGHT_CHARTREUSE;
	let style = BlocStyle::new(color, RADIUS);

	let widgets_ids = vec![widgets_manager.add_widget(
		Box::new(TextInput::new(
			Rect::from_origin(WIDGET_SIZE),
			TextInputStyle::new(paler(color, 0.2), None, 12.),
			"name".to_string(),
		)),
		true,
	)];
	let widgets_relative_positions = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager, _| {
		let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
		// let slot_height = bloc.slots[0].get_base(widgets_manager).rect.height();
		Vector2::new(MARGIN, MARGIN + (widget_height) * 0.5)
	});

	let slots = Vec::new();

	let fn_relative_position: FnRelativePosition = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let sequence_0_height = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.height();
		let max_height = 200.;
		Vector2::new(MARGIN + INNER_MARGIN, MARGIN + (max_height - sequence_0_height) * 0.5)
	});
	let sequence_id = Sequence::add(color, fn_relative_position, widgets_manager);
	// TODO caaaacaaaaaa
	let get_size = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let sequence_0_size = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.size;
		let height = 200.;
		Vector2::new(2. * MARGIN + INNER_MARGIN + sequence_0_size.x, 2. * MARGIN + height)
	});

	Bloc::new(position, style, widgets_ids, widgets_relative_positions, slots, vec![sequence_id], get_size, bloc_type)
}

pub fn new_sequence_bloc(position: Point2<f64>, widgets_manager: &mut WidgetsManager) -> Bloc {
	let bloc_type = BlocType::Sequence;
	let color = Colors::LIGHT_GREY;
	let style = BlocStyle::new(color, RADIUS);

	let widgets_ids = Vec::new();
	let widgets_relative_positions = Box::new(|_: &Bloc, _: &WidgetsManager, _| Vector2::zeros());

	let fn_relative_position: FnRelativePosition = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let sequence_0_height = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.height();
		let max_height = 200.;
		Vector2::new(MARGIN + INNER_MARGIN, MARGIN + (max_height - sequence_0_height) * 0.5)
	});
	let sequence_id = Sequence::add(color, fn_relative_position, widgets_manager);
	// TODO caaaacaaaaaa
	let get_size = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let sequence_0_size = widgets_manager.get::<Sequence>(&bloc.sequences_ids[0]).unwrap().get_base().rect.size;
		let height = 200.;
		Vector2::new(2. * MARGIN + INNER_MARGIN + sequence_0_size.x, 2. * MARGIN + height)
	});

	Bloc::new(position, style, widgets_ids, widgets_relative_positions, vec![], vec![sequence_id], get_size, bloc_type)
}
