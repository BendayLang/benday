use crate::blocs::containers::NewSlot;
use crate::blocs::Container;
use crate::blocs::{BlocContainer, BlocType};
use nalgebra::{Point2, Vector2};
use pg_sdl::camera::Camera;
use pg_sdl::color::{darker, paler, with_alpha, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use pg_sdl::widgets::button::{Button, ButtonStyle};
use pg_sdl::widgets::{Base, Widget, WidgetId, WidgetsManager, FOCUS_HALO_ALPHA, FOCUS_HALO_DELTA, HOVER};
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

pub struct NewBlocStyle {
	color: Color,
	hovered_color: Color,
	focused_color: Color,
	border_color: Color,
	corner_radius: f64,
}

impl NewBlocStyle {
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

type FnRelativePosition = Box<dyn Fn(&NewBloc, &WidgetsManager, usize) -> Vector2<f64>>;
type FnGetSize = Box<dyn Fn(&NewBloc, &WidgetsManager) -> Vector2<f64>>;

pub struct NewBloc {
	base: Base,
	style: NewBlocStyle,
	grab_delta: Option<Vector2<f64>>,
	widgets_ids: Vec<WidgetId>,
	widgets_relative_positions: FnRelativePosition,
	slots: Vec<NewSlot>,
	slots_relative_positions: FnRelativePosition,
	get_size: FnGetSize,
	parent: Option<Container>,
	bloc_type: BlocType,
}

impl NewBloc {
	const SHADOW: Vector2<f64> = Vector2::new(6., 8.);
	const W_SIZE: Vector2<f64> = Vector2::new(80., 20.);
	const MARGIN: f64 = 12.;
	const INNER_MARGIN: f64 = 6.;

	pub fn add(position: Point2<f64>, bloc_type: BlocType, widgets_manager: &mut WidgetsManager) -> WidgetId {
		let style = NewBlocStyle::new(Colors::LIGHT_BLUE, 12.);
		let widgets_ids = vec![widgets_manager.add_widget(
			Box::new(Button::new(
				Rect::from(Point2::origin(), Self::W_SIZE),
				ButtonStyle::new(paler(style.color, 0.4), Some(7.), 12.),
				"button".to_string(),
			)),
			true,
		)];
		let widgets_relative_positions = Box::new(|_bloc: &Self, _: &WidgetsManager, _| Vector2::new(Self::MARGIN, Self::MARGIN));
		let slots = vec![NewSlot::new(style.color, "slot".to_string(), widgets_manager)];
		let slots_relative_positions = Box::new(|bloc: &Self, widgets_manager: &WidgetsManager, _| {
			let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
			Vector2::new(Self::MARGIN, Self::MARGIN + widget_height + Self::INNER_MARGIN)
		});
		let get_size = Box::new(|bloc: &Self, widgets_manager: &WidgetsManager| {
			let widget_height = widgets_manager.get_widget(&bloc.widgets_ids[0]).unwrap().get_base().rect.height();
			let slot_size = bloc.slots[0].get_base(widgets_manager).rect.size;
			Vector2::new(2. * Self::MARGIN + slot_size.x, 2. * Self::MARGIN + widget_height + Self::INNER_MARGIN + slot_size.y)
		});
		let mut bloc = Self {
			base: Base::new(Rect::from(position, Vector2::zeros())),
			style,
			grab_delta: None,
			widgets_ids: widgets_ids.clone(),
			widgets_relative_positions,
			slots: slots.clone(),
			slots_relative_positions,
			get_size,
			parent: None,
			bloc_type: BlocType::VariableAssignment,
		};
		bloc.update_size(widgets_manager);
		bloc.update_position(widgets_manager);
		let id = widgets_manager.add_widget(Box::new(bloc), true);

		widgets_ids.iter().for_each(|widget_id| widgets_manager.put_on_top_cam(widget_id));
		slots.iter().for_each(|slot| widgets_manager.put_on_top_cam(&slot.get_id()));
		id
	}

	/// Met à jour la taille du bloc
	pub fn update_size(&mut self, widgets_manager: &mut WidgetsManager) {
		self.base.rect.size = (self.get_size)(self, widgets_manager);
	}

	/// Met à jour la position de ses enfants (widgets, slots et séquences)
	pub fn update_position(&mut self, widgets_manager: &mut WidgetsManager) {
		self.widgets_ids.iter().enumerate().for_each(|(nth_widget, &widget_id)| {
			widgets_manager.get_widget_mut(&widget_id).unwrap().get_base_mut().rect.position =
				self.base.rect.position + (self.widgets_relative_positions)(self, widgets_manager, nth_widget);
		});
		self.slots.iter().enumerate().for_each(|(nth_slot, slot)| {
			slot.get_base_mut(widgets_manager).rect.position =
				self.base.rect.position + (self.slots_relative_positions)(self, widgets_manager, nth_slot);
		});
	}

	/// Returns a vec of the bloc's childs ids from leaf to root (including itself)
	pub fn get_recursive_bloc_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			if slot.has_child() {
				childs.extend(widgets_manager.get::<Self>(&slot.get_id()).unwrap().get_recursive_bloc_childs(widgets_manager));
			}
		});
		/* TODO
		self.sequences.iter().for_each(|sequence| {
			childs.extend(sequence.get_recursive_childs(widgets_manager));
		});
		 */
		childs.push(self.base.id);
		childs
	}
	/// Returns a vec of the bloc's childs ids, including widgets, from leaf to root (including itself)
	pub fn get_recursive_childs(&self, widgets_manager: &WidgetsManager) -> Vec<WidgetId> {
		let mut childs = Vec::new();
		self.slots.iter().for_each(|slot| {
			if slot.has_child() {
				childs.extend(widgets_manager.get::<Self>(&slot.get_id()).unwrap().get_recursive_childs(widgets_manager));
			} else {
				childs.push(slot.get_id());
			}
		});
		/* TODO
		self.sequences.iter().for_each(|sequence| {
			childs.extend(sequence.get_recursive_childs(widgets_manager));
		});
		 */
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
				let slot_rect = slot.get_base(widgets_manager).rect;
				if slot_rect.collide_rect(rect) {
					let new_ratio = (slot_rect.center() - rect.center()).norm();
					if new_ratio > ratio {
						bloc_container = Some(BlocContainer::Slot { nth_slot: nth_slot });
						ratio = new_ratio;
					}
				}
			}
		});
		/*
		self.sequences.iter().enumerate().for_each(|(sequence_id, sequence)| {
			if sequence.get_rect().collide_rect(rect.translated(-self.base.rect.position.coords)) {
				let (place, new_ratio) = sequence.get_place_ratio(rect.translated(-self.base.rect.position.coords));
				if new_ratio > ratio {
					bloc_container = Some(BlocContainer::Sequence { sequence_id, place });
					ratio = new_ratio;
				}
			}
		});
		 */
		if let Some(bloc_container) = bloc_container {
			return Some((bloc_container, ratio));
		}
		None
	}

	pub fn get_parent(&self) -> &Option<Container> {
		&self.parent
	}

	/// Sets parents and childs depending on if there is a connection or a disconnection
	pub fn set_parent_and_child(parent: &Container, child_id: &WidgetId, connection: bool, widgets_manager: &mut WidgetsManager) {
		widgets_manager.get_mut::<NewBloc>(child_id).unwrap().parent = if connection { Some(parent.clone()) } else { None };

		let Container { bloc_id: parent_id, bloc_container } = parent;
		match bloc_container {
			BlocContainer::Slot { nth_slot } => {
				widgets_manager.get_mut::<NewBloc>(parent_id).unwrap().slots[*nth_slot].set_child(if connection {
					Some(*child_id)
				} else {
					None
				});

				let text_input_id = widgets_manager.get::<NewBloc>(parent_id).unwrap().slots[*nth_slot].get_text_input_id();
				let text_input = widgets_manager.get_widget_mut(&text_input_id).unwrap();
				if connection {
					text_input.get_base_mut().set_invisible();
				} else {
					text_input.get_base_mut().set_visible();
					widgets_manager.put_on_top_cam(&text_input_id);
					let child_childs = widgets_manager.get::<NewBloc>(child_id).unwrap().get_recursive_childs(widgets_manager);
					child_childs.iter().rev().for_each(|child_id| widgets_manager.put_on_top_cam(child_id));
				}
			}
			BlocContainer::Sequence { .. } => {
				todo!()
			}
		}
	}

	pub fn get_root(&self, widgets_manager: &WidgetsManager) -> WidgetId {
		let mut id = self.base.id;
		loop {
			if let Some(Container { bloc_id: parent_id, .. }) = widgets_manager.get::<NewBloc>(&id).unwrap().parent {
				id = parent_id;
			} else {
				return id;
			}
		}
	}
}

impl Widget for NewBloc {
	fn update(
		&mut self, input: &Input, _delta_sec: f64, widgets_manager: &mut WidgetsManager, _text_drawer: &TextDrawer,
		camera: Option<&Camera>,
	) -> bool {
		let camera = camera.unwrap();
		let mut changed = self.base.update(input, Vec::new());

		let mut childs_ids = self.get_recursive_childs(widgets_manager);
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
		let position = rect.position + Vector2::new(Self::MARGIN, Self::MARGIN * 0.5);
		draw_text(canvas, camera, text_drawer, position, &text, 10., &TextStyle::default(), Align::Left);
	}

	fn get_base(&self) -> Base {
		self.base
	}
	fn get_base_mut(&mut self) -> &mut Base {
		&mut self.base
	}
}
