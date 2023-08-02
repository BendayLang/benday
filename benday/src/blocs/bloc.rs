use std::fmt::{Display, Formatter};
use std::time::Duration;

use crate::blocs::containers::{Sequence, Slot};
use crate::blocs::{BlocContainer, BlocType};
use crate::blocs::{Container, FnGetSize, WigBloc, RADIUS, TOP_BOX_BT_MARGIN, TOP_BOX_BT_RADIUS, TOP_BOX_BT_SIZE};
use crate::get_base_;
use models::ast;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use pg_sdl::widgets::button::{Button, ButtonStyle};
use pg_sdl::widgets::manager::Command;
use pg_sdl::widgets::text_input::TextInput;
use pg_sdl::widgets::{Base, Manager, Widget, WidgetId, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;

use super::as_ast_node::AsAstNode;

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
	pub widgets: Vec<WigBloc>,
	pub slots: Vec<Slot>,
	pub sequences_ids: Vec<WidgetId>,
	pub get_size: FnGetSize,
	parent: Option<Container>,
	bloc_type: BlocType,
}

impl Display for Bloc {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Bloc({:?}, id: {})", self.bloc_type, self.get_id())
	}
}

impl Bloc {
	pub const SHADOW: Vector2<f64> = Vector2::new(6., 8.);

	pub fn new(
		style: BlocStyle, widgets: Vec<WigBloc>, slots: Vec<Slot>, sequences_ids: Vec<WidgetId>, get_size: FnGetSize,
		bloc_type: BlocType,
	) -> Self {
		let base = Base::new(Rect::zeros(), Some(RADIUS), false);
		Self { base, style, grab_delta: None, widgets, slots, sequences_ids, get_size, parent: None, bloc_type }
	}

	pub fn add(self, container: &Container, manager: &mut Manager) -> WidgetId {
		let id = manager.add_widget(Box::new(self), true);

		let rect = Rect::from_origin(Vector2::new(TOP_BOX_BT_SIZE, TOP_BOX_BT_SIZE));
		let corner_radius = Some(TOP_BOX_BT_RADIUS - TOP_BOX_BT_MARGIN);
		let y = -TOP_BOX_BT_SIZE - TOP_BOX_BT_MARGIN;
		let top_box_widgets = vec![
			WigBloc {
				id: manager.add_widget(
					Box::new(Button::new(rect, corner_radius, ButtonStyle::new(Colors::LIGHT_YELLOW, 12.), "i".to_string())),
					true,
				),
				fn_relative_position: Box::new(move |bloc: &Bloc, _: &Manager| {
					Vector2::new(bloc.base.rect.width() * 0.5 - 1.5 * TOP_BOX_BT_SIZE - 1. * TOP_BOX_BT_MARGIN, y)
				}),
			},
			WigBloc {
				id: manager.add_widget(
					Box::new(Button::new(rect, corner_radius, ButtonStyle::new(Colors::LIGHTER_GREY, 12.), "c".to_string())),
					true,
				),
				fn_relative_position: Box::new(move |bloc: &Bloc, _: &Manager| {
					Vector2::new(bloc.base.rect.width() * 0.5 - 0.5 * TOP_BOX_BT_SIZE, y)
				}),
			},
			WigBloc {
				id: manager.add_widget(
					Box::new(Button::new(rect, corner_radius, ButtonStyle::new(Colors::LIGHT_GREEN, 12.), ">".to_string())),
					true,
				),
				fn_relative_position: Box::new(move |bloc: &Bloc, _: &Manager| {
					Vector2::new(bloc.base.rect.width() * 0.5 + 0.5 * TOP_BOX_BT_SIZE + 1. * TOP_BOX_BT_MARGIN, y)
				}),
			},
		];
		top_box_widgets.iter().for_each(|top_box_widget| manager.get_widget_mut(&top_box_widget.id).set_invisible());

		manager.get_mut::<Bloc>(&id).widgets.extend(top_box_widgets);
		manager
			.get_mut::<Bloc>(&id)
			.widgets
			.iter()
			.for_each(|widget| manager.get_widget_mut(&widget.id).get_base_mut().parent_id = Some(id));
		manager
			.get_mut::<Bloc>(&id)
			.slots
			.iter()
			.for_each(|slot| manager.get_widget_mut(slot.get_id()).get_base_mut().parent_id = Some(id));
		manager
			.get_mut::<Bloc>(&id)
			.sequences_ids
			.iter()
			.for_each(|sequence_id| manager.get_widget_mut(sequence_id).get_base_mut().parent_id = Some(id));

		let mut childs_ids = Vec::new();
		childs_ids.extend(manager.get::<Bloc>(&id).widgets.iter().map(|widget| widget.id).collect::<Vec<WidgetId>>());
		childs_ids.extend(manager.get::<Bloc>(&id).slots.iter().map(|slot| *slot.get_id()).collect::<Vec<WidgetId>>());
		childs_ids.extend(manager.get::<Bloc>(&id).sequences_ids.clone());
		childs_ids.iter().for_each(|child_id| manager.put_on_top_cam(child_id));

		Self::set_parent_and_child(container, &id, true, manager);
		Self::update_size_and_layout(manager);

		id
	}

