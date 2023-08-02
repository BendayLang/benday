// #![allow(dead_code, unused_variables, unused_imports)]
mod animation;
mod blocs;

use crate::animation::{ease_in_out, interpolate, interpolate_rect, Animation, interpolate_color, parametric_ease_in_back, ease_in};
use crate::blocs::bloc::Bloc;
use crate::blocs::containers::Sequence;
use crate::blocs::{new_root_sequence_bloc, BlocContainer, BlocType, Container, BLOC_NAMES, RADIUS};
use blocs::as_ast_node::AsAstNode;
use nalgebra::{Point2, Vector2};
use pg_sdl::app::{App, PgSdl};
use pg_sdl::camera::Camera;
use pg_sdl::color::{Colors, with_alpha};
use pg_sdl::custom_rect::Rect;
use pg_sdl::input::Input;
use pg_sdl::primitives::{draw_rect, draw_rounded_rect, draw_text, fill_rounded_rect};
use pg_sdl::style::Align;
use pg_sdl::text::{TextDrawer, TextStyle};
use pg_sdl::widgets::select::Select;
use pg_sdl::widgets::slider::{Slider, SliderStyle, SliderType};
use pg_sdl::widgets::switch::{Switch, SwitchStyle};
use pg_sdl::widgets::{Manager, Widget, WidgetId};
use runner::exectute::action::ActionType;
use runner::exectute::console::Console;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;
use std::cmp::Ordering;
use std::time::Duration;
use sdl2::pixels::Color;
use pg_sdl::widgets::text_input::TextInput;

const ANIM_TIME: Duration = Duration::from_millis(600);
const RUN_COLOR: Color = Colors::LIGHT_YELLOW;

#[allow(dead_code)]
#[derive(Clone)]
enum AppState {
	Idle,
	AddingBloc { widget_id: WidgetId, container: Container },
	Saving,
	Running { animation_timer: Duration, console: Console, animations: Vec<Animation> },
}

pub struct BendayFront {
	state: AppState,
	/// Lists the widgets that are blocs
	blocs: Vec<WidgetId>,
	hovered_container: Option<Container>,
	rect: Option<Rect>,
	debug_slider_id: WidgetId,
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
							Some(4.),
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

			let animations = actions.iter().map(|action| {
				let node_id = action.get_id();
				let node_rect = manager.get_widget(node_id).get_base().rect;
				let node_radius = manager.get_widget(node_id).get_base().radius.unwrap_or_else(|| 1.);
				match action.get_type() {
					ActionType::Entered { from } => {
						let rect_1 = manager.get_widget(from).get_base().rect;
						let radius_1 = manager.get_widget(from).get_base().radius.unwrap_or_else(|| 1.);
						Animation::EnterBloc { rect_1, rect_2: node_rect, radius_1, radius_2: node_radius }
					}
					ActionType::Return(_) => {
						if let Some(parent_id) = &manager.get_widget(node_id).get_base().parent_id {
							let rect_2 = manager.get_widget(parent_id).get_base().rect;
							let radius_2 = manager.get_widget(parent_id).get_base().radius.unwrap_or_else(|| 1.);
							Animation::Return { rect_1: node_rect, rect_2, radius_1: node_radius, radius_2 }
						} else {
							Animation::Other { rect: node_rect, radius: node_radius }
						}
					}
					ActionType::AssignVariable { .. } => {
						Animation::AssignVariable { rect: node_rect, radius: node_radius, bloc_id: *node_id }
					}
					ActionType::CheckVarNameValidity(result) => {
						Animation::CheckValidity { rect: node_rect, radius: node_radius, valid: result.is_ok() }
					}
					_ => Animation::Other { rect: node_rect, radius: node_radius },
				}
			}).collect::<Vec<Animation>>();

