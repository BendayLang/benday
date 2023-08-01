// #![allow(dead_code, unused_variables, unused_imports)]
mod blocs;

use std::f64::consts::PI;

use crate::blocs::bloc::Bloc;
use crate::blocs::containers::Sequence;
use crate::blocs::{new_root_sequence_bloc, BlocContainer, BlocType, Container, BLOC_NAMES, RADIUS};
use blocs::as_ast_node::AsAstNode;
use nalgebra::{Point2, Vector2};
use pg_sdl::app::{App, PgSdl};
use pg_sdl::camera::Camera;
use pg_sdl::color::Colors;
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use pg_sdl::widgets::select::Select;
use pg_sdl::widgets::slider::{Slider, SliderStyle, SliderType};
use pg_sdl::widgets::switch::{Switch, SwitchStyle};
use pg_sdl::widgets::{Manager, Widget, WidgetId};
use runner::exectute::action::Action;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use std::time::Duration;

enum AppState {
	Idle,
	AddingBloc { widget_id: WidgetId, container: Container },
	Saving,
	Running { index: u16, console: runner::exectute::console::Console, actions: Vec<Action> },
}

pub struct BendayFront {
	state: AppState,
	/// Lists the widgets that are blocs
	blocs: Vec<WidgetId>,
	hovered_container: Option<Container>,
	rect: Option<Rect>,
	r1: Rect,
	r2: Rect,
	switch_id: WidgetId,
	t: Duration,
}

