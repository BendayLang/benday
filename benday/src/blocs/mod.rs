pub mod as_ast_node;
pub mod bloc;
pub mod containers;

use crate::blocs::bloc::{Bloc, BlocStyle};
use crate::blocs::containers::Slot;
use nalgebra::{Point2, Vector2};
use pg_sdl::color::{paler, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::widgets::button::{Button, ButtonStyle};
use pg_sdl::widgets::{Base, WidgetId, WidgetsManager};

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
	Print,
	IfElse,
	While,
}

const W_SIZE: Vector2<f64> = Vector2::new(80., 20.);
const MARGIN: f64 = 12.;
const INNER_MARGIN: f64 = 6.;

fn new_test_bloc(position: Point2<f64>, widgets_manager: &mut WidgetsManager) -> (Bloc, Vec<WidgetId>, Vec<Slot>) {
	let color = Colors::LIGHT_BLUE;
	let style = BlocStyle::new(color, 12.);
	
	let widgets_ids = vec![widgets_manager.add_widget(
		Box::new(Button::new(
			Rect::from(Point2::origin(), W_SIZE),
			ButtonStyle::new(paler(color, 0.4), Some(7.), 12.),
			"button".to_string(),
		)),
		true,
	)];
	let widgets_relative_positions = Box::new(|_bloc: &Bloc, _: &WidgetsManager, _| Vector2::new(MARGIN, MARGIN));
	let slots = vec![Slot::new(color, "slot".to_string(), widgets_manager)];
	let slots_relative_positions = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager, _| {
		let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
		Vector2::new(MARGIN, MARGIN + widget_height + INNER_MARGIN)
	});
	let get_size = Box::new(|bloc: &Bloc, widgets_manager: &WidgetsManager| {
		let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
		let slot_size = bloc.slots[0].get_base(widgets_manager).rect.size;
		Vector2::new(2. * MARGIN + slot_size.x, 2. * MARGIN + widget_height + INNER_MARGIN + slot_size.y)
	});

	(
		Bloc::new(
			position,
			style,
			widgets_ids.clone(),
			widgets_relative_positions,
			slots.clone(),
			slots_relative_positions,
			get_size,
			BlocType::VariableAssignment,
		),
		widgets_ids,
		slots,
	)
}

fn new_variable_assignment_bloc() -> Bloc {
	todo!()
}
