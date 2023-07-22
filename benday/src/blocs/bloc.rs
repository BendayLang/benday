use std::time::Duration;

use crate::blocs::containers::{Sequence, Slot};
use crate::blocs::{new_if_else_bloc, new_variable_assignment_bloc, Container, FnGetSize, FnRelativePositions};
use crate::blocs::{BlocContainer, BlocType};
use models::ast;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use pg_sdl::widgets::button::{Button, ButtonStyle};
use pg_sdl::widgets::text_input::TextInput;
use pg_sdl::widgets::{Base, Widget, WidgetId, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

use super::as_ast_node::AsAstNode;
use super::{new_function_call_bloc, new_sequence_bloc};

pub struct BlocStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: f64,
}

impl BlocStyle {
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

pub struct Bloc {
	base: Base,
	style: BlocStyle,
	grab_delta: Option<Vector2<f64>>,
	pub widgets_ids: Vec<WidgetId>,
	widgets_relative_positions: FnRelativePositions,
	pub slots: Vec<Slot>,
	pub sequences_ids: Vec<WidgetId>,
	get_size: FnGetSize,
	parent: Option<Container>,
	bloc_type: BlocType,
}

impl Bloc {
	const SHADOW: Vector2<f64> = Vector2::new(6., 8.);

	pub fn new(
		position: Point2<f64>, style: BlocStyle, widgets_ids: Vec<WidgetId>, widgets_relative_positions: FnRelativePositions,
		slots: Vec<Slot>, sequences_ids: Vec<WidgetId>, get_size: FnGetSize, bloc_type: BlocType,
	) -> Self {
		Self {
			base: Base::new(Rect::from(position, Vector2::zeros())),
			style,
			grab_delta: None,
			widgets_ids,
			widgets_relative_positions,
			slots,
			sequences_ids,
			get_size,
			parent: None,
			bloc_type,
		}
	}

	pub fn add(position: Point2<f64>, bloc_type: BlocType, widgets_manager: &mut WidgetsManager) -> WidgetId {
		let mut bloc = match bloc_type {
			BlocType::VariableAssignment => new_variable_assignment_bloc(position, widgets_manager),
			BlocType::IfElse => new_if_else_bloc(position, widgets_manager),
			BlocType::Sequence => new_sequence_bloc(position, widgets_manager),
			BlocType::FunctionCall => new_function_call_bloc(position, widgets_manager),
			BlocType::FunctionDeclaration => todo!(),
			BlocType::While => todo!(),
		};

		let widgets_ids = bloc.widgets_ids.clone();
		let slots_ids = bloc.slots.iter().map(|slot| slot.get_id()).collect::<Vec<WidgetId>>();
		let sequences_ids = bloc.sequences_ids.clone();
		bloc.update_size(widgets_manager);
		bloc.update_layout(widgets_manager);

		let id = widgets_manager.add_widget(Box::new(bloc), true);
		widgets_ids.iter().for_each(|widget_id| widgets_manager.put_on_top_cam(widget_id));
		sequences_ids.iter().for_each(|sequence_id| widgets_manager.put_on_top_cam(sequence_id));
		slots_ids.iter().for_each(|slot_id| widgets_manager.put_on_top_cam(slot_id));
		id
	}

	/// Met à jour la taille du bloc
	pub fn update_size(&mut self, widgets_manager: &mut WidgetsManager) {
		self.sequences_ids.iter().for_each(|sequence_id| {
			let new_size = widgets_manager.get::<Sequence>(sequence_id).unwrap().get_updated_size(widgets_manager);
			widgets_manager.get_mut::<Sequence>(sequence_id).unwrap().get_base_mut().rect.size = new_size;
		});
		self.base.rect.size = (self.get_size)(self, widgets_manager);
	}