impl App for BendayFront {
	fn update(&mut self, delta: Duration, input: &Input, manager: &mut Manager, camera: &mut Camera) -> bool {
		let mut changed = false;
		
		{
			if input.mouse.right_button.is_pressed() {
				let mouse_position = input.mouse.position.cast();
				let point = camera.transform().inverse() * mouse_position;
				
				let mut container = None;
				for bloc_id in manager.get_cam_order().iter().rev() {
					if self.blocs.contains(bloc_id) {
						container = manager.get::<Bloc>(bloc_id).collide_point_container(point, manager);
						if container.is_some() {
							break;
						}
					}
				}
				
				if let Some(container) = container {
					let size = Vector2::new(100., 120.);
					let widget_id = manager.add_widget(
						Box::new(Select::new(
							Rect::from(mouse_position - Vector2::new(size.x * 0.5, 0.), size),
							Default::default(),
							BLOC_NAMES.iter().map(|&str| str.to_string()).collect(),
							"Name".to_string(),
						)),
						false,
					);
					manager.focus_widget(widget_id);
					
					self.state = AppState::AddingBloc { widget_id, container };
					changed = true;
				}
			}
		}
		
		// Run
		if manager.get::<Switch>(&2).is_pressed_on() {
			let root_sequence = manager.get::<Sequence>(&0);
			let ast = root_sequence.as_ast_node(manager);
			let (console, actions) = runner::exectute::runner(&ast);
			println!("Actions : {:?}", actions);
			for str in &console.stdout {
				println!("{str}");
			}
			manager.get_mut::<Slider>(&3).set_visible();
			self.state = AppState::Running { index: 0, console, actions };
		} else if manager.get::<Switch>(&2).is_pressed_off() {
			manager.get_mut::<Slider>(&3).set_invisible();
			self.state = AppState::Idle;
		}
		
		match &self.state {
			AppState::Idle => {
				if let Some(focused_widget) = &manager.focused_widget() {
					if self.blocs.contains(focused_widget) {
						// Take a bloc
						if manager.get::<Bloc>(focused_widget).get_base().state.is_pressed() {
							let parent = manager.get::<Bloc>(focused_widget).get_parent().clone();
							if let Some(container) = parent.clone() {
								Bloc::set_parent_and_child(&container, focused_widget, false, manager);
								Bloc::update_size_and_layout(manager);
							}
						}
						// Release the moving bloc
						else if manager.get::<Bloc>(focused_widget).get_base().state.is_released() {
							if let Some(container) = self.hovered_container.clone() {
								// Release in container
								Bloc::set_parent_and_child(&container, focused_widget, true, manager);
								Bloc::update_size_and_layout(manager);
							} else {
								// Release in void => destroy
								let childs_ids = manager.get::<Bloc>(focused_widget).get_recursive_widget_childs(manager);
								childs_ids.iter().for_each(|child_id| {
									manager.remove_widget(child_id);
									if let Some(index) = self.blocs.iter().position(|i| i == child_id) {
										self.blocs.remove(index);
									}
								});
							}
							self.hovered_container = None;
							self.rect = None;
						}
						// Update the (moving bloc) hovered container
						else if manager.get::<Bloc>(focused_widget).get_base().state.is_down() && !input.mouse.delta.is_empty()
						{
							// iter through all blocs to get the bloc with the biggest 'ratio'
							let moving_bloc_childs = manager.get::<Bloc>(focused_widget).get_recursive_childs(manager);
							let rect = manager.get::<Bloc>(focused_widget).get_base().rect.translated(-Bloc::SHADOW);
							let (mut new_hovered_container, mut ratio) = (None, 0.);
							
							manager.get_cam_order().iter().for_each(|bloc_id| {
								if self.blocs.contains(bloc_id) && !moving_bloc_childs.contains(bloc_id) {
									if let Some((new_container, new_ratio)) =
										manager.get::<Bloc>(bloc_id).collide_rect_container(rect, manager)
									{
										if new_ratio >= ratio {
											(new_hovered_container, ratio) = (Some(new_container), new_ratio);
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
											manager
												.get::<Sequence>(&bloc.sequences_ids[*nth_sequence])
												.get_gap_rect(*place, manager),
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
			}
			AppState::AddingBloc { widget_id, container } => {
				if !manager.get::<Select>(widget_id).is_focused() {
					let option = manager.get::<Select>(widget_id).get_option().to_string();
					if let Some(bloc_type) = BlocType::from_string(option) {
						let bloc = bloc_type.new_bloc(manager);
						self.blocs.push(bloc.add(container, manager));
					}
					manager.remove_widget(widget_id);
					self.state = AppState::Idle;
					changed = true;
				}
			}
			AppState::Saving => {}
			AppState::Running { .. } => {}
		}
		
		// test
		if manager.get::<Switch>(&self.switch_id).is_switched() {
			if self.t < Duration::new(1, 0) {
				self.t += delta;
				if self.t > Duration::new(1, 0) {
					self.t = Duration::new(1, 0);
					manager.get_mut::<Switch>(&self.switch_id).set_switched(false);
				}
				changed = true;
			}
		} else {
			if self.t > Duration::ZERO {
				if self.t < delta {
					self.t = Duration::ZERO;
					manager.get_mut::<Switch>(&self.switch_id).set_switched(true);
				} else { self.t -= delta; }
				changed = true;
			}
		}
		// test
		
		changed
	}
	
	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, manager: &Manager, camera: &Camera) {
		camera.draw_grid(canvas, text_drawer, Colors::LIGHT_GREY, true, false);
		
		manager.draw(canvas, text_drawer, camera);
		draw_text(canvas, None, text_drawer, Point2::new(250., 90.), "Run", 25., &TextStyle::default(), Align::Bottom);
		
		if let Some(rect) = self.rect {
			draw_rect(canvas, Some(camera), Colors::WHITE, rect);
		} else if let Some(focused_widget) = manager.focused_widget() {
			if self.blocs.contains(&focused_widget) {
				if manager.get::<Bloc>(&focused_widget).get_base().state.is_down() {
					let rect = manager.get::<Bloc>(&focused_widget).get_base().rect.translated(-Bloc::SHADOW);
					draw_rounded_rect(canvas, Some(camera), Colors::RED, rect, RADIUS);
				}
			}
		}
		
		// test
		let radius = 7.;
		draw_rounded_rect(canvas, None, Colors::BLACK, self.r1, radius);
		draw_rounded_rect(canvas, None, Colors::BLACK, self.r2, radius);
		let t = self.t.as_secs_f64();
		let t = ((((t - 0.5) * PI).sin() + 1.) * 0.5);
		let r_int = interpolate_rect(self.r1, self.r2, t);
		fill_rounded_rect(canvas, None, Colors::YELLOW, r_int, radius);
	}
}

fn main() {
	let mut manager = Manager::default();
	
	let mut bloc = new_root_sequence_bloc(&mut manager);
	bloc.get_base_mut().rect.size = (bloc.get_size)(&bloc, &manager);
	let root_id = manager.add_widget(Box::new(bloc), true);
	manager.get_widget_mut(&root_id).set_invisible();
	
	// Run switch
	let style = SwitchStyle::new(Colors::LIGHT_GREEN, Colors::LIGHT_RED);
	let rect = Rect::new(200., 100., 80., 40.);
	manager.add_widget(Box::new(Switch::new(rect, style)), false);
	
	// Debug slider
	let style = SliderStyle::new(Colors::LIGHT_RED, Colors::GREY);
	let rect = Rect::new(490., 118., 300., 24.);
	let slider_type = SliderType::Discrete { snap: 50, default_value: 0, display: Some(Box::new(|v| format!("{}", v))) };
	let slider_id = manager.add_widget(Box::new(Slider::new(rect, style, slider_type)), false);
	manager.get_mut::<Slider>(&slider_id).set_invisible();
	
	// test
	let style = SwitchStyle::new(Colors::LIGHT_GREEN, Colors::LIGHT_GREY);
	let rect = Rect::new(220., 200., 40., 20.);
	let switch_id = manager.add_widget(Box::new(Switch::new(rect, style)), false);
	// test
	
	let resolution = Vector2::new(1280, 720);
	let ttf_context = sdl2::ttf::init().expect("SDL2 ttf could not be initialized");
	
	let mut app = PgSdl::init("Benday", resolution, Some(120), true, Colors::LIGHT_GREY, manager);
	
	let mut my_app = BendayFront {
		state: AppState::Idle,
		blocs: vec![root_id],
		hovered_container: None,
		rect: None,
		r1: Rect::new(180., 300., 120., 80.),
		r2: Rect::new(400., 400., 80., 40.),
		switch_id,
		t: Duration::ZERO
	};
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


fn interpolate_rect(rect_1: Rect, rect_2: Rect, t: f64) -> Rect {
	let center = rect_1.center() * (1. - t) + rect_2.center().coords * t;
	let size = rect_1.size * (1. - t) + rect_2.size * t;
	Rect::from_center(center, size)
}