			manager.get_mut::<Slider>(&3).reset_value();
			manager.get_mut::<Slider>(&3).change_snap(animations.len() as u32);
			manager.get_mut::<Slider>(&3).set_visible();
			self.state = AppState::Running { animation_timer: Duration::ZERO, console, animations };
		} else if manager.get::<Switch>(&2).is_pressed_off() {
			manager.get_mut::<Slider>(&3).set_invisible();
			self.state = AppState::Idle;
		}

		match self.state.clone() {
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
				if !manager.get::<Select>(&widget_id).is_focused() {
					let option = manager.get::<Select>(&widget_id).get_option().to_string();
					if let Some(bloc_type) = BlocType::from_string(option) {
						let bloc = bloc_type.new_bloc(manager);
						self.blocs.push(bloc.add(&container, manager));
					}
					manager.remove_widget(&widget_id);
					self.state = AppState::Idle;
					changed = true;
				}
			}
			AppState::Saving => {}
			AppState::Running { animation_timer, console, animations } => {
				let timer_target = ANIM_TIME * manager.get::<Slider>(&self.debug_slider_id).get_value() as u32;
				
				let mut timer = animation_timer;
				match animation_timer.cmp(&timer_target) {
					Ordering::Less => {
						if timer_target - timer > 3 * ANIM_TIME {
							timer = timer_target - 3 * ANIM_TIME;
						}
						self.state = AppState::Running {
							animation_timer: if timer + delta > timer_target { timer_target } else { timer + delta },
							console,
							animations,
						};
						changed = true;
					}
					Ordering::Equal => (),
					Ordering::Greater => {
						if timer - timer_target > 3 * ANIM_TIME {
							timer = timer_target + 3 * ANIM_TIME;
						}
						self.state = AppState::Running {
							animation_timer: if timer < timer_target + delta { timer_target } else { timer - delta },
							console,
							animations,
						};
						changed = true;
					}
				}
			}
		}

		changed
	}

	fn draw(&self, canvas: &mut Canvas<Surface>, text_drawer: &mut TextDrawer, manager: &Manager, camera: &Camera) {
		camera.draw_grid(canvas, text_drawer, Colors::LIGHT_GREY, true, false);

		manager.draw(canvas, text_drawer, camera);
		draw_text(canvas, None, text_drawer, Point2::new(250., 90.), "Run", 22., &TextStyle::default(), Align::Bottom);

		if let Some(rect) = self.rect {
			draw_rect(canvas, Some(camera), Colors::WHITE, rect);
		} else if let Some(focused_widget) = manager.focused_widget() {
			if self.blocs.contains(&focused_widget) && manager.get::<Bloc>(&focused_widget).get_base().state.is_down() {
				let rect = manager.get::<Bloc>(&focused_widget).get_base().rect.translated(-Bloc::SHADOW);
				draw_rounded_rect(canvas, Some(camera), Colors::RED, rect, RADIUS);
			}
		}

		match &self.state {
			AppState::Running { animation_timer, animations, .. } => {
				let timer = animation_timer.as_secs_f64() / ANIM_TIME.as_secs_f64();
				let (t, animation_index) = if timer == 0. {
					(0., 0)
				} else {
					(timer - timer.ceil() + 1., timer.ceil() as usize - 1)
				};
				let text = &format!("t ({}) index ({})", t, animation_index);
				let position = Point2::new(200., 250.);
				draw_text(canvas, None, text_drawer, position, text, 25., &TextStyle::default(), Align::Center);
				
				let t = ease_in_out(t);

				match animations[animation_index] {
					Animation::EnterBloc { rect_1, radius_1, rect_2, radius_2 } => {
						let rect = interpolate_rect(rect_1, rect_2, ease_in_out(t));
						let radius = interpolate(radius_1, radius_2, ease_in_out(t));
						canvas.set_blend_mode(BlendMode::Blend);
						fill_rounded_rect(canvas, Some(camera), with_alpha(RUN_COLOR, 63), rect, radius);
						draw_rounded_rect(canvas, Some(camera), RUN_COLOR, rect, radius);
					}
					Animation::Return { rect_1, radius_1, rect_2, radius_2 } => {
						let rect = interpolate_rect(rect_1, rect_2, ease_in_out(t));
						let radius = interpolate(radius_1, radius_2, ease_in_out(t));
						canvas.set_blend_mode(BlendMode::Blend);
						fill_rounded_rect(canvas, Some(camera), with_alpha(Colors::LIGHT_BLUE, 63), rect, radius);
						draw_rounded_rect(canvas, Some(camera), Colors::LIGHT_BLUE, rect, radius);
					}
					Animation::AssignVariable { rect, radius, bloc_id } => {
						// canvas.set_blend_mode(BlendMode::Blend);
						// fill_rounded_rect(canvas, Some(camera), with_alpha(RUN_COLOR, 63), rect, radius);
						let ease_in_back = parametric_ease_in_back(10.);
						draw_rounded_rect(canvas, Some(camera), RUN_COLOR, rect, radius);
						let name_widget = manager.get::<TextInput>(manager.get::<Bloc>(&bloc_id).widgets[0].get_id());
						let value_widget = manager.get::<TextInput>(manager.get::<Bloc>(&bloc_id).slots[0].get_id());
						let position = interpolate(
							value_widget.get_base().rect.mid_left().coords + Vector2::new(5., 0.),
							name_widget.get_base().rect.mid_left().coords + Vector2::new(5., 0.),
							ease_in(t)
						).into();
						let font_size = interpolate(value_widget.get_style().font_size, 0., ease_in_back(t));
						let text = value_widget.get_text();
						draw_text(canvas, Some(camera), text_drawer, position, text, font_size, &TextStyle::default(), Align::Left);
					}
					Animation::CheckValidity { rect, radius, valid } => {
						let start_color = RUN_COLOR;
						let target_color = if valid { Colors::GREEN } else { Colors::RED };
						let color = interpolate_color(start_color, target_color, t);
						canvas.set_blend_mode(BlendMode::Blend);
						fill_rounded_rect(canvas, Some(camera), with_alpha(color, 63), rect, radius);
						draw_rounded_rect(canvas, Some(camera), color, rect, radius);
					}
					Animation::Other { rect, radius } => {
						canvas.set_blend_mode(BlendMode::Blend);
						fill_rounded_rect(canvas, Some(camera), with_alpha(RUN_COLOR, 63), rect, radius);
						draw_rounded_rect(canvas, Some(camera), RUN_COLOR, rect, radius);
					}
				}
			}
			_ => (),
		}
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
	let slider_type = SliderType::Discrete { snap: 2, default_value: 0, display: Some(Box::new(|v| format!("{}", v))) };
	let debug_slider_id = manager.add_widget(Box::new(Slider::new(rect, style, slider_type)), false);
	manager.get_mut::<Slider>(&debug_slider_id).set_invisible();

	let resolution = Vector2::new(1280, 720);

	let mut app = PgSdl::init("Benday", resolution, Some(120), true, Colors::LIGHT_GREY, manager);

	let mut my_app = BendayFront {
		state: AppState::Idle,
		blocs: vec![root_id],
		hovered_container: None,
		rect: None,
		debug_slider_id,
	};

	app.run(&mut my_app);
}