	/// Met à jour la position de ses enfants (widgets, slots et séquences)
	pub fn update_layout(&mut self, widgets_manager: &mut WidgetsManager) {
		self.widgets_ids.iter().enumerate().for_each(|(nth_widget, &widget_id)| {
			widgets_manager.get_widget_mut(&widget_id).unwrap().get_base_mut().rect.position =
				self.base.rect.position + (self.widgets_relative_positions)(self, widgets_manager, nth_widget);
		});
		// update_layout
		self.slots.iter().for_each(|slot| {
			slot.get_base_mut(widgets_manager).rect.position =
				self.base.rect.position + slot.get_relative_position(self, widgets_manager);
		});
		self.sequences_ids.iter().for_each(|sequence_id| {
			let sequence_position =
				widgets_manager.get::<Sequence>(sequence_id).unwrap().get_relative_position(self, widgets_manager);
			widgets_manager.get_mut::<Sequence>(sequence_id).unwrap().get_base_mut().rect.position =
				self.base.rect.position + sequence_position;

			let sequence = widgets_manager.get::<Sequence>(sequence_id).unwrap();
			sequence.get_updated_layout(widgets_manager).iter().zip(sequence.get_childs_ids().clone()).for_each(
				|(new_position, child_id)| {
					widgets_manager.get_mut::<Bloc>(&child_id).unwrap().get_base_mut().rect.position = *new_position;
				},
			);
		});
	}

	pub fn get_parent(&self) -> &Option<Container> {
		&self.parent
	}

