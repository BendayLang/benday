// #![allow(dead_code, unused_variables)]
mod blocs;

use blocs::BlocType;
use pg_sdl::custom_rect::Rect;
use blocs::{Bloc, BlocContainer, BlocElement};
use nalgebra::{Point2, Vector2};
use pg_sdl::app::{App, PgSdl};
use pg_sdl::camera::Camera;
use pg_sdl::color::{hsv_color, Colors};
use pg_sdl::input::Input;
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use pg_sdl::primitives::draw_text;
use pg_sdl::widgets::{
	WidgetsManager,
	button::{Button, ButtonStyle},
	text_input::{TextInput, TextInputStyle}
};

#[derive(PartialEq, Copy, Clone, Debug)]
struct Element {
	bloc_id: u32,
	bloc_element: BlocElement,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Container {
	bloc_id: u32,
	bloc_container: BlocContainer,
}

#[derive(Clone)]
enum AppState {
	Idle { selected_element: Option<Element>, hovered_element: Option<Element> },
	BlocMoving { moving_bloc_id: u32, delta: Vector2<f64>, hovered_container: Option<Container> },
}

pub struct MyApp {
	id_counter: u32,
	app_state: AppState,
	blocs: HashMap<u32, Bloc>,
	blocs_order: Vec<u32>,
}

impl App for MyApp {
	fn update(&mut self, delta_sec: f64, input: &Input, widgets_manager: &mut WidgetsManager, camera: &mut Camera) -> bool {
		let mut changed = false;

		match self.app_state.clone() {
			AppState::Idle { selected_element, hovered_element } => {
				changed |= camera.update(input, widgets_manager.is_widget_focused() || selected_element.is_some());

				// Add new bloc
				if widgets_manager.get::<Button>(0).unwrap().is_pressed() {
					let id = self.id_counter;
					let new_bloc = Bloc::new_bloc(
						id,
						hsv_color((id * 15) as u16, 1., 1.),
						Point2::new(8., 10.) * id as f64,
						BlocType::IfElse,
					);
					self.blocs.insert(id, new_bloc);
					self.blocs_order.push(id);
					self.id_counter += 1;
					update_layout_and_positions(&id, &mut self.blocs);
				}
				// Mouse click
				else if input.mouse.left_button.is_pressed() {
					if let Some(Element { bloc_id, bloc_element }) = hovered_element {
						match bloc_element {
							// Select a bloc
							BlocElement::Body => {
								// Rearrange blocs order
								let mut new_blocs_order = self.blocs_order.clone();
								let childs = self.blocs.get(&bloc_id).unwrap().get_recursive_childs(&self.blocs);
								let childs_order_ids = childs
									.iter()
									.rev()
									.map(|child_id| {
										new_blocs_order.remove(new_blocs_order.iter().position(|i| i == child_id).unwrap())
									})
									.collect::<Vec<u32>>();
								new_blocs_order.extend(childs_order_ids);
								self.blocs_order = new_blocs_order;

								// Rearrange parents / childs
								if let Some(Container { bloc_id: parent_id, bloc_container }) =
									self.blocs.get(&bloc_id).unwrap().get_parent().clone()
								{
									{
										let mut bloc = self.blocs.remove(&parent_id).unwrap();
										bloc.remove_child(bloc_container, &mut self.blocs);
										self.blocs.insert(parent_id, bloc);
									}

									self.blocs.get_mut(&bloc_id).unwrap().set_parent(None);
									let root_id = get_root(&parent_id, &self.blocs);
									update_layout_and_positions(&root_id, &mut self.blocs);
								}
								childs.iter().for_each(|child_id| {
									self.blocs.get_mut(child_id).unwrap().translate(-Bloc::SHADOW);
								});
								let delta = self.blocs.get(&bloc_id).unwrap().get_position().clone()
									- camera.transform.inverse() * input.mouse.position.cast();

								self.app_state = AppState::BlocMoving { moving_bloc_id: bloc_id, delta, hovered_container: None };
							}
							_ => {
								let selected_element = Some(Element { bloc_id, bloc_element });
								self.app_state = AppState::Idle { selected_element, hovered_element };
							}
						}
					}
					// Click in void
					else {
						self.app_state = AppState::Idle { selected_element: None, hovered_element: None };
					}
					changed = true;
				}
				// Update witch element is (mouse) hovered
				if !input.mouse.delta.is_empty() {
					let mouse_position = camera.transform.inverse() * input.mouse.position.cast();
					let mut new_hovered_element = None;
					for id in self.blocs_order.iter().rev() {
						if let Some(bloc_element) = self.blocs.get(&id).unwrap().collide_element(mouse_position) {
							new_hovered_element = Some(Element { bloc_id: *id, bloc_element });
							break;
						}
					}
					if new_hovered_element != hovered_element {
						self.app_state =
							AppState::Idle { selected_element: selected_element, hovered_element: new_hovered_element };
						changed = true;
					}
				}

				// Update the selected element
				/*
				if let Some(Element { bloc_id, bloc_element }) = selected_element {
					self.blocs.get_mut(&bloc_id).unwrap().update_element(&bloc_element, input, delta_sec, text_drawer, &camera);
				}
				 */
			}
			AppState::BlocMoving { moving_bloc_id, delta, hovered_container } => {
				// Release the bloc
				if input.mouse.left_button.is_released() {
					if let Some(Container { bloc_id, bloc_container }) = hovered_container.clone() {
						// Update parents and childs
						{
							let mut bloc = self.blocs.remove(&bloc_id).unwrap();
							bloc.set_child(moving_bloc_id, bloc_container.clone(), &mut self.blocs);
							self.blocs.insert(bloc_id, bloc);
						}
						self.blocs.get_mut(&moving_bloc_id).unwrap().set_parent(hovered_container);
						// Update layout and childs positions
						let root_id = get_root(&bloc_id, &self.blocs);
						update_layout_and_positions(&root_id, &mut self.blocs);
					} else {
						let childs = self.blocs.get(&moving_bloc_id).unwrap().get_recursive_childs(&self.blocs);
						childs.iter().for_each(|child_id| {
							self.blocs.get_mut(child_id).unwrap().translate(Bloc::SHADOW);
						});
					}

					let element = Some(Element { bloc_id: moving_bloc_id, bloc_element: BlocElement::Body });
					self.app_state = AppState::Idle { selected_element: element, hovered_element: element };
					changed = true;
				// Move the bloc
				}
				// Move the moving bloc
				else if !input.mouse.delta.is_empty() {
					let mouse_position = camera.transform.inverse() * input.mouse.position.cast();
					self.blocs.get_mut(&moving_bloc_id).unwrap().set_position(mouse_position + delta);
					update_layout_and_positions(&moving_bloc_id, &mut self.blocs);

					// Update the (moving bloc) hovered container
					// iter through all blocs to get the bloc with the biggest 'ratio' of "hoveredness"
					let moving_bloc = self.blocs.get(&moving_bloc_id).unwrap();
					let moving_bloc_childs = moving_bloc.get_recursive_childs(&self.blocs);
					let (mut new_hovered_container, mut ratio) = (None, 0.);
					self.blocs_order.iter().for_each(|bloc_id| {
						if !moving_bloc_childs.contains(bloc_id) {
							if let Some((new_bloc_container, new_ratio)) = self
								.blocs
								.get(&bloc_id)
								.unwrap()
								.collide_container(*moving_bloc.get_rect())
							{
								if new_ratio >= ratio {
									new_hovered_container =
										Some(Container { bloc_id: *bloc_id, bloc_container: new_bloc_container });
									ratio = new_ratio;
								}
							}
						}
					});
					if new_hovered_container != hovered_container {
						self.app_state = AppState::BlocMoving { moving_bloc_id, delta, hovered_container: new_hovered_container };
					}
					changed = true;
				}
			}
		}
		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &TextDrawer, camera: &Camera) {
		camera.draw_grid(canvas, text_drawer, Colors::LIGHT_GREY, true, false);

		self.blocs_order.iter().for_each(|bloc_id| {
			let (moving, selected, hovered) = match &self.app_state {
				AppState::Idle { selected_element, hovered_element } => (
					false,
					if let Some(Element { bloc_id: element_bloc_id, bloc_element }) = selected_element {
						if bloc_id == element_bloc_id {
							Some(bloc_element)
						} else {
							None
						}
					} else {
						None
					},
					if let Some(Element { bloc_id: element_bloc_id, bloc_element }) = hovered_element {
						if bloc_id == element_bloc_id {
							Some(bloc_element)
						} else {
							None
						}
					} else {
						None
					},
				),
				AppState::BlocMoving { moving_bloc_id: selected_id, .. } => {
					if bloc_id == selected_id {
						(true, Some(&BlocElement::Body), Some(&BlocElement::Body))
					} else {
						(false, None, None)
					}
				}
			};
			self.blocs.get(bloc_id).unwrap().draw(canvas, text_drawer, &camera, moving, selected, hovered);
		});

		match &self.app_state {
			AppState::BlocMoving { hovered_container, .. } => {
				if let Some(Container { bloc_id, bloc_container }) = hovered_container {
					let bloc = self.blocs.get(bloc_id).unwrap();
					bloc.draw_container_hover(canvas, &camera, bloc_container);
				}
			}
			_ => (),
		}

		if let AppState::BlocMoving { hovered_container, .. } = &self.app_state {
			if let Some(Container { bloc_container, .. }) = hovered_container {
				let text = &format!("{:?}", bloc_container);
				draw_text(canvas, None, text_drawer, Point2::new(100., 50.), text, 20., &TextStyle::default(), Align::TopLeft);
			}
		}
	}
}

fn main() {
	let my_app = &mut MyApp {
		id_counter: 0,
		app_state: AppState::Idle { selected_element: None, hovered_element: None },
		blocs: HashMap::new(),
		blocs_order: Vec::new(),
	};

	let resolution = Vector2::new(1280, 720);

	let mut app = PgSdl::init("Benday", resolution.x, resolution.y, Some(120), true, Colors::LIGHT_GREY);

	app.add_widget(
		Box::new(Button::new(
			Rect::new(100., 100., 200., 100.),
			"New bloc".to_string(),
			ButtonStyle::default(),
			false,
		)),
	);
	/*
	app.add_widget(
		Box::new(Button::new(
			Rect::new(-200., 100., 80., 30.),
			"button".to_string(),
			ButtonStyle::new(Colors::LIGHT_YELLOW, None, 10.),
			true,
		)),
	);
	app.add_widget(
		Box::new(TextInput::new(
			Rect::new(400., 100., 120., 28.),
			"fixed".to_string(),
			TextInputStyle::default(),
			false,
		)),
	);
	app.add_widget(
		Box::new(TextInput::new(
			Rect::new(400., 100., 120., 28.),
			"placeholder".to_string(),
			TextInputStyle::default(),
			true,
		)),
	);
	app.add_widget(
		Box::new(Slider::new(
			Rect::new(100., 300., 200., 40.),
			SliderStyle::new(Colors::ORANGE, Colors::LIGHT_GREY),
			SliderType::Discrete {
				snap: 10,
				default_value: 5,
				display: Some(Box::new(|value| format!("{}", value))),
			},
			false,
		)),
	);
	app.add_widget(
		Box::new(Slider::new(
			Rect::new(320., 300., 30., 200.),
			SliderStyle::default(),
			SliderType::Continuous {
				default_value: 0.2,
				display: Some(Box::new(|value| format!("{:.2}", value))),
			},
			false,
		)),
	);
	app.add_widget(
		Box::new(Switch::new(
			Rect::new(100., 280., 50., 30.),
			SwitchStyle::default(),
			true,
		)),
	);
	app.add_widget(
		Box::new(Switch::new(
			Rect::new(135., 350., 30., 60.),
			SwitchStyle::new(Colors::LIGHT_GREEN, Colors::LIGHT_RED),
			true,
		)),
	);
	 */

	app.run(my_app);
}

fn get_root(bloc_id: &u32, blocs: &HashMap<u32, Bloc>) -> u32 {
	let mut bloc_id = bloc_id;
	loop {
		if let Some(Container { bloc_id: parent_id, .. }) = blocs.get(bloc_id).unwrap().get_parent() {
			bloc_id = parent_id;
		} else {
			return *bloc_id;
		}
	}
}

fn update_layout_and_positions(bloc_id: &u32, blocs: &mut HashMap<u32, Bloc>) {
	let childs = blocs.get(bloc_id).unwrap().get_recursive_childs(&blocs);
	childs.iter().for_each(|child_id| {
		let mut bloc = blocs.remove(&child_id).unwrap();
		bloc.update_layout(&blocs);
		blocs.insert(*child_id, bloc);
	});
	childs.iter().rev().for_each(|child_id| {
		let mut bloc = blocs.remove(&child_id).unwrap();
		bloc.update_child_position(blocs);
		blocs.insert(*child_id, bloc);
	});
}
