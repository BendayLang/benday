use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::{render::Canvas, video::Window};
use sdl2::gfx::primitives::DrawRenderer;
use crate::camera::Camera;


/// Draws a one pixel wide line
pub fn draw_line(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, start: Point2<f64>, end: Point2<f64>) {
	if let Some(camera) = camera{
		let start = camera.transform * start;
		let end = camera.transform * end;
		DrawRenderer::line(canvas, start.x as i16, start.y as i16, end.x as i16, end.y as i16, color).unwrap();
	}
	else {
		DrawRenderer::line(canvas, start.x as i16, start.y as i16, end.x as i16, end.y as i16, color).unwrap();
	}
}

/// Draws the contour of a rectangle as seen by the camera
pub fn draw_rect(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let size = camera.transform * size;
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		if camera.is_in_scope(rect) {
			canvas.set_draw_color(color);
			canvas.draw_rect(rect).unwrap();
		};
	} else {
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		canvas.set_draw_color(color);
		canvas.draw_rect(rect).unwrap();
	}
	
}
/// Draws a filled rectangle as seen by the camera
pub fn fill_rect(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let size = camera.transform * size;
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		if camera.is_in_scope(rect) {
			canvas.set_draw_color(color);
			canvas.fill_rect(rect).unwrap();
		};
	} else {
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		canvas.set_draw_color(color);
		canvas.fill_rect(rect).unwrap();
	}
	
}

/// Draws the contour of a rectangle as seen by the camera
pub fn draw_rounded_rect(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let size = camera.transform * size;
		let radius = (camera.scale() * radius) as u16;
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		if camera.is_in_scope(rect) {
			let (x1, x2) = (rect.left(), rect.right() - 1);
			let (y1, y2) = (rect.top(), rect.bottom() - 1);
			DrawRenderer::rounded_rectangle(canvas, x1 as i16, y1 as i16, x2 as i16, y2 as i16, radius as i16, color).unwrap();
		};
	} else {
		let (x1, x2) = (position.x as i16, (position.x + size.x) as i16 - 1);
		let (y1, y2) = (position.y as i16, (position.y + size.y) as i16 - 1);
		DrawRenderer::rounded_rectangle(canvas, x1, y1, x2, y2, radius as i16, color).unwrap();
	}
}
/// Draws a filled rectangle as seen by the camera
pub fn fill_rounded_rect(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let size = camera.transform * size;
		let radius = (camera.scale() * radius) as u16;
		let rect = Rect::new(position.x as i32, position.y as i32, size.x as u32, size.y as u32);
		if camera.is_in_scope(rect) {
			let (x1, x2) = (rect.left(), rect.right() - 1);
			let (y1, y2) = (rect.top(), rect.bottom() - 1);
			DrawRenderer::rounded_box(canvas, x1 as i16, y1 as i16, x2 as i16, y2 as i16, radius as i16, color).unwrap();
		};
	} else {
		let (x1, x2) = (position.x as i16, (position.x + size.x) as i16 - 1);
		let (y1, y2) = (position.y as i16, (position.y + size.y) as i16 - 1);
		DrawRenderer::rounded_box(canvas, x1, y1, x2, y2, radius as i16, color).unwrap();
	}
}
