use pg_sdl::custom_rect::Rect;
use std::ops::{Add, Mul};
use sdl2::pixels::Color;
use models::ast::Id;
use pg_sdl::color::{color_to_hsv, hsv_color};

pub fn interpolate<T: Add<T, Output = T> + Mul<f64, Output = T>>(a: T, b: T, t: f64) -> T {
	a * (1. - t) + b * t
}

pub fn interpolate_color(start_color: Color, end_color: Color, t: f64) -> Color {
	let (start_h, start_s, start_v) = color_to_hsv(start_color);
	let (end_h, end_s, end_v) = color_to_hsv(end_color);
	hsv_color(
		interpolate(start_h as f64, end_h as f64, t) as u16,
		interpolate(start_s as f64, end_s as f64, t) as f32,
		interpolate(start_v as f64, end_v as f64, t) as f32,
	)
}

pub fn interpolate_rect(start_rect: Rect, end_rect: Rect, t: f64) -> Rect {
	let center = interpolate(start_rect.center().coords, end_rect.center().coords, t).into();
	let size = interpolate(start_rect.size, end_rect.size, t);
	Rect::from_center(center, size)
}

pub fn ease_in_out(t: f64) -> f64 {
	(-2. * t + 3.) * t * t
}

/// An Ease-In-Out function with a parameter (should be between 1 and 2) for the strength of the easing
pub fn parametric_ease_in_out(parameter: f64) -> Box<dyn Fn(f64) -> f64> {
	Box::new(move |t| (t.powf(parameter)) / (t.powf(parameter) + (1. - t).powf(parameter)))
}

pub fn ease_in(t: f64) -> f64 {
	t * t
}
pub fn ease_out(t: f64) -> f64 {
	(-t + 2.) * t
}

pub fn parametric_ease_in_back(parameter: f64) -> Box<dyn Fn(f64) -> f64> {
	Box::new(move |t| (parameter * t - parameter + 1.) * t * t)
}

pub fn ease_in_back(t: f64) -> f64 {
	(2. * t - 1.) * t * t
}
pub fn ease_out_back(t: f64) -> f64 {
	(2. * t * t - 5. * t + 4.) * t
}



#[derive(Clone, Debug)]
pub enum Animation {
	EnterBloc { rect_1: Rect, radius_1: f64, rect_2: Rect, radius_2: f64 },
	Return { rect_1: Rect, radius_1: f64, rect_2: Rect, radius_2: f64 },
	CheckValidity { rect: Rect, radius: f64, valid: bool },
	AssignVariable { rect: Rect, radius: f64, bloc_id: Id },
	Other { rect: Rect, radius: f64 },
}
