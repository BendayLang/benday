pub mod bloc;
pub mod containers;

use pg_sdl::widgets::WidgetId;

#[derive(PartialEq, Debug, Clone)]
pub enum BlocContainer {
	Slot { nth_slot: usize },
	Sequence { nth_sequence: usize, place: usize },
}

pub enum BlocType {
	VariableAssignment,
	Print,
	IfElse,
	While,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Container {
	pub bloc_id: WidgetId,
	pub bloc_container: BlocContainer,
}