	/// Met à jour la taille du bloc
	fn update_size(&mut self, manager: &Manager) {
		self.sequences_ids.iter().for_each(|sequence_id| {
			let new_size = manager.get::<Sequence>(sequence_id).get_updated_size(manager);
			manager.get_mut::<Sequence>(sequence_id).get_base_mut().rect.size = new_size;
		});
		self.base.rect.size = (self.get_size)(self, manager);
	}

	/// Met à jour la position de ses enfants (widgets, slots et séquences)
	fn update_layout(&mut self, manager: &Manager) {
		self.widgets.iter().for_each(|widget| {
			manager.get_widget_mut(&widget.id).get_base_mut().rect.position =
				self.base.rect.position + (widget.fn_relative_position)(self, manager);
		});
		// update_layout
		self.slots.iter().for_each(|slot| {
			manager.get_widget_mut(slot.get_id()).get_base_mut().rect.position =
				self.base.rect.position + slot.get_relative_position(self, manager);
		});
		self.sequences_ids.iter().for_each(|sequence_id| {
			let sequence_position = manager.get::<Sequence>(sequence_id).get_relative_position(self, manager);
			manager.get_mut::<Sequence>(sequence_id).get_base_mut().rect.position = self.base.rect.position + sequence_position;

			let sequence = manager.get::<Sequence>(sequence_id);
			sequence.get_updated_layout(manager).iter().zip(sequence.get_childs_ids().clone()).for_each(
				|(new_position, child_id)| {
					manager.get_mut::<Bloc>(&child_id).get_base_mut().rect.position = *new_position;
				},
			);
		});
	}

	pub fn update_size_and_layout(manager: &Manager) {
		let root_id = 1;
		let childs = manager.get::<Bloc>(&root_id).get_recursive_childs(manager);

		childs.iter().for_each(|child_id| {
			manager.get_mut::<Bloc>(child_id).update_size(manager);
		});
		childs.iter().rev().for_each(|child_id| {
			manager.get_mut::<Bloc>(child_id).update_layout(manager);
		});
	}

