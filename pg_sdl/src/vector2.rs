use nalgebra::{Matrix2, Rotation2, Vector2};

const ROTATION90: Rotation2<f64> = Rotation2::from_matrix_unchecked(Matrix2::new(0.0, -1.0, 1.0, 0.0));

pub trait Vector2Plus {
	fn new_unitary(angle: f64) -> Self;
	fn new_polar(length: f64, angle: f64) -> Self;
	fn get_angle(&self) -> f64;
	fn perpendicular(&self) -> Self;
}

impl Vector2Plus for Vector2<f64> {
	/// Returns a new unitary vector with length 1 and a given angle in radians
	fn new_unitary(angle: f64) -> Self {
		let (sin, cos) = angle.sin_cos();
		Vector2::new(cos, sin)
	}
	/// Returns a new vector with a given length and angle in radians
	fn new_polar(length: f64, angle: f64) -> Self {
		Self::new_unitary(angle) * length
	}
	/// Returns the angle of the vector in radians form the x axis (0 - 2π)
	fn get_angle(&self) -> f64 {
		self.y.atan2(self.x)
	}
	/// Returns the normalized perpendicular vector (rotated 90° in the trigonometric direction)
	fn perpendicular(&self) -> Self {
		ROTATION90 * self.normalize()
	}
}
