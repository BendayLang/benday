use crate::camera::Camera;
use crate::color::Colors;
use crate::style::Align;
use crate::text::{TextDrawer, TextStyle};
use crate::vector2::Vector2Plus;
use nalgebra::{Matrix3, Point, Point2, Transform2, Vector2};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::{render::Canvas, video::Window};

/// Draws a one pixel wide line
pub fn draw_line(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, start: Point2<f64>, end: Point2<f64>) {
	if let Some(camera) = camera {
		let start = camera.transform * start;
		let end = camera.transform * end;
		DrawRenderer::line(canvas, start.x as i16, start.y as i16, end.x as i16, end.y as i16, color).unwrap();
	} else {
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
pub fn draw_rounded_rect(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>, radius: f64,
) {
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
pub fn fill_rounded_rect(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, size: Vector2<f64>, radius: f64,
) {
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

/// Draws the contour of an ellipse as seen by the camera
pub fn draw_ellipse(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radii: Vector2<f64>,
) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let radii = camera.transform * radii;
		let rect =
			Rect::new((position.x - radii.x) as i32, (position.y - radii.y) as i32, 2 * radii.x as u32, 2 * radii.y as u32);
		if camera.is_in_scope(rect) {
			DrawRenderer::ellipse(canvas, position.x as i16, position.y as i16, radii.x as i16, radii.y as i16, color).unwrap();
		};
	} else {
		let rect =
			Rect::new((position.x - radii.x) as i32, (position.y - radii.y) as i32, 2 * radii.x as u32, 2 * radii.y as u32);
		DrawRenderer::ellipse(canvas, position.x as i16, position.y as i16, radii.x as i16, radii.y as i16, color).unwrap();
	}
}
/// Draws a filled ellipse as seen by the camera
pub fn fill_ellipse(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radii: Vector2<f64>,
) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let radii = camera.transform * radii;
		let rect =
			Rect::new((position.x - radii.x) as i32, (position.y - radii.y) as i32, 2 * radii.x as u32, 2 * radii.y as u32);
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_ellipse(canvas, position.x as i16, position.y as i16, radii.x as i16, radii.y as i16, color)
				.unwrap();
		};
	} else {
		let rect =
			Rect::new((position.x - radii.x) as i32, (position.y - radii.y) as i32, 2 * radii.x as u32, 2 * radii.y as u32);
		DrawRenderer::filled_ellipse(canvas, position.x as i16, position.y as i16, radii.x as i16, radii.y as i16, color)
			.unwrap();
	}
}

/// Draws the contour of a circle as seen by the camera
pub fn draw_circle(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let radius = camera.scale() * radius;
		let rect = Rect::new((position.x - radius) as i32, (position.y - radius) as i32, 2 * radius as u32, 2 * radius as u32);
		if camera.is_in_scope(rect) {
			DrawRenderer::circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
		};
	} else {
		let rect = Rect::new((position.x - radius) as i32, (position.y - radius) as i32, 2 * radius as u32, 2 * radius as u32);
		DrawRenderer::circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
	}
}
/// Draws a filled circle as seen by the camera
pub fn fill_circle(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let radius = camera.scale() * radius;
		let rect = Rect::new((position.x - radius) as i32, (position.y - radius) as i32, 2 * radius as u32, 2 * radius as u32);
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
		};
	} else {
		let rect = Rect::new((position.x - radius) as i32, (position.y - radius) as i32, 2 * radius as u32, 2 * radius as u32);
		DrawRenderer::filled_circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
	}
}

