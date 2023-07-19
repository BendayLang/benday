use nalgebra::{Complex, Point2, Similarity, Unit, Vector2};
use std::ops::Mul;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
	pub position: Point2<f64>,
	pub size: Vector2<f64>,
}

impl Rect {
	pub fn new(x: f64, y: f64, width: f64, height: f64) -> Rect {
		Self { position: Point2::new(x, y), size: Vector2::new(width, height) }
	}
	pub fn from(position: Point2<f64>, size: Vector2<f64>) -> Rect {
		Self { position, size }
	}
	pub fn into_rect(self) -> sdl2::rect::Rect {
		sdl2::rect::Rect::new(self.x() as i32, self.y() as i32, self.width() as u32, self.height() as u32)
	}

	pub fn x(&self) -> f64 {
		self.position.x
	}
	pub fn y(&self) -> f64 {
		self.position.y
	}
	pub fn width(&self) -> f64 {
		self.size.x
	}
	pub fn height(&self) -> f64 {
		self.size.y
	}

	pub fn top(&self) -> f64 {
		self.position.y + self.size.y
	}
	pub fn v_mid(&self) -> f64 {
		self.position.y + self.size.y * 0.5
	}
	pub fn bottom(&self) -> f64 {
		self.position.y
	}

	pub fn left(&self) -> f64 {
		self.position.x
	}
	pub fn h_mid(&self) -> f64 {
		self.position.x + self.size.x * 0.5
	}
	pub fn right(&self) -> f64 {
		self.position.x + self.size.x
	}

	pub fn top_left(&self) -> Point2<f64> {
		self.position + Vector2::new(self.size.x * 0., self.size.y)
	}
	pub fn mid_top(&self) -> Point2<f64> {
		self.position + Vector2::new(self.size.x * 0.5, self.size.y)
	}
	pub fn top_right(&self) -> Point2<f64> {
		self.position + self.size
	}
	pub fn mid_left(&self) -> Point2<f64> {
		self.position + Vector2::new(0., self.size.y * 0.5)
	}
	pub fn center(&self) -> Point2<f64> {
		self.position + self.size * 0.5
	}
	pub fn mid_right(&self) -> Point2<f64> {
		self.position + Vector2::new(self.size.x, self.size.y * 0.5)
	}
	pub fn bottom_left(&self) -> Point2<f64> {
		self.position
	}
	pub fn mid_bottom(&self) -> Point2<f64> {
		self.position + Vector2::new(self.size.x * 0.5, 0.)
	}
	pub fn bottom_right(&self) -> Point2<f64> {
		self.position + Vector2::new(self.size.x, 0.)
	}

	pub fn translate(&mut self, delta: Vector2<f64>) {
		self.position += delta;
	}
	/// Returns a new rect with the same size translated by delta
	pub fn translated(&self, delta: Vector2<f64>) -> Self {
		Self::from(self.position + delta, self.size)
	}
	/// Returns a new rect with the same center enlarged in every side by delta
	pub fn enlarged(&self, delta: f64) -> Self {
		let extension = Vector2::new(delta, delta);
		Self::from(self.position - extension, self.size + 2. * extension)
	}

	pub fn collide_point(&self, point: Point2<f64>) -> bool {
		self.left() < point.x && point.x < self.right() && self.bottom() < point.y && point.y < self.top()
	}
	pub fn collide_rect(&self, rect: Rect) -> bool {
		self.left() < rect.right() && rect.left() < self.right() && self.bottom() < rect.top() && rect.bottom() < self.top()
	}
}

impl Mul<Rect> for Similarity<f64, Unit<Complex<f64>>, 2> {
	type Output = Rect;

	fn mul(self, rhs: Rect) -> Self::Output {
		Rect::from(self * rhs.position, self * rhs.size)
	}
}
