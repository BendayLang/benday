#![allow(dead_code, unused_variables, unused_imports)]
mod blocs;

use crate::blocs::bloc::Bloc;
use crate::blocs::{BlocType, Container};
use as_any::Downcast;

use nalgebra::{Point2, Vector2};
use pg_sdl::app::{App, PgSdl};
use pg_sdl::camera::Camera;
use pg_sdl::color::{hsv_color, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::{
	button::{Button, ButtonStyle},
	Widget, WidgetId, WidgetsManager,
};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct MyApp {
	hovered_container: Option<Container>,
	/// Lists the widgets that are blocs
	blocs: Vec<WidgetId>,
}

impl App for MyApp {
	fn update(&mut self, _delta_sec: f64, input: &Input, widgets_manager: &mut WidgetsManager, camera: &mut Camera) -> bool {
		let mut changed = false;
		changed |= camera.update(input, widgets_manager.focused_widget().is_some());

		// Add new bloc
		if widgets_manager.get::<Button>(&0).unwrap().is_pressed() {
			let position = Point2::new(8., 10.) * self.blocs.len() as f64;
			let bloc_id = Bloc::add(position, BlocType::Test, widgets_manager);
			self.blocs.push(bloc_id);
		}
		else if widgets_manager.get::<Button>(&1).unwrap().is_pressed() {
			let position = Point2::new(8., 10.) * self.blocs.len() as f64;
			let bloc_id = Bloc::add(position, BlocType::VariableAssignment, widgets_manager);
			self.blocs.push(bloc_id);
		}
		else if widgets_manager.get::<Button>(&2).unwrap().is_pressed() {
			let position = Point2::new(8., 10.) * self.blocs.len() as f64;
			let bloc_id = Bloc::add(position, BlocType::IfElse, widgets_manager);
			self.blocs.push(bloc_id);
		}
		
		if let Some(focused_widget) = widgets_manager.focused_widget() {
			if self.blocs.contains(&focused_widget) {
				let bloc = widgets_manager.get::<Bloc>(&focused_widget).unwrap();
				// Take a bloc
				if bloc.get_base().state.is_pressed() {
					let parent = bloc.get_parent();
					if let Some(container) = parent.clone() {
						Bloc::set_parent_and_child(&container, &focused_widget, false, widgets_manager);
						// Update layout and childs positions
						let root_id = widgets_manager.get::<Bloc>(&container.bloc_id).unwrap().get_root(widgets_manager);
						update_layout(root_id, widgets_manager);
					}
				}
				// Update the (moving bloc) hovered container
				else if bloc.get_base().state.is_down() && !input.mouse.delta.is_empty() {
					// iter through all blocs to get the bloc with the biggest 'ratio'
					let moving_bloc_childs = bloc.get_recursive_bloc_childs(widgets_manager);
					let (mut new_hovered_container, mut ratio) = (None, 0.);

					widgets_manager.get_cam_order().iter().for_each(|bloc_id| {
						if self.blocs.contains(bloc_id) && !moving_bloc_childs.contains(bloc_id) {
							if let Some((new_bloc_container, new_ratio)) = widgets_manager
								.get::<Bloc>(bloc_id)
								.unwrap()
								.collide_container(bloc.get_base().rect, widgets_manager)
							{
								if new_ratio >= ratio {
									new_hovered_container =
										Some(Container { bloc_id: *bloc_id, bloc_container: new_bloc_container });
									ratio = new_ratio;
								}
							}
						}
					});
					if new_hovered_container != self.hovered_container {
						self.hovered_container = new_hovered_container;
					}
				}
				// Release the moving bloc
				else if bloc.get_base().state.is_released() {
					if let Some(container) = self.hovered_container.clone() {
						Bloc::set_parent_and_child(&container, &focused_widget, true, widgets_manager);
						// Update layout and childs positions
						let root_id = widgets_manager.get::<Bloc>(&container.bloc_id).unwrap().get_root(widgets_manager);
						update_layout(root_id, widgets_manager);
					}
					self.hovered_container = None;
				}
			}
		};

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Window>, text_drawer: &mut TextDrawer, camera: &Camera) {
		camera.draw_grid(canvas, text_drawer, Colors::LIGHT_GREY, true, false);
		/*
		if let Some(Container{ bloc_id, bloc_container }) = &self.hovered_container {
			match bloc_container {
				BlocContainer::Slot {nth_slot} => {
					let rect = widgets_manager.get::<NewBloc>(bloc_id).unwrap().slots[nth_slot].get_base().rect;
				},
				BlocContainer::Sequence { nth_sequence, place } => {
					todo!()
				}
			}
		}
		 */
	}
}

fn main() {
	let mut widgets_manager = WidgetsManager::default();
	let style = ButtonStyle::new(Colors::LIGHT_AZURE, Some(6.), 16.);
	widgets_manager.add_widget(
		Box::new(Button::new(Rect::new(100., 100., 140., 80.), style.clone(), "Test Bloc".to_string())),
		false,
	);
	widgets_manager.add_widget(
		Box::new(Button::new(Rect::new(300., 100., 140., 80.), style.clone(), "VarAssign Bloc".to_string())),
		false,
	);
	widgets_manager.add_widget(
		Box::new(Button::new(Rect::new(500., 100., 140., 80.), style, "IfElse Bloc".to_string())),
		false,
	);

	let resolution = Vector2::new(1280, 720);
	let ttf_context = sdl2::ttf::init().expect("SDL2 ttf could not be initialized");

	let mut app = PgSdl::init("Benday", resolution, Some(120), true, Colors::LIGHT_GREY, widgets_manager);

	let mut my_app = MyApp { hovered_container: None, blocs: Vec::new() };
	let font_path = std::path::PathBuf::from(format!("{}/{}", pg_sdl::text::FONT_PATH, pg_sdl::text::DEFAULT_FONT_NAME));
	for font in [(&font_path, 0, 30)] {
		let (path, from, to) = font;
		for size in from..=to {
			let font: sdl2::ttf::Font = ttf_context.load_font(path, size).unwrap();
			app.text_drawer.fonts.insert((path.clone(), size), font);
		}
	}

	app.run(&mut my_app);
}

fn update_layout(bloc_id: WidgetId, widgets_manager: &mut WidgetsManager) {
	let childs = widgets_manager.get::<Bloc>(&bloc_id).unwrap().get_recursive_bloc_childs(widgets_manager);
	childs.iter().for_each(|child_id| {
		let mut bloc_w = widgets_manager.remove(child_id).unwrap();
		let bloc = bloc_w.as_mut().downcast_mut::<Bloc>().unwrap();
		bloc.update_size(widgets_manager);
		widgets_manager.insert(*child_id, bloc_w);
	});
	childs.iter().rev().for_each(|child_id| {
		let mut bloc_w = widgets_manager.remove(child_id).unwrap();
		let bloc = bloc_w.as_mut().downcast_mut::<Bloc>().unwrap();
		bloc.update_position(widgets_manager);
		widgets_manager.insert(*child_id, bloc_w);
	});
}