/// Draws the contour of a polygon from its vertices as seen by the camera
pub fn draw_polygon(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, vertices: &Vec<Point2<f64>>) {
	if let Some(camera) = camera {
		let vertices: Vec<Point2<f64>> = vertices.iter().map(|point| camera.transform * point).collect();
	}
	let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
	let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
	let x_min = *vx.iter().min().unwrap() as i32;
	let y_min = *vy.iter().min().unwrap() as i32;
	let x_max = *vx.iter().max().unwrap() as i32;
	let y_max = *vy.iter().max().unwrap() as i32;
	let rect = Rect::new(x_min, y_min, (x_max - x_min) as u32, (y_max - y_min) as u32);
	if let Some(camera) = camera {
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
		}
	} else {
		DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
	}
}
/// Draws a filled polygon from its vertices as seen by the camera
pub fn fill_polygon(canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, vertices: &Vec<Point2<f64>>) {
	if let Some(camera) = camera {
		let vertices: Vec<Point2<f64>> = vertices.iter().map(|point| camera.transform * point).collect();
	}
	let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
	let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
	let x_min = *vx.iter().min().unwrap() as i32;
	let y_min = *vy.iter().min().unwrap() as i32;
	let x_max = *vx.iter().max().unwrap() as i32;
	let y_max = *vy.iter().max().unwrap() as i32;
	let rect = Rect::new(x_min, y_min, (x_max - x_min) as u32, (y_max - y_min) as u32);
	if let Some(camera) = camera {
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
		}
	} else {
		DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
	}
}

/// Draws an arrow as seen by the camera
pub fn draw_arrow(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, color: Color, start: Point2<f64>, end: Point2<f64>, width: f64,
) {
	if start == end {
		return;
	}
	let mut start = start;
	let mut end = end;
	let mut width = width;
	if let Some(camera) = camera {
		start = camera.transform * start;
		end = camera.transform * end;
		width = camera.scale() * width;
	}
	// TODO clean up
	let x_dir = end - start;
	let y_dir = x_dir.perpendicular() * width / 2.0;
	let transform =
		Transform2::from_matrix_unchecked(Matrix3::new(x_dir.x, y_dir.x, start.x, x_dir.y, y_dir.y, start.y, 0.0, 0.0, 1.0));

	let head_back: f64 = 1.0 - 3.0 * width / x_dir.norm();

	let mut points = Vec::from([
		Point2::new(head_back, -1.0),
		Point2::new(head_back, -3.0),
		Point2::new(1.0, 0.0),
		Point2::new(head_back, 3.0),
		Point2::new(head_back, 1.0),
	]);
	if x_dir.norm() > 3.0 * width {
		points.append(&mut Vec::from([Point2::new(0.0, 1.0), Point2::new(0.0, -1.0)]));
	}
	points.iter_mut().for_each(|v| *v = transform * *v);
	let points_x: Vec<i16> = points.iter().map(|v| v.x as i16).collect();
	let points_y: Vec<i16> = points.iter().map(|v| v.y as i16).collect();

	DrawRenderer::filled_polygon(canvas, &points_x, &points_y, color).unwrap();
	DrawRenderer::polygon(canvas, &points_x, &points_y, Colors::BLACK).unwrap();
}

/// Draws text as seen by the camera
pub fn draw_text(
	canvas: &mut Canvas<Window>, camera: Option<&Camera>, text_drawer: &TextDrawer, color: Color, position: Point2<f64>,
	font_size: f64, text: String, align: Align,
) {
	if text.is_empty() {
		return;
	}
	if let Some(camera) = camera {
		let position = camera.transform * position;
		let position = nalgebra::Point2::new(position.x as f64, position.y as f64);
		let text_style = &TextStyle { font_size: (camera.scale() * font_size) as u16, color, ..Default::default() };
		let (width, height) = text_drawer.text_size(text_style, &text);
		let rect = Rect::new(position.x as i32, position.y as i32, width, height);
		if camera.is_in_scope(rect) {
			text_drawer.draw(canvas, position, text_style, &text, align);
		}
	} else {
		let position = nalgebra::Point2::new(position.x as f64, position.y as f64);
		let text_style = &TextStyle { font_size: font_size as u16, color, ..Default::default() };
		let (width, height) = text_drawer.text_size(text_style, &text);
		let rect = Rect::new(position.x as i32, position.y as i32, width, height);
		text_drawer.draw(canvas, position, text_style, &text, align);
	}
}
