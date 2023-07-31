pub mod as_ast_node;
pub mod bloc;
pub mod containers;

use crate::blocs::bloc::{Bloc, BlocStyle};
use crate::blocs::containers::{Sequence, Slot};
use crate::get_base_;
use nalgebra::Vector2;
use pg_sdl::color::{paler, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::widgets::text_input::{TextInput, TextInputStyle};
use pg_sdl::widgets::{Manager, Widget, WidgetId};

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

pub struct WigBloc {
	id: WidgetId,
	fn_relative_position: FnRelativePosition,
}

pub const BLOC_NAMES: [&str; 5] = ["Variable assignment", "If else", "Function call", "Sequence", "While"];

pub enum BlocType {
	VariableAssignment,
	IfElse,
	While,
	FunctionCall, // widget 1 = name. slots = params
	FunctionDeclaration,
	Sequence,
}

impl BlocType {
	pub fn from_string(string: String) -> Option<Self> {
		match string.as_ref() {
			"Variable assignment" => Some(Self::VariableAssignment),
			"If else" => Some(Self::IfElse),
			"Function call" => Some(Self::FunctionCall),
			"Sequence" => Some(Self::Sequence),
			_ => None,
		}
	}
	pub fn new_bloc(&self, manager: &mut Manager) -> Bloc {
		match self {
			BlocType::VariableAssignment => new_variable_assignment_bloc(manager),
			BlocType::IfElse => new_if_else_bloc(manager),
			BlocType::FunctionCall => new_function_call_bloc(manager),
			BlocType::FunctionDeclaration => todo!(),
			BlocType::Sequence => new_sequence_bloc(manager),
			BlocType::While => todo!(),
		}
	}
}

pub type FnRelativePosition = Box<dyn Fn(&Bloc, &Manager) -> Vector2<f64>>;
pub type FnGetSize = Box<dyn Fn(&Bloc, &Manager) -> Vector2<f64>>;

const TEXT_INPUT_SIZE: Vector2<f64> = Vector2::new(80., 20.);
const MARGIN: f64 = 12.;
const INNER_MARGIN: f64 = 6.;
pub const RADIUS: f64 = 12.;
const TOP_BOX_BT_SIZE: f64 = 13.;
const TOP_BOX_BT_MARGIN: f64 = 5.;
const TOP_BOX_BT_RADIUS: f64 = 8.;

pub fn new_variable_assignment_bloc(manager: &mut Manager) -> Bloc {
	let bloc_type = BlocType::VariableAssignment;
	let color = Colors::LIGHT_VIOLET;
	let style = BlocStyle::new(color, RADIUS);

	let widgets = vec![WigBloc {
		id: manager.add_widget(
			Box::new(TextInput::new(
				Rect::from_origin(TEXT_INPUT_SIZE),
				TextInputStyle::new(paler(color, 0.2), None, 12., true),
				"name".to_string(),
			)),
			true,
		),
		fn_relative_position: Box::new(|bloc: &Bloc, manager: &Manager| {
			let widget_height = manager.get_widget(&bloc.widgets[0].id).get_base().rect.height();
			let slot_height = get_base_!(bloc.slots[0], manager).rect.height();
			Vector2::new(MARGIN, MARGIN + (slot_height - widget_height) * 0.5)
		}),
	}];

	let fn_relative_position = Box::new(|bloc: &Bloc, manager: &Manager| {
		let widget_width = manager.get_widget(&bloc.widgets[0].id).get_base().rect.width();
		Vector2::new(MARGIN + widget_width + INNER_MARGIN, MARGIN)
	});
	let slots = vec![Slot::new(color, "value".to_string(), fn_relative_position, manager)];

	let sequences = Vec::new();

	let get_size = Box::new(|bloc: &Bloc, manager: &Manager| {
		let widget_width = manager.get_widget(&bloc.widgets[0].id).get_base().rect.width();
		let slot_size = get_base_!(bloc.slots[0], manager).rect.size;
		Vector2::new(2. * MARGIN + widget_width + INNER_MARGIN + slot_size.x, 2. * MARGIN + slot_size.y)
	});

	Bloc::new(style, widgets, slots, sequences, get_size, bloc_type)
}

pub fn new_if_else_bloc(manager: &mut Manager) -> Bloc {
	let bloc_type = BlocType::IfElse;
	let color = Colors::LIGHT_ORANGE;
	let style = BlocStyle::new(color, RADIUS);

	let widgets = Vec::new();

	let fn_relative_position = Box::new(|bloc: &Bloc, manager: &Manager| {
		let slot_0_height = get_base_!(bloc.slots[0], manager).rect.height();
		let sequence_0_height = manager.get::<Sequence>(&bloc.sequences_ids[0]).get_base().rect.height();
		let max_height = slot_0_height.max(sequence_0_height);
		Vector2::new(MARGIN, MARGIN + (max_height - slot_0_height) * 0.5)
	});
	let slots = vec![Slot::new(color, "condition".to_string(), fn_relative_position, manager)];

	let sequences_ids = (0..2)
		.map(|nth_sequence| {
			let fn_relative_position = Box::new(move |bloc: &Bloc, manager: &Manager| {
				let slot_size = get_base_!(bloc.slots[0], manager).rect.size;
				let sequence_0_height = manager.get::<Sequence>(&bloc.sequences_ids[0]).get_base().rect.height();
				let max_height = slot_size.y.max(sequence_0_height);
				let y = MARGIN
					+ (max_height - sequence_0_height) * 0.5
					+ if nth_sequence == 1 { sequence_0_height + INNER_MARGIN } else { 0. };
				Vector2::new(MARGIN + slot_size.x + INNER_MARGIN, y)
			});
			Sequence::add(color, fn_relative_position, manager)
		})
		.collect();

	let get_size = Box::new(|bloc: &Bloc, manager: &Manager| {
		let slot_0_size = get_base_!(bloc.slots[0], manager).rect.size;
		let sequence_0_size = manager.get::<Sequence>(&bloc.sequences_ids[0]).get_base().rect.size;
		let sequence_1_size = manager.get::<Sequence>(&bloc.sequences_ids[1]).get_base().rect.size;
		let width = slot_0_size.x + INNER_MARGIN + sequence_0_size.x.max(sequence_1_size.x);
		let height = slot_0_size.y.max(sequence_0_size.y) + sequence_1_size.y;
		Vector2::new(2. * MARGIN + width, 2. * MARGIN + height)
	});

	Bloc::new(style, widgets, slots, sequences_ids, get_size, bloc_type)
}

pub fn new_function_call_bloc(manager: &mut Manager) -> Bloc {
	let bloc_type = BlocType::FunctionCall;
	let color = Colors::LIGHT_CHARTREUSE;
	let style = BlocStyle::new(color, RADIUS);

	let widgets = vec![WigBloc {
		id: manager.add_widget(
			Box::new(TextInput::new(
				Rect::from_origin(TEXT_INPUT_SIZE),
				TextInputStyle::new(paler(color, 0.2), None, 12., true),
				"function".to_string(),
			)),
			true,
		),
		fn_relative_position: Box::new(|bloc: &Bloc, manager: &Manager| {
			let widget_height = manager.get_widget(&bloc.widgets[0].id).get_base().rect.height();
			let slots_height: f64 =
				bloc.slots.iter().map(|slot| { get_base_!(slot, manager).rect.size.y } + INNER_MARGIN).sum::<f64>()
					- INNER_MARGIN;
			Vector2::new(MARGIN, MARGIN + (slots_height - widget_height) * 0.5)
		}),
	}];

	let slots = (0..5)
		.map(|nth_slot| {
			let fn_relative_position = Box::new(move |bloc: &Bloc, manager: &Manager| {
				let widget_width = manager.get_widget(&bloc.widgets[0].id).get_base().rect.width();
				let y = INNER_MARGIN
					+ (0..nth_slot).map(|i| get_base_!(bloc.slots[i], manager).rect.height() + INNER_MARGIN).sum::<f64>()
					- INNER_MARGIN;
				Vector2::new(MARGIN + widget_width + INNER_MARGIN, MARGIN + y)
			});
			Slot::new(color, "value".to_string(), fn_relative_position, manager)
		})
		.collect();

	let sequences_ids = Vec::new();

	let get_size = Box::new(|bloc: &Bloc, manager: &Manager| {
		let widget_width = manager.get_widget(&bloc.widgets[0].id).get_base().rect.width();
		let slots_width = bloc
			.slots
			.iter()
			.map(|slot| get_base_!(slot, manager).rect.width())
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap();
		let slots_height: f64 =
			bloc.slots.iter().map(|slot| { get_base_!(slot, manager).rect.size.y } + INNER_MARGIN).sum::<f64>() - INNER_MARGIN;
		Vector2::new(2. * MARGIN + widget_width + INNER_MARGIN + slots_width, 2. * MARGIN + slots_height)
	});

	Bloc::new(style, widgets, slots, sequences_ids, get_size, bloc_type)
}

pub fn new_sequence_bloc(manager: &mut Manager) -> Bloc {
	let bloc_type = BlocType::Sequence;
	let color = Colors::LIGHT_GREY;
	let style = BlocStyle::new(color, RADIUS);

	let widgets = Vec::new();

	let slots = Vec::new();

	let fn_relative_position: FnRelativePosition = Box::new(|_: &Bloc, _: &Manager| Vector2::new(MARGIN, MARGIN));
	let sequences_ids = vec![Sequence::add(color, fn_relative_position, manager)];

	let get_size: FnGetSize = Box::new(|bloc: &Bloc, manager: &Manager| {
		let sequence_size = manager.get::<Sequence>(&bloc.sequences_ids[0]).get_base().rect.size;
		Vector2::new(2. * MARGIN, 2. * MARGIN) + sequence_size
	});

	Bloc::new(style, widgets, slots, sequences_ids, get_size, bloc_type)
}

pub fn new_root_sequence_bloc(manager: &mut Manager) -> Bloc {
	let bloc_type = BlocType::Sequence;
	let color = Colors::GREY;
	let style = BlocStyle::new(color, RADIUS);

	let widgets = Vec::new();

	let slots = Vec::new();

	let fn_relative_position: FnRelativePosition = Box::new(|_: &Bloc, _: &Manager| Vector2::zeros());
	let sequences_ids = vec![Sequence::add(color, fn_relative_position, manager)];

	let get_size: FnGetSize = Box::new(|bloc: &Bloc, manager: &Manager| {
		let sequence_size = manager.get::<Sequence>(&bloc.sequences_ids[0]).get_base().rect.size;
		sequence_size
	});

	Bloc::new(style, widgets, slots, sequences_ids, get_size, bloc_type)
}