	/// Returns a vec of the bloc's childs ids (of type Bloc) from leaf to root (including itself)
	pub fn get_recursive_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			if slot.has_child() {
				childs.extend(widgets_manager.get::<Self>(&slot.get_id()).unwrap().get_recursive_childs(widgets_manager));
			}
		});
		self.sequences_ids.iter().for_each(|sequence_id| {
			childs.extend(widgets_manager.get::<Sequence>(sequence_id).unwrap().get_recursive_childs(widgets_manager));
		});
		childs.push(self.base.id);
		childs
	}

	/// Returns a vec of the bloc's childs ids, including widgets, from leaf to root (including itself)
	pub fn get_recursive_widget_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			if slot.has_child() {
				childs.extend(widgets_manager.get::<Self>(&slot.get_id()).unwrap().get_recursive_widget_childs(widgets_manager));
			} else {
				childs.push(slot.get_id());
			}
		});
		self.sequences_ids.iter().for_each(|sequence_id| {
			childs.extend(widgets_manager.get::<Sequence>(sequence_id).unwrap().get_recursive_widget_childs(widgets_manager));
		});
		childs.extend(self.widgets_ids.clone());
		childs.push(self.base.id);
		childs
	}

	/// Checks if a rect is hovering on a container and checks the 'ratio'
	pub fn collide_container(&self, rect: Rect, widgets_manager: &WidgetsManager) -> Option<(BlocContainer, f64)> {
		if !self.base.rect.collide_rect(rect) {
			return None;
		}

		let (mut bloc_container, mut ratio) = (None, 0.);

		self.slots.iter().enumerate().for_each(|(nth_slot, slot)| {
			if !slot.has_child() {
				if let Some(new_ratio) = get_ratio(rect, slot.get_base(widgets_manager).rect) {
					if new_ratio > ratio {
						bloc_container = Some(BlocContainer::Slot { nth_slot });
						ratio = new_ratio;
					}
				}
			}
		});
		self.sequences_ids.iter().enumerate().for_each(|(nth_sequence, sequence_id)| {
			let sequence = widgets_manager.get::<Sequence>(sequence_id).unwrap();
			if sequence.get_base().rect.collide_rect(rect) {
				(0..=sequence.get_childs_ids().len()).for_each(|place| {
					if let Some(new_ratio) = get_ratio(rect, sequence.get_gap_rect(place, widgets_manager)) {
						if new_ratio > ratio {
							bloc_container = Some(BlocContainer::Sequence { nth_sequence, place });
							ratio = new_ratio;
						}
					}
				})
			}
		});

		if let Some(bloc_container) = bloc_container {
			return Some((bloc_container, ratio));
		}
		None
	}

	/// Sets parents and childs depending on if there is a connection or a disconnection
	pub fn set_parent_and_child(parent: &Container, child_id: &WidgetId, connection: bool, widgets_manager: &mut WidgetsManager) {
		widgets_manager.get_mut::<Bloc>(child_id).unwrap().parent = if connection { Some(parent.clone()) } else { None };

		let Container { bloc_id: parent_id, bloc_container } = parent;
		match bloc_container {
			BlocContainer::Slot { nth_slot } => {
				widgets_manager.get_mut::<Bloc>(parent_id).unwrap().slots[*nth_slot].set_child(if connection {
					Some(*child_id)
				} else {
					None
				});

				let text_input_id = widgets_manager.get::<Bloc>(parent_id).unwrap().slots[*nth_slot].get_text_input_id();
				let text_input = widgets_manager.get_widget_mut(&text_input_id).unwrap();
				if connection {
					text_input.get_base_mut().set_invisible();
				} else {
					text_input.get_base_mut().set_visible();
					widgets_manager.put_on_top_cam(&text_input_id);
					let child_childs =
						widgets_manager.get::<Bloc>(child_id).unwrap().get_recursive_widget_childs(widgets_manager);
					child_childs.iter().rev().for_each(|child_id| widgets_manager.put_on_top_cam(child_id));
				}
			}
			BlocContainer::Sequence { nth_sequence, place } => {
				let sequence_id = widgets_manager.get::<Bloc>(parent_id).unwrap().sequences_ids[*nth_sequence];
				let sequence = widgets_manager.get::<Sequence>(&sequence_id).unwrap();

				let child_nb = sequence.get_childs_ids().len();
				if connection {
					// all this to increment the place in the 'parent' field for the blocs bellow the new one
					sequence.get_childs_ids().clone()[*place..child_nb].iter().for_each(|child_id| {
						let container = widgets_manager.get::<Bloc>(child_id).unwrap().get_parent().clone().unwrap();
						let Container { bloc_id, bloc_container } = container;
						let BlocContainer::Sequence { nth_sequence, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
						widgets_manager.get_mut::<Bloc>(child_id).unwrap().parent = Some(Container {
							bloc_id,
							bloc_container: BlocContainer::Sequence { nth_sequence, place: place + 1 }
						});
					});
					// insert the bloc
					let sequence = widgets_manager.get_mut::<Sequence>(&sequence_id).unwrap();
					sequence.get_childs_ids_mut().insert(*place, *child_id);
				} else {
					// all this to decrement the place in the 'parent' field for the blocs bellow the new one
					sequence.get_childs_ids().clone()[(place + 1)..child_nb].iter().for_each(|child_id| {
						let container = widgets_manager.get::<Bloc>(child_id).unwrap().get_parent().clone().unwrap();
						let Container { bloc_id, bloc_container } = container;
						let BlocContainer::Sequence { nth_sequence, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
						widgets_manager.get_mut::<Bloc>(child_id).unwrap().parent = Some(Container {
							bloc_id,
							bloc_container: BlocContainer::Sequence { nth_sequence, place: place - 1 }
						});
					});
					// remove the bloc
					let sequence = widgets_manager.get_mut::<Sequence>(&sequence_id).unwrap();
					sequence.get_childs_ids_mut().remove(*place);
				}
			}
		}
	}

	pub fn get_root(&self, widgets_manager: &WidgetsManager) -> WidgetId {
		let mut id = self.base.id;
		loop {
			if let Some(Container { bloc_id: parent_id, .. }) = widgets_manager.get::<Bloc>(&id).unwrap().parent {
				id = parent_id;
			} else {
				return id;
			}
		}
	}
}

impl AsAstNode for Bloc {
	fn as_ast_node(&self, widgets_manager: &WidgetsManager) -> ast::Node {
		let id = self.base.id;
		let data: ast::NodeData = match self.bloc_type {
			BlocType::Sequence => {
				// TODO pas sur...
				widgets_manager.get::<Sequence>(&self.sequences_ids[0]).unwrap().as_ast_node(widgets_manager).data
			}
			BlocType::VariableAssignment => {
				let name_text_input_id = self.widgets_ids.first().unwrap();
				let name_text_input = widgets_manager.get::<TextInput>(name_text_input_id).unwrap();
				let name = name_text_input.get_text().to_string();
				let value: Box<ast::Node> = Box::new(self.slots.first().unwrap().as_ast_node(widgets_manager));
				ast::NodeData::VariableAssignment(ast::VariableAssignment { name, value })
			}
			BlocType::IfElse => ast::NodeData::IfElse(ast::IfElse {
				if_: ast::If {
					condition: Box::new(self.slots.first().unwrap().as_ast_node(widgets_manager)),
					sequence: Box::new(
						widgets_manager.get::<Sequence>(&self.sequences_ids[0]).unwrap().as_ast_node(widgets_manager),
					),
				},
				elif: None,  // TODO
				else_: None, // TODO
			}),
			BlocType::While => ast::NodeData::While(ast::While {
				is_do: false, // TODO
				condition: Box::new(self.slots.first().unwrap().as_ast_node(widgets_manager)),
				sequence: Box::new(widgets_manager.get::<Sequence>(&self.sequences_ids[0]).unwrap().as_ast_node(widgets_manager)),
			}),
			BlocType::FunctionCall => {
				let name_text_input_id = self.widgets_ids.first().unwrap();
				// TODO ca ne sera peut etre plus un text input...
				let name_text_input = widgets_manager.get::<TextInput>(name_text_input_id).unwrap();
				let name = name_text_input.get_text().to_string();
				let argv: Vec<ast::Node> = self.slots.iter().map(|slot| slot.as_ast_node(widgets_manager)).collect();
				ast::NodeData::FunctionCall(ast::FunctionCall { name, argv })
			}
			BlocType::FunctionDeclaration => unimplemented!("Fn decl to ast"),
		};
		ast::Node { id, data }
	}
}

impl Widget for Bloc {
	fn update(
		&mut self, input: &Input, _delta: Duration, widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let camera = camera.unwrap();
		let mut changed = self.base.update(input, Vec::new());

		let mut childs_ids = self.get_recursive_widget_childs(widgets_manager);
		childs_ids.remove(childs_ids.len() - 1);

		if self.base.state.is_pressed() {
			self.grab_delta = Some(self.base.rect.position - camera.transform().inverse() * input.mouse.position.cast());
			widgets_manager.put_on_top_cam(&self.base.id);
			childs_ids.iter().rev().for_each(|child_id| {
				widgets_manager.get_widget_mut(child_id).unwrap().get_base_mut().rect.translate(-Self::SHADOW);
				widgets_manager.put_on_top_cam(child_id);
			});
		} else if self.base.state.is_released() {
			self.grab_delta = None;
			childs_ids.iter().for_each(|child_id| {
				widgets_manager.get_widget_mut(child_id).unwrap().get_base_mut().rect.translate(Self::SHADOW);
			});
		} else if let Some(grab_delta) = self.grab_delta {
			if !input.mouse.delta.is_empty() {
				let new_position = camera.transform().inverse() * input.mouse.position.cast() + grab_delta;
				let delta = new_position - self.base.rect.position;
				childs_ids.iter().for_each(|child_id| {
					widgets_manager.get_widget_mut(child_id).unwrap().get_base_mut().rect.translate(delta);
				});
				self.base.rect.translate(delta);
				changed = true;
			}
		}

		changed
	}

	fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: Option<&Camera>, focused: bool, hovered: bool,
	) {
		let color = if hovered { self.style.hovered_color } else { self.style.color };
		let border_color = if focused && !self.base.is_pushed() { self.style.focused_color } else { self.style.border_color };
		let rect = if self.base.is_pushed() { self.base.rect.translated(-Self::SHADOW) } else { self.base.rect };

		if self.base.is_pushed() {
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

		let text = format!("{}", self.base.id); // format!("{} {:?}", self.base.id, self.parent);
		let position = rect.position + Vector2::new(6., 3.);
		draw_text(canvas, camera, text_drawer, position, &text, 10., &TextStyle::default(), Align::Left);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}

fn get_ratio(moving_rect: Rect, fixed_rect: Rect) -> Option<f64> {
	if moving_rect.collide_rect(fixed_rect) {
		Some(1000. - (fixed_rect.center() - moving_rect.center()).norm())
	} else {
		None
	}
}
