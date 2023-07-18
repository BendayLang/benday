use crate::blocs::{Bloc, BlocContainer};
use crate::Container;
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::primitives::{draw_rounded_rect, fill_rounded_rect};
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::{
	text_input::{TextInput, TextInputStyle},
	Widget,
};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use std::collections::HashMap;

/// Compartiment d'un bloc.
///
/// Peut contenir du texte où un bloc.
pub struct Slot {
	rect: Rect,
	text_box: TextInput,
	child_id: Option<u32>,
}

impl Slot {
	pub const DEFAULT_SIZE: Vector2<f64> = Vector2::new(80.0, 25.0);

	pub fn new(color: Color, placeholder: String) -> Self {
		let text_input_style = TextInputStyle::new(paler(color, 0.4), 12., Some(3.));
		Self {
			rect: Rect::from(Point2::origin(), Self::DEFAULT_SIZE),
			text_box: TextInput::new(Rect::from(Point2::origin(), Self::DEFAULT_SIZE), text_input_style, placeholder),
			child_id: None,
		}
	}

	pub fn get_size(&self) -> Vector2<f64> {
		self.rect.size
	}

	pub fn get_rect(&self) -> Rect {
		self.rect
	}

	/// Returns a vec of the slot's childs ids from leaf to root (including itself)
	pub fn get_recursive_childs(&self, blocs: &HashMap<u32, Bloc>) -> Vec<u32> {
		if let Some(bloc_id) = self.child_id {
			blocs.get(&bloc_id).unwrap().get_recursive_childs(blocs)
		} else {
			Vec::new()
		}
	}

	pub fn update_size(&mut self, blocs: &HashMap<u32, Bloc>) {
		self.rect.size = if let Some(bloc_id) = self.child_id {
			*blocs.get(&bloc_id).unwrap().get_size()
		} else {
			self.text_box.get_base().rect.size
		};
	}

	pub fn update_child_position(&mut self, parent_position: Point2<f64>, blocs: &mut HashMap<u32, Bloc>) {
		if let Some(child_id) = self.child_id {
			blocs.get_mut(&child_id).unwrap().set_position(parent_position + self.rect.position.coords);
		} else {
			self.text_box.get_base_mut().rect.position = parent_position + self.rect.position.coords;
		}
	}

	pub fn set_position(&mut self, position: Point2<f64>) {
		self.rect.position = position;
	}

	pub fn translate(&mut self, delta: Vector2<f64>) {
		self.text_box.get_base_mut().rect.position += delta;
	}

	pub fn get_ratio(&self, rect: Rect) -> f64 {
		1.0 - 2.0 * (rect.v_mid() - self.rect.position.y - self.rect.height() * 0.5).abs() / rect.height()
	}

	pub fn has_child(&self) -> bool {
		self.child_id.is_some()
	}

	/// Vide le slot de son contenu.
	pub fn remove_child(&mut self) {
		self.child_id = None;
		/*
		self.text_box.size.y = Self::DEFAULT_SIZE.y;
		self.text_box.update_size(camera);
		self.text_box.corner_radius = Self::RADIUS;
		self.text_box.hovered = false;
		 */
	}

	/// Ajoute un bloc enfant donné dans le slot.
	pub fn set_child(&mut self, child_id: u32) {
		self.child_id = Some(child_id);
	}

	/// Affiche le slot.
	pub fn draw(
		&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: &Camera, selected: bool, hovered: bool,
	) {
		if self.child_id.is_none() {
			self.text_box.draw(canvas, text_drawer, Some(camera), selected, hovered);
		}
	}

	pub fn draw_hover(&self, canvas: &mut Canvas<Window>, camera: &Camera, position: Point2<f64>) {
		let hovered_color = Color::from((0, 0, 0, 50));
		canvas.set_blend_mode(BlendMode::Mod);
		fill_rounded_rect(canvas, Some(camera), hovered_color, self.rect.translated(position.coords), 3.);
		canvas.set_blend_mode(BlendMode::None);
	}