	/// Sets parents and childs depending on if there is a connection or a disconnection
	pub fn set_parent_and_child(parent: &Container, child_id: &WidgetId, connection: bool, manager: &mut Manager) {
		manager.get_mut::<Bloc>(child_id).parent = if connection { Some(parent.clone()) } else { None };

		let Container { bloc_id: parent_id, bloc_container } = parent;
		match bloc_container {
			BlocContainer::Slot { nth_slot } => {
				manager.get_mut::<Bloc>(parent_id).slots[*nth_slot].set_child(if connection { Some(*child_id) } else { None });

				let text_input_id = manager.get::<Bloc>(parent_id).slots[*nth_slot].get_text_input_id();
				{
					let mut text_input = manager.get_widget_mut(&text_input_id);
					if connection {
						text_input.set_invisible();
					} else {
						text_input.set_visible();
					}
				}
				if !connection {
					manager.put_on_top_cam(&text_input_id);
					let childs_ids = manager.get::<Bloc>(child_id).get_recursive_widget_childs(manager).clone();
					childs_ids.iter().rev().for_each(|child_id| {
						manager.put_on_top_cam(child_id);
					});
				}
			}
			BlocContainer::Sequence { nth_sequence, place } => {
				let sequence_id = manager.get::<Bloc>(parent_id).sequences_ids[*nth_sequence];
				if connection {
					let sequence = manager.get::<Sequence>(&sequence_id);
					let child_nb = sequence.get_childs_ids().len();
					// all this to increment the place in the 'parent' field for the blocs bellow the new one
					sequence.get_childs_ids().clone()[*place..child_nb].iter().for_each(|child_id| {
						let container = manager.get::<Bloc>(child_id).get_parent().clone().unwrap();
						let Container { bloc_id, bloc_container } = container;
						let BlocContainer::Sequence { nth_sequence, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
						manager.get_mut::<Bloc>(child_id).parent = Some(Container {
							bloc_id,
							bloc_container: BlocContainer::Sequence { nth_sequence, place: place + 1 }
						});
					});
				} else {
					let sequence = manager.get::<Sequence>(&sequence_id);
					let child_nb = sequence.get_childs_ids().len();
					// all this to decrement the place in the 'parent' field for the blocs bellow the new one
					sequence.get_childs_ids().clone()[(place + 1)..child_nb].iter().for_each(|child_id| {
						let container = manager.get::<Bloc>(child_id).get_parent().clone().unwrap();
						let Container { bloc_id, bloc_container } = container;
						let BlocContainer::Sequence { nth_sequence, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
						manager.get_mut::<Bloc>(child_id).parent = Some(Container {
							bloc_id,
							bloc_container: BlocContainer::Sequence { nth_sequence, place: place - 1 }
						});
					});
				}
				let mut sequence = manager.get_mut::<Sequence>(&sequence_id);
				if connection {
					sequence.get_childs_ids_mut().insert(*place, *child_id);
				} else {
					sequence.get_childs_ids_mut().remove(*place);
				}
			}
		}

		if connection {
			let childs_ids = manager.get::<Sequence>(&0).get_recursive_widget_childs(manager).clone();
			childs_ids.iter().rev().for_each(|child_id| {
				manager.put_on_top_cam(child_id);
			});
		}
	}

	pub fn get_parent(&self) -> &Option<Container> {
		&self.parent
	}

	/// Returns a vec of the bloc's childs ids (of type Bloc) from leaf to root (including itself)
	pub fn get_recursive_childs(&self, manager: &Manager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			if slot.has_child() {
				childs.extend(manager.get::<Self>(slot.get_id()).get_recursive_childs(manager));
			}
		});
		self.sequences_ids.iter().for_each(|sequence_id| {
			childs.extend(manager.get::<Sequence>(sequence_id).get_recursive_childs(manager));
		});
		childs.push(*self.get_id());
		childs
	}

	/// Returns a vec of the bloc's childs ids, including widgets, from leaf to root (including itself)
	pub fn get_recursive_widget_childs(&self, manager: &Manager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().rev().for_each(|slot| {
			if slot.has_child() {
				childs.extend(manager.get::<Self>(slot.get_id()).get_recursive_widget_childs(manager));
			} else {
				childs.push(*slot.get_id());
			}
		});
		self.sequences_ids.iter().rev().for_each(|sequence_id| {
			childs.extend(manager.get::<Sequence>(sequence_id).get_recursive_widget_childs(manager));
		});
		self.widgets.iter().for_each(|widget| childs.push(widget.id));
		childs.push(*self.get_id());
		childs
	}

	pub fn collide_point_container(&self, point: Point2<f64>, manager: &Manager) -> Option<Container> {
		if !self.base.rect.collide_point(point) {
			return None;
		}

		for (nth_slot, slot) in self.slots.iter().enumerate() {
			if !slot.has_child() && get_base_!(slot, manager).rect.collide_point(point) {
				return Some(Container { bloc_id: *self.get_id(), bloc_container: BlocContainer::Slot { nth_slot } });
			}
		}

		for (nth_sequence, sequence_id) in self.sequences_ids.iter().enumerate() {
			let sequence = manager.get::<Sequence>(sequence_id);
			if sequence.get_base().rect.collide_point(point) {
				for place in 0..=sequence.get_childs_ids().len() {
					if sequence.get_gap_rect(place, manager).collide_point(point) {
						return Some(Container {
							bloc_id: *self.get_id(),
							bloc_container: BlocContainer::Sequence { nth_sequence, place },
						});
					}
				}
			}
		}
		None
	}

