use std::ops::{Add, Mul};
use pg_sdl::custom_rect::Rect;


fn interpolate<T: Add<T, Output = T> + Mul<f64, Output = T>>(a: T, b: T, t: f64) -> T {
	a * (1. - t) + b * t
}

pub fn interpolate_rect(rect_1: Rect, rect_2: Rect, t: f64) -> Rect {
	let center = interpolate(rect_1.center().coords, rect_2.center().coords, t).into();
	let size = interpolate(rect_1.size, rect_2.size, t);
	Rect::from_center(center, size)
}

/// The basic Ease-In-Out function
pub fn ease_in_out(t: f64) -> f64 {
	-2. * t.powf(3.) + 3. * t.powf(2.)
}

/// An Ease-In-Out function with a parameter (should be between 1 and 2)
pub fn parametric_ease_in_out(parameter: f64) -> Box<dyn Fn(f64) -> f64> {
	Box::new(move |t| (t.powf(parameter)) / (t.powf(parameter) + (1. - t).powf(parameter)))
}

pub enum Animation {
	ChangeBloc { rect_1: Rect, rect_2: Rect },
	Other,
}