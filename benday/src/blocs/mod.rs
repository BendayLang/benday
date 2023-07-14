pub mod containers;
mod widgets;

use crate::blocs::containers::{Sequence, Slot};
use crate::Container;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::Colors;
use pg_sdl::input::Input;
use pg_sdl::prelude::{Align, TextDrawer};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum BlocElement {
	Body,
	DeleteButton,
	CopyButton,
	InfoButton,
	Slot(usize),
	Sequence(usize),
	CustomButton(usize),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlocContainer {
	Slot { slot_id: usize },
	Sequence { sequence_id: usize, place: usize },
}

pub enum BlocType {
	VariableAssignment,
	Print,
	IfElse,
	While,
}

/// A bloc represents a piece of code that can be executed.
///
/// It has a "skeleton" that contains everything that all blocs have in common:
/// - color
/// - position (it's always absolute)
/// - vector of slots
/// - vector of sequences
///
/// And a bloc type, witch is an enum that contains data specific to the bloc.
pub struct Bloc {
	id: u32,
	color: Color,
	position: Point2<f64>,
	size: Vector2<f64>,
	slots: Vec<Slot>,
	slots_positions: Box<dyn Fn(&Self, usize) -> Vector2<f64>>,
	sequences: Vec<Sequence>,
	sequences_positions: Box<dyn Fn(&Self, usize) -> Vector2<f64>>,
	get_size: Box<dyn Fn(&Self) -> Vector2<f64>>,
	parent: Option<Container>,
	bloc_type: BlocType,
}
impl Bloc {
	pub const RADIUS: f64 = 8.0;
	const MARGIN: f64 = 12.0;
	const INNER_MARGIN: f64 = 6.0;
	pub const SHADOW: Vector2<f64> = Vector2::new(6.0, 8.0);
	const TOP_BOX_SIZE: Vector2<f64> = Vector2::new(100.0, 25.0);
	const TOP_BOX_MARGIN: f64 = 3.0;
	const HOVER_ALPHA: u8 = 20;
	// const COLOR: Color = hsv_color(330, 0.3, 1.0);
	const PRINT_TEXT_SIZE: Vector2<f64> = Vector2::new(30.0, 10.0); // size of "PRINT" text
	const IF_TEXT_SIZE: Vector2<f64> = Vector2::new(25.0, 10.0); // size of "IF" text

	pub fn new_bloc(id: u32, color: Color, position: Point2<f64>, bloc_type: BlocType) -> Self {
		let (slots, slots_positions, sequences, sequences_positions, get_size): (
			Vec<Slot>,
			Box<dyn Fn(&Bloc, usize) -> Vector2<f64>>,
			Vec<Sequence>,
			Box<dyn Fn(&Bloc, usize) -> Vector2<f64>>,
			Box<dyn Fn(&Bloc) -> Vector2<f64>>,
		) = match bloc_type {
			BlocType::Print => (
				vec![Slot::new(color, "value".to_string()), Slot::new(color, "value".to_string())],
				Box::new(|bloc: &Bloc, slot_id: usize| match slot_id {
					0 => Vector2::new(Self::PRINT_TEXT_SIZE.x + Self::INNER_MARGIN, 0.0) + Vector2::new(1.0, 1.0) * Self::MARGIN,
					_ => {
						Vector2::new(
							Self::PRINT_TEXT_SIZE.x + Self::INNER_MARGIN,
							bloc.slots[0].get_size().y + Self::INNER_MARGIN,
						) + Vector2::new(1.0, 1.0) * Self::MARGIN
					}
				}),
				Vec::new() as Vec<Sequence>,
				Box::new(|_bloc: &Bloc, _sequence_id| panic!("no sequences in PrintBloc")),
				Box::new(|bloc: &Bloc| {
					let width = bloc.slots[0].get_size().x.max(bloc.slots[1].get_size().x);
					let height = bloc.slots[0].get_size().y + bloc.slots[1].get_size().y;
					Vector2::new(width + Self::PRINT_TEXT_SIZE.x + Self::INNER_MARGIN, height + Self::INNER_MARGIN)
						+ Vector2::new(2.0, 2.0) * Self::MARGIN
				}),
			),
			BlocType::IfElse => (
				vec![Slot::new(color, "condition".to_string())],
				Box::new(|_bloc: &Bloc, _slot_id: usize| {
					Vector2::new(Self::IF_TEXT_SIZE.x + Self::INNER_MARGIN, 0.0) + Vector2::new(1.0, 1.0) * Self::MARGIN
				}),
				vec![Sequence::new(color), Sequence::new(color)],
				Box::new(|bloc: &Bloc, sequence_id| {
					let width = Self::IF_TEXT_SIZE.x + bloc.slots[0].get_size().x + 2.0 * Self::INNER_MARGIN;
					let height = bloc.sequences[0..sequence_id].iter().map(|sequence| sequence.get_size().y).sum::<f64>()
						+ sequence_id as f64 * Self::INNER_MARGIN;
					Vector2::new(width, height) + Vector2::new(1.0, 1.0) * Self::MARGIN
				}),
				Box::new(|bloc: &Bloc| {
					let nb_sequences = bloc.sequences.len();
					let width = Self::IF_TEXT_SIZE.x
						+ bloc.slots[0].get_size().x
						+ bloc
							.sequences
							.iter()
							.map(|sequence| sequence.get_size().x)
							.max_by(|a, b| a.partial_cmp(b).unwrap())
							.unwrap() + 2.0 * Self::INNER_MARGIN;
					let height = (bloc.sequences.iter().map(|sequence| sequence.get_size().y).sum::<f64>()
						+ (nb_sequences - 1) as f64 * Self::INNER_MARGIN)
						.max(bloc.slots[0].get_size().y);
					Vector2::new(width, height) + Vector2::new(2.0, 2.0) * Self::MARGIN
				}),
			),
			_ => panic!("bloc not implemented yet !"),
		};
		let mut bloc = Self {
			id,
			color,
			position,
			size: Vector2::zeros(),
			slots,
			slots_positions,
			sequences,
			sequences_positions,
			get_size,
			parent: None,
			bloc_type,
		};

		(0..bloc.slots.len()).for_each(|slot_id| {
			let slot_position = (*bloc.slots_positions)(&bloc, slot_id);
			bloc.slots[slot_id].set_position(slot_position);
		});
		(0..bloc.sequences.len()).for_each(|sequence_id| {
			let sequence_position = (*bloc.sequences_positions)(&bloc, sequence_id);
			bloc.sequences[sequence_id].set_position(sequence_position);
		});
		bloc.size = (*bloc.get_size)(&bloc);
		bloc
	}

	/*
	pub fn repr(&self, blocs: &HashMap<u32, Bloc>) -> String {
		format!("Bloc( {} )", self.slots.get(0).unwrap().repr(blocs))
	}
	 */

	pub fn set_parent(&mut self, parent: Option<Container>) {
		self.parent = parent
	}

	pub fn get_parent(&self) -> &Option<Container> {
		&self.parent
	}

	pub fn set_position(&mut self, position: Point2<f64>) {
		self.position = position
	}

	pub fn get_position(&self) -> &Point2<f64> {
		&self.position
	}

	pub fn translate(&mut self, delta: Vector2<f64>) {
		self.position += delta;
		self.slots.iter_mut().for_each(|slot| slot.translate(delta));
	}

	pub fn get_size(&self) -> &Vector2<f64> {
		&self.size
	}

	/// Returns a vec of the bloc's childs ids from leaf to root (including itself)
	pub fn get_recursive_childs(&self, blocs: &HashMap<u32, Bloc>) -> Vec<u32> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			childs.extend(slot.get_recursive_childs(blocs));
		});
		self.sequences.iter().for_each(|sequence| {
			childs.extend(sequence.get_recursive_childs(blocs));
		});
		childs.push(self.id);
		childs
	}

	/// Met à jour la taille du bloc et la position de ses slots et séquences
	pub fn update_layout(&mut self, blocs: &HashMap<u32, Bloc>) {
		self.slots.iter_mut().for_each(|slot| slot.update_size(blocs));
		(0..self.slots.len()).for_each(|slot_id| {
			let slot_position = (*self.slots_positions)(&self, slot_id);
			self.slots[slot_id].set_position(slot_position);
		});
		self.sequences.iter_mut().for_each(|sequence| sequence.update_size(blocs));
		(0..self.sequences.len()).for_each(|sequence_id| {
			let sequence_position = (*self.sequences_positions)(&self, sequence_id);
			self.sequences[sequence_id].set_position(sequence_position);
		});
		self.size = (*self.get_size)(&self);
	}

	/// Met à jour la position de ses enfants
	pub fn update_child_position(&mut self, blocs: &mut HashMap<u32, Bloc>) {
		self.slots.iter_mut().for_each(|slot| slot.update_child_position(self.position, blocs));
		self.sequences.iter().for_each(|sequences| sequences.update_child_position(self.position, blocs))
	}

	pub fn collide_element(&self, point: Point2<f64>) -> Option<BlocElement> {
		if !self.collide_point(point) {
			return None;
		}

		for (slot_id, slot) in self.slots.iter().enumerate() {
			if slot.collide_point(point - self.position.coords) {
				return Some(BlocElement::Slot(slot_id));
			}
		}

		for (sequence_id, sequence) in self.sequences.iter().enumerate() {
			if sequence.collide_point(point - self.position.coords) {
				return Some(BlocElement::Sequence(sequence_id));
			}
		}

		Some(BlocElement::Body)
	}

	pub fn update_element(
		&mut self, bloc_element: &BlocElement, input: &Input, delta_sec: f64, text_drawer: &mut TextDrawer, camera: &Camera,
	) -> bool {
		match bloc_element {
			BlocElement::Slot(slot_id) => self.slots[*slot_id].update_text_box(input, delta_sec, text_drawer, camera),
			BlocElement::CustomButton(_) => false,
			_ => false,
		}
	}
	pub fn collide_container(&self, position: Point2<f64>, size: Vector2<f64>) -> Option<(BlocContainer, f64)> {
		if !self.collide_rect(position, size) {
			return None;
		}

		let (mut bloc_container, mut ratio) = (None, 0.0);

		self.slots.iter().enumerate().for_each(|(slot_id, slot)| {
			if slot.collide_rect(position - self.position.coords, size) && !slot.has_child() {
				let new_ratio = slot.get_ratio(position - self.position.coords, size);
				if new_ratio > ratio {
					bloc_container = Some(BlocContainer::Slot { slot_id });
					ratio = new_ratio;
				}
			}
		});

		self.sequences.iter().enumerate().for_each(|(sequence_id, sequence)| {
			if sequence.collide_rect(position - self.position.coords, size) {
				let (place, new_ratio) = sequence.get_place_ratio(position - self.position.coords, size);
				if new_ratio > ratio {
					bloc_container = Some(BlocContainer::Sequence { sequence_id, place });
					ratio = new_ratio;
				}
			}
		});

		if let Some(bloc_container) = bloc_container {
			return Some((bloc_container, ratio));
		}
		None
	}

	pub fn set_child(&mut self, child_id: u32, bloc_container: BlocContainer, blocs: &mut HashMap<u32, Bloc>) {
		match bloc_container {
			BlocContainer::Slot { slot_id } => {
				self.slots[slot_id].set_child(child_id);
			}
			BlocContainer::Sequence { sequence_id, place } => {
				self.sequences[sequence_id].set_child(child_id, place, blocs);
			}
		}
	}

	pub fn remove_child(&mut self, bloc_container: BlocContainer, blocs: &mut HashMap<u32, Bloc>) {
		match bloc_container {
			BlocContainer::Slot { slot_id } => {
				self.slots[slot_id].remove_child();
			}
			BlocContainer::Sequence { sequence_id, place } => {
				self.sequences[sequence_id].remove_child(place, blocs);
			}
		}
	}

	pub fn collide_point(&self, point: Point2<f64>) -> bool {
		self.position.x < point.x
			&& point.x < self.position.x + self.size.x
			&& self.position.y < point.y
			&& point.y < self.position.y + self.size.y
	}

	pub fn collide_rect(&self, position: Point2<f64>, size: Vector2<f64>) -> bool {
		self.position.x < position.x + size.x
			&& position.x < self.position.x + self.size.x
			&& self.position.y < position.y + size.y
			&& position.y < self.position.y + self.size.y
	}

	pub fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera, moving: bool,
		selected: Option<&BlocElement>, hovered: Option<&BlocElement>,
	) {
		// SHADOW
		if moving {
			let shadow_color = Color::from((0, 0, 0, 50));
			canvas.set_blend_mode(BlendMode::Mod);
			camera.fill_rounded_rect(canvas, shadow_color, self.position + Self::SHADOW, self.size, Self::RADIUS);
			canvas.set_blend_mode(BlendMode::None);
		};
		// BODY
		camera.fill_rounded_rect(canvas, self.color, self.position, self.size, Self::RADIUS);
		if selected.is_some() || hovered.is_some() {
			// TOP BOX
			let position = Vector2::new((self.size.x - Self::TOP_BOX_SIZE.x) * 0.5, -Self::TOP_BOX_SIZE.y);
			camera.fill_rounded_rect(canvas, self.color, self.position + position, Self::TOP_BOX_SIZE, Self::RADIUS);
		}
		// HOVERED
		if let Some(element) = hovered {
			match element {
				BlocElement::Body => {
					let hovered_color = Color::from((0, 0, 0, Bloc::HOVER_ALPHA));
					canvas.set_blend_mode(BlendMode::Mod);
					camera.fill_rounded_rect(canvas, hovered_color, self.position, self.size, Self::RADIUS);
					canvas.set_blend_mode(BlendMode::None);
				}
				_ => (),
			}
		}
		// SLOTS
		self.slots.iter().enumerate().for_each(|(slot_id, slot)| {
			let selected =
				if let Some(BlocElement::Slot(selected_slot_id)) = selected { &slot_id == selected_slot_id } else { false };
			let hovered =
				if let Some(BlocElement::Slot(hovered_slot_id)) = hovered { &slot_id == hovered_slot_id } else { false };
			slot.draw(canvas, text_drawer, camera, selected, hovered);
		});
		// SEQUENCES
		self.sequences.iter().enumerate().for_each(|(sequence_id, sequence)| {
			let selected = if let Some(BlocElement::Sequence(selected_sequence_id)) = selected {
				&sequence_id == selected_sequence_id
			} else {
				false
			};
			let hovered = if let Some(BlocElement::Sequence(hovered_sequence_id)) = hovered {
				&sequence_id == hovered_sequence_id
			} else {
				false
			};
			sequence.draw(canvas, camera, self.position, selected, hovered);
		});
		// SELECTED
		if let Some(element) = selected {
			match element {
				BlocElement::Body => {
					camera.draw_rounded_rect(canvas, Colors::BLACK, self.position, self.size, Self::RADIUS);
				}
				_ => (),
			}
		}
		let text = format!("{}", self.id);
		camera.draw_text(canvas, text_drawer, Colors::BLACK, self.position, 15.0, text, Align::TopLeft);
	}

	pub fn draw_container_hover(&self, canvas: &mut Canvas<Window>, camera: &Camera, bloc_container: &BlocContainer) {
		match bloc_container {
			BlocContainer::Slot { slot_id } => {
				self.slots[*slot_id].draw_hover(canvas, camera, self.position);
			}
			BlocContainer::Sequence { sequence_id, place } => {
				self.sequences[*sequence_id].draw_hover(canvas, camera, self.position, *place);
			}
		}
	}
}