	// Retourne l’ASTNode de la séquence.
	/*
	fn as_ast(&self, blocs: &HashMap<u32, Bloc>) -> ASTNodeValue {
		if let Some(bloc_id) = self.bloc_id {
			bloc_id
		} else {
			ASTNodeValue(if self.text_box.text.is_empty() { None } else { Some(&self.text_box.text) })
		}
	}
	 */
}

pub struct Sequence {
	rect: Rect,
	color: Color,
	childs_ids: Vec<u32>,
	childs_positions: Vec<Vector2<f64>>,
}

impl Sequence {
	const DEFAULT_SIZE: Vector2<f64> = Vector2::new(120.0, 80.0);
	const MARGIN: f64 = 7.0;
	const RADIUS: f64 = 10.0;

	pub fn new(color: Color) -> Self {
		Self {
			rect: Rect::from(Point2::origin(), Self::DEFAULT_SIZE),
			color,
			childs_ids: Vec::new(),
			childs_positions: Vec::new(),
		}
	}

	pub fn set_position(&mut self, position: Point2<f64>) {
		self.rect.position = position;
	}

	/// Retourne la taille de la séquence.
	pub fn get_size(&self) -> Vector2<f64> {
		self.rect.size
	}

	pub fn get_rect(&self) -> Rect {
		self.rect
	}

	/// Returns a vec of the sequence's childs ids from leaf to root (including itself)
	pub fn get_recursive_childs(&self, blocs: &HashMap<u32, Bloc>) -> Vec<u32> {
		let mut childs = Vec::new();
		self.childs_ids.iter().for_each(|child_id| childs.extend(blocs.get(&child_id).unwrap().get_recursive_childs(blocs)));
		childs
		/* // TODO see if it may be cleaner
		self.childs_ids.iter().map(|child_id| {
			blocs.get(&child_id).unwrap().get_recursive_childs(blocs)
		}).collect()
		 */
	}

	/// Met à jour la taille de la séquence.
	pub fn update_size(&mut self, blocs: &HashMap<u32, Bloc>) {
		self.rect.size = if self.childs_ids.is_empty() {
			Self::DEFAULT_SIZE
		} else {
			let width = self
				.childs_ids
				.iter()
				.map(|bloc_id| blocs.get(bloc_id).unwrap().get_size().x)
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap();
			let height = (self.childs_ids.iter().map(|bloc_id| blocs.get(bloc_id).unwrap().get_size().y).sum::<f64>())
				.max(Self::DEFAULT_SIZE.y);
			let nb_blocs = self.childs_ids.len();
			Vector2::new(width, height) + Vector2::new(1, nb_blocs).cast() * Self::MARGIN
		};
		(0..self.childs_ids.len()).for_each(|place| {
			self.childs_positions[place] = Vector2::new(
				0.0,
				(0..place).map(|i| blocs.get(self.childs_ids.get(i).unwrap()).unwrap().rect.height() + Self::MARGIN).sum(),
			);
		});
	}

	pub fn update_child_position(&self, parent_position: Point2<f64>, blocs: &mut HashMap<u32, Bloc>) {
		self.childs_ids.iter().enumerate().for_each(|(place, child_id)| {
			blocs
				.get_mut(&child_id)
				.unwrap()
				.set_position(parent_position + self.rect.position.coords + self.childs_positions[place]);
		});
	}

	fn get_child_position(&self, place: usize) -> Vector2<f64> {
		if place == self.childs_ids.len() {
			Vector2::new(0.0, self.rect.size.y)
		} else {
			self.childs_positions[place]
		}
	}

	pub fn get_place_ratio(&self, rect: Rect) -> (usize, f64) {
		if self.childs_ids.is_empty() {
			return (0, 1.0 - 2.0 * (rect.v_mid() - self.rect.bottom() - self.rect.height() * 0.5).abs() / rect.height());
		}
		let (mut place, mut ratio) = (0, 0.0);

		for interstice in 0..=self.childs_ids.len() {
			let objectif = if interstice == 0 { 0.0 } else { self.get_child_position(interstice).y - Self::MARGIN * 0.5 };
			let new_ratio = 1.0 - 2.0 * (rect.v_mid() - self.rect.bottom() - objectif).abs() / rect.height();
			if new_ratio > ratio {
				ratio = new_ratio;
				place = interstice;
			}
		}

		(place, ratio)
	}