	/// Checks if a rect is hovering on a container and checks the 'ratio'
	pub fn collide_rect_container(&self, rect: Rect, manager: &Manager) -> Option<(Container, f64)> {
		if !self.base.rect.collide_rect(rect) {
			return None;
		}

		let (mut bloc_container, mut ratio) = (None, 0.);

		self.slots.iter().enumerate().for_each(|(nth_slot, slot)| {
			if !slot.has_child() {
				if let Some(new_ratio) = get_ratio(rect, get_base_!(slot, manager).rect) {
					if new_ratio > ratio {
						bloc_container =
							Some(Container { bloc_id: *self.get_id(), bloc_container: BlocContainer::Slot { nth_slot } });
						ratio = new_ratio;
					}
				}
			}
		});
		self.sequences_ids.iter().enumerate().for_each(|(nth_sequence, sequence_id)| {
			let sequence = manager.get::<Sequence>(sequence_id);
			if sequence.get_base().rect.collide_rect(rect) {
				(0..=sequence.get_childs_ids().len()).for_each(|place| {
					if let Some(new_ratio) = get_ratio(rect, sequence.get_gap_rect(place, manager)) {
						if new_ratio > ratio {
							bloc_container = Some(Container {
								bloc_id: *self.get_id(),
								bloc_container: BlocContainer::Sequence { nth_sequence, place },
							});
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
}

impl Widget for Bloc {
	fn update(
		&mut self, input: &Input, _delta: Duration, manager: &mut Manager, _: &mut TextDrawer, camera: Option<&Camera>,
	) -> bool {
		let camera = camera.unwrap();
		let mut changed = self.base.update(input, Vec::new());

		let mut childs_ids = self.get_recursive_widget_childs(manager);
		childs_ids.remove(childs_ids.len() - 1);

		if self.base.state.is_pressed() {
			self.grab_delta = Some(self.base.rect.position - camera.transform().inverse() * input.mouse.position.cast());
			manager.push_command(Command::PutOnTopCam { id: *self.get_id() });
			childs_ids.iter().rev().for_each(|child_id| {
				manager.get_widget_mut(child_id).get_base_mut().rect.translate(-Self::SHADOW);
				manager.push_command(Command::PutOnTopCam { id: *child_id });
			});
		} else if self.base.state.is_released() {
			self.grab_delta = None;
			childs_ids.iter().for_each(|child_id| {
				manager.get_widget_mut(child_id).get_base_mut().rect.translate(Self::SHADOW);
			});
		} else if let Some(grab_delta) = self.grab_delta {
			if !input.mouse.delta.is_empty() {
				let new_position = camera.transform().inverse() * input.mouse.position.cast() + grab_delta;
				let delta = new_position - self.base.rect.position;
				childs_ids.iter().for_each(|child_id| {
					manager.get_widget_mut(child_id).get_base_mut().rect.translate(delta);
				});
				self.base.rect.translate(delta);
				changed = true;
			}
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, camera: Option<&Camera>) {
		let color = if self.is_hovered() { self.style.hovered_color } else { self.style.color };
		let border_color =
			if self.is_focused() && !self.base.is_pushed() { self.style.focused_color } else { self.style.border_color };
		let rect = if self.base.is_pushed() { self.base.rect.translated(-Self::SHADOW) } else { self.base.rect };

		if self.base.is_pushed() {
			// shadow
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(self.style.border_color, FOCUS_HALO_ALPHA),
				self.base.rect,
				self.style.corner_radius,
			);
		} else if self.is_focused() {
			// focus
			canvas.set_blend_mode(BlendMode::Blend);
			fill_rounded_rect(
				canvas,
				camera,
				with_alpha(self.style.focused_color, FOCUS_HALO_ALPHA),
				rect.enlarged(FOCUS_HALO_DELTA),
				FOCUS_HALO_DELTA + self.style.corner_radius,
			);
		}

		if self.is_hovered() || self.is_focused() {
			// Top box
			let size =
				Vector2::new(3. * TOP_BOX_BT_SIZE + 4. * TOP_BOX_BT_MARGIN, 2. * (TOP_BOX_BT_SIZE + 2. * TOP_BOX_BT_MARGIN));
			let top_rect = Rect::from(Point2::new(rect.h_mid() - size.x * 0.5, rect.bottom() - size.y * 0.5), size);
			fill_rounded_rect(canvas, camera, color, top_rect, TOP_BOX_BT_RADIUS);
			draw_rounded_rect(canvas, camera, border_color, top_rect, TOP_BOX_BT_RADIUS);
		}

		fill_rounded_rect(canvas, camera, color, rect, self.style.corner_radius);
		draw_rounded_rect(canvas, camera, border_color, rect, self.style.corner_radius);

		let text = format!("{}", self.get_id());
		let position = rect.position + Vector2::new(6., 3.);
		draw_text(canvas, camera, text_drawer, position, &text, 10., &TextStyle::default(), Align::Left);
	}

	fn get_base(&self) -> &Base {
		&self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}

	fn collide_point(&self, point: Point2<f64>) -> bool {
		if self.base.rect.collide_point(point) {
			return true;
		}
		if self.is_hovered() || self.is_focused() {
			let size =
				Vector2::new(3. * TOP_BOX_BT_SIZE + 4. * TOP_BOX_BT_MARGIN, 2. * (TOP_BOX_BT_SIZE + 2. * TOP_BOX_BT_MARGIN));
			let top_rect =
				Rect::from(Point2::new(self.base.rect.h_mid() - size.x * 0.5, self.base.rect.bottom() - size.y * 0.5), size);
			top_rect.collide_point(point)
		} else {
			false
		}
	}

	fn on_focus(&mut self, manager: &mut Manager) {
		let nb_widgets = self.widgets.len();
		self.widgets[(nb_widgets - 3)..nb_widgets].iter().for_each(|WigBloc { id, .. }| manager.get_widget_mut(id).set_visible());
	}

	fn on_unfocus(&mut self, manager: &mut Manager) {
		if self.is_hovered() {
			return;
		}
		let nb_widgets = self.widgets.len();
		self.widgets[(nb_widgets - 3)..nb_widgets]
			.iter()
			.for_each(|WigBloc { id, .. }| manager.get_widget_mut(id).set_invisible());
	}

	fn on_hover(&mut self, manager: &mut Manager) {
		let nb_widgets = self.widgets.len();
		self.widgets[(nb_widgets - 3)..nb_widgets].iter().for_each(|WigBloc { id, .. }| manager.get_widget_mut(id).set_visible());
	}

	fn on_unhover(&mut self, manager: &mut Manager) {
		if self.is_focused() {
			return;
		}
		let nb_widgets = self.widgets.len();
		self.widgets[(nb_widgets - 3)..nb_widgets]
			.iter()
			.for_each(|WigBloc { id, .. }| manager.get_widget_mut(id).set_invisible());
	}
}

fn get_ratio(bloc_rect: Rect, container_rect: Rect) -> Option<f64> {
	if bloc_rect.collide_rect(container_rect) {
		Some(1000. - (container_rect.center() - bloc_rect.bottom_left()).norm())
	} else {
		None
	}
}

impl AsAstNode for Bloc {
	fn as_ast_node(&self, manager: &Manager) -> ast::Node {
		let widget_id = self.get_id();
		let data: ast::NodeData = match self.bloc_type {
			BlocType::Sequence => {
				// TODO pas sur...
				manager.get::<Sequence>(&self.sequences_ids[0]).as_ast_node(manager).data
			}
			BlocType::VariableAssignment => {
				let name_text_input_id = self.widgets[0].id;
				let name_text_input = manager.get::<TextInput>(&name_text_input_id);
				let name = name_text_input.get_text().to_string();
				let value: Box<ast::Node> = Box::new(self.slots[0].as_ast_node(manager));
				ast::NodeData::VariableAssignment(ast::VariableAssignment { name_id: name_text_input_id, name, value })
			}
			BlocType::IfElse => ast::NodeData::IfElse(ast::IfElse {
				r#if: ast::If {
					condition: Box::new(self.slots[0].as_ast_node(manager)),
					sequence: Box::new(manager.get::<Sequence>(&self.sequences_ids[0]).as_ast_node(manager)),
				},
				elif: None,   // TODO
				r#else: None, // TODO
			}),
			BlocType::While => ast::NodeData::While(ast::While {
				is_do: false, // TODO
				condition: Box::new(self.slots[0].as_ast_node(manager)),
				sequence: Box::new(manager.get::<Sequence>(&self.sequences_ids[0]).as_ast_node(manager)),
			}),
			BlocType::FunctionCall => {
				let name_text_input_id = self.widgets[0].id;
				// TODO ca ne sera peut etre plus un text input...
				let name_text_input = manager.get::<TextInput>(&name_text_input_id);
				let name = name_text_input.get_text().to_string();
				let argv: Vec<ast::Node> = self.slots.iter().map(|slot| slot.as_ast_node(manager)).collect();
				ast::NodeData::FunctionCall(ast::FunctionCall { name, argv })
			}
			BlocType::FunctionDeclaration => unimplemented!("Fn decl to ast"),
		};
		ast::Node { id: *widget_id, data }
	}
}
