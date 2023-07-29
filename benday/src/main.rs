#![allow(dead_code, unused_variables, unused_imports)]
mod blocs;

use std::time::Duration;

use crate::blocs::bloc::Bloc;
use crate::blocs::containers::Sequence;
use crate::blocs::{BlocContainer, BlocType, Container};
use as_any::Downcast;
use blocs::as_ast_node::AsAstNode;
use nalgebra::{Point2, Vector2};
use sdl2::pixels::PixelFormatEnum;
use pg_sdl::app::{App, PgSdl};
use pg_sdl::camera::Camera;
use pg_sdl::color::{hsv_color, Colors};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::draw_rect;
use pg_sdl::text::TextDrawer;
use pg_sdl::widgets::blank_box::{BlankBox, BlankBoxStyle};
use pg_sdl::widgets::select::{Select, SelectStyle};
use pg_sdl::widgets::switch::Switch;
use pg_sdl::widgets::{
	button::{Button, ButtonStyle},
	Manager, Widget, WidgetId,
};
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;

pub struct MyApp {
	/// Lists the widgets that are blocs
	blocs: Vec<WidgetId>,
	hovered_container: Option<Container>,
	rect: Option<Rect>,
}

impl App for MyApp {
	fn update(&mut self, _delta: Duration, input: &Input, manager: &mut Manager, camera: &mut Camera) -> bool {
		let mut changed = false;
		changed |= camera.update(input, manager.focused_widget().is_some());

		// Add new bloc
		let position = Point2::new(8., 10.) * self.blocs.len() as f64;
		if manager.get::<Button>(&6).is_pressed() {
			self.blocs.push(Bloc::add(position, BlocType::FunctionCall, manager));
		} else if manager.get::<Button>(&7).is_pressed() {
			self.blocs.push(Bloc::add(position, BlocType::VariableAssignment, manager));
		} else if manager.get::<Button>(&8).is_pressed() {
			self.blocs.push(Bloc::add(position, BlocType::IfElse, manager));
		}

		// Run
		if manager.get::<Button>(&9).is_pressed() {
			let root_sequence = manager.get::<Sequence>(&0);
			let ast = root_sequence.as_ast_node(manager);
			let (return_value, stdout, variables, actions) = runner::exectute::runner(&ast);
			println!("Actions : {:?}", actions);
			for str in stdout {
				println!("{str}");
			}
		}

		if let Some(focused_widget) = manager.focused_widget() {
			if self.blocs.contains(&focused_widget) {
				// Take a bloc
				if manager.get::<Bloc>(&focused_widget).get_base().state.is_pressed() {
					let parent = manager.get::<Bloc>(&focused_widget).get_parent().clone();
					if let Some(container) = parent.clone() {
						Bloc::set_parent_and_child(&container, &focused_widget, false, manager);
						Bloc::update_size_and_layout(manager);
					}
				}
				// Release the moving bloc
				else if manager.get::<Bloc>(&focused_widget).get_base().state.is_released() {
					if let Some(container) = self.hovered_container.clone() {
						Bloc::set_parent_and_child(&container, &focused_widget, true, manager);
						Bloc::update_size_and_layout(manager);
					}
					self.hovered_container = None;
					self.rect = None;
				}
				// Update the (moving bloc) hovered container
				else if manager.get::<Bloc>(&focused_widget).get_base().state.is_down() && !input.mouse.delta.is_empty() {
					// iter through all blocs to get the bloc with the biggest 'ratio'
					let moving_bloc_childs = manager.get::<Bloc>(&focused_widget).get_recursive_childs(manager);
					let (mut new_hovered_container, mut ratio) = (None, 0.);

					manager.get_cam_order().iter().for_each(|bloc_id| {
						if self.blocs.contains(bloc_id) && !moving_bloc_childs.contains(bloc_id) {
							if let Some((new_bloc_container, new_ratio)) = manager
								.get::<Bloc>(bloc_id)
								.collide_container(manager.get::<Bloc>(&focused_widget).get_base().rect, manager)
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
						// spé
						self.rect = if let Some(Container { bloc_id, bloc_container }) = &self.hovered_container {
							let bloc = manager.get::<Bloc>(bloc_id);
							match bloc_container {
								BlocContainer::Slot { nth_slot } => Some(get_base_!(bloc.slots[*nth_slot], manager).rect),
								BlocContainer::Sequence { nth_sequence, place } => Some(
									manager.get::<Sequence>(&bloc.sequences_ids[*nth_sequence]).get_gap_rect(*place, manager),
								),
							}
						} else {
							None
						};
						// spé
					}
				}
			}
		};

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, manager: &Manager, camera: &Camera) {
		camera.draw_grid(canvas, text_drawer, Colors::LIGHT_GREY, true, false);

		manager.draw(canvas, text_drawer, camera);

		if let Some(rect) = self.rect {
			draw_rect(canvas, Some(camera), Colors::WHITE, rect);
		}
		
		let rect1 = Rect::new(700.,400.,100.,60.);
		let rect2 = Rect::new(750.,420.,100.,60.);
		draw_rect(canvas, None, Colors::YELLOW, rect1);
		draw_rect(canvas, None, Colors::DARK_RED, rect2);
		let mut surf = Surface::new(rect1.width() as u32, rect1.height() as u32, PixelFormatEnum::RGBA32).unwrap();
		
		// let canvas = surf.into_canvas().unwrap();
		
		surf.fill_rect(rect2.translated(-rect1.position.coords).into_rect(), Colors::GREEN).unwrap();
		
		let texture = text_drawer.texture_creator.create_texture_from_surface(surf).unwrap();
		canvas.copy(&texture, None, Some(rect1.into_rect())).unwrap();
	}
}

fn main() {
	let mut manager = Manager::default();

	let root_id = Bloc::add(Point2::origin(), BlocType::RootSequence, &mut manager);
	manager.get_widget_mut(&root_id).set_invisible();

	let style = ButtonStyle::new(Colors::LIGHT_AZURE, Some(6.), 16.);
	manager.add_widget(Box::new(Button::new(Rect::new(100., 100., 140., 80.), style.clone(), "Fn()".to_string())), false);
	manager
		.add_widget(Box::new(Button::new(Rect::new(300., 100., 140., 80.), style.clone(), "VarAssign Bloc".to_string())), false);
	manager.add_widget(Box::new(Button::new(Rect::new(500., 100., 140., 80.), style.clone(), "IfElse Bloc".to_string())), false);
	manager.add_widget(Box::new(Button::new(Rect::new(700., 100., 140., 80.), style, "RUN".to_string())), false);
	let names = vec!["Alice", "Bob", "Charlie", "David", "Emilie", "Florence", "Gary", "Hervé", "Inès"];
	manager.add_widget(
		Box::new(Select::new(
			Rect::new(100., 250., 100., 120.),
			Default::default(),
			names.iter().map(|&str| str.to_string()).collect(),
			"Name".to_string(),
		)),
		false,
	);

	let resolution = Vector2::new(1280, 720);
	let ttf_context = sdl2::ttf::init().expect("SDL2 ttf could not be initialized");

	let mut app = PgSdl::init("Benday", resolution, Some(120), true, Colors::LIGHT_GREY, manager);

	let mut my_app = MyApp { blocs: vec![root_id], hovered_container: None, rect: None };
	let font_path = std::path::PathBuf::from(format!("{}/{}", pg_sdl::text::FONT_PATH, pg_sdl::text::DEFAULT_FONT_NAME));
	for font in [(&font_path, 0, 45)] {
		let (path, from, to) = font;
		for size in from..=to {
			let font: sdl2::ttf::Font = ttf_context.load_font(path, size).unwrap();
			app.text_drawer.fonts.insert((path.clone(), size), font);
		}
	}

	app.run(&mut my_app);
}