	/// Enlève le bloc donné de la séquence.
	pub fn remove_child(&mut self, place: usize, blocs: &mut HashMap<u32, Bloc>) {
		// all this to decrement the place in the 'parent' field for the blocs bellow the new one
		self.childs_ids[(place + 1)..self.childs_ids.len()].iter().for_each(|child_id| {
			let container = blocs.get(child_id).unwrap().get_parent().clone().unwrap();
			let Container { bloc_id, bloc_container } = container;
			let BlocContainer::Sequence { sequence_id, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
			blocs.get_mut(child_id).unwrap().set_parent(Some(Container {
				bloc_id,
				bloc_container: BlocContainer::Sequence { sequence_id, place: place - 1 }
			}));
		});
		// remove the bloc
		self.childs_ids.remove(place);
	}

	/// Ajoute un bloc donné à une position donnée dans la séquence.
	pub fn set_child(&mut self, child_id: u32, place: usize, blocs: &mut HashMap<u32, Bloc>) {
		// all this to increment the place in the 'parent' field for the blocs bellow the new one
		self.childs_ids[place..self.childs_ids.len()].iter().for_each(|child_id| {
			let container = blocs.get(child_id).unwrap().get_parent().clone().unwrap();
			let Container { bloc_id, bloc_container } = container;
			let BlocContainer::Sequence { sequence_id, place } = bloc_container else { panic!("Bloc in sequence have parent not of type Sequence") };
			blocs.get_mut(child_id).unwrap().set_parent(Some(Container {
				bloc_id,
				bloc_container: BlocContainer::Sequence { sequence_id, place: place + 1 }
			}));
		});
		// insert the new bloc
		self.childs_ids.insert(place, child_id);
		self.childs_positions.insert(place, Vector2::zeros())
		/*
		if gap_id == self.blocs_ids.len() {
			self.blocs_ids.last().unwrap() = bloc_id;
		} else {
			self.blocs_ids[gap_id] = bloc_id;
		}
		 */
	}

	/// Affiche la séquence.
	pub fn draw(&self, canvas: &mut Canvas<Window>, camera: &Camera, position: Point2<f64>, selected: bool, hovered: bool) {
		fill_rounded_rect(canvas, Some(camera), darker(self.color, 0.7), self.rect.translated(position.coords), Self::RADIUS);
		if selected {
			draw_rounded_rect(canvas, Some(camera), Colors::BLACK, self.rect.translated(position.coords), Self::RADIUS);
		}
		if hovered {
			let hovered_color = Color::from((0, 0, 0, Bloc::HOVER_ALPHA));
			canvas.set_blend_mode(BlendMode::Mod);
			fill_rounded_rect(canvas, Some(camera), hovered_color, self.rect.translated(position.coords), Self::RADIUS);
			canvas.set_blend_mode(BlendMode::None);
		}
	}

	pub fn draw_hover(&self, canvas: &mut Canvas<Window>, camera: &Camera, position: Point2<f64>, place: usize) {
		let (size, place_position, radius) = if place == 0 {
			(Vector2::new(60.0, 40.0), Vector2::zeros(), Self::RADIUS)
		} else {
			(Vector2::new(60.0, Self::MARGIN), self.get_child_position(place) - Vector2::new(0.0, Self::MARGIN), 2.0)
		};

		let hovered_color = Color::from((0, 0, 0, 50));
		canvas.set_blend_mode(BlendMode::Mod);
		fill_rounded_rect(
			canvas,
			Some(camera),
			hovered_color,
			Rect::from(self.rect.position + position.coords + place_position, size),
			radius,
		);
		canvas.set_blend_mode(BlendMode::None);
	}
}

/*
	fn as_AST(self) -> ASTNodeSequence:
		"""Retourne la list contenant les ASTNodes de la séquence."""
		return ASTNodeSequence([bloc.as_ASTNode() for bloc in self.blocs])
*/

/*
	fn bloc_size(self, bloc_id: int) -> Vector2<f64>:
		"""Retourne la taille du bloc donné."""
		return self.blocs[bloc_id] if type(self.blocs[bloc_id]) is Vec2 else self.blocs[bloc_id].size
*/
