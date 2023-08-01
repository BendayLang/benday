use crate::camera::Camera;
use crate::color::Colors;
use crate::custom_rect::Rect;
use crate::style::Align;
use crate::text::{FontSize, TextDrawer, TextStyle};
use crate::vector2::Vector2Plus;
use nalgebra::{Matrix3, Point2, Transform2, Vector2};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use sdl2::{render::Canvas, video::Window};

/// Draws a one pixel wide line that links points start to end
pub fn draw_line(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, start: Point2<f64>, end: Point2<f64>) {
	if let Some(camera) = camera {
		let start = camera.transform() * start;
		let end = camera.transform() * end;
		DrawRenderer::line(canvas, start.x as i16, start.y as i16, end.x as i16, end.y as i16, color).unwrap();
	} else {
		DrawRenderer::line(canvas, start.x as i16, start.y as i16, end.x as i16, end.y as i16, color).unwrap();
	}
}

pub fn draw_hline(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, x1: f64, x2: f64, y: f64) {
	if let Some(camera) = camera {
		// TODO find a better way
		let x1 = (camera.transform() * Point2::new(x1, 0.)).x;
		let x2 = (camera.transform() * Point2::new(x2, 0.)).x;
		let y = (camera.transform() * Point2::new(0., y)).y;
		DrawRenderer::hline(canvas, x1 as i16, x2 as i16, y as i16, color).unwrap();
	} else {
		DrawRenderer::hline(canvas, x1 as i16, x2 as i16, y as i16, color).unwrap();
	}
}
pub fn draw_vline(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, x: f64, y1: f64, y2: f64) {
	if let Some(camera) = camera {
		let x = (camera.transform() * Point2::new(x, 0.)).x;
		let y1 = (camera.transform() * Point2::new(0., y1)).y;
		let y2 = (camera.transform() * Point2::new(0., y2)).y;

		DrawRenderer::vline(canvas, x as i16, y1 as i16, y2 as i16, color).unwrap();
	} else {
		DrawRenderer::vline(canvas, x as i16, y1 as i16, y2 as i16, color).unwrap();
	}
}

pub fn draw_rect(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		if camera.is_in_scope(rect) {
			canvas.set_draw_color(color);
			canvas.draw_rect(rect.into_rect()).unwrap();
		};
	} else {
		canvas.set_draw_color(color);
		canvas.draw_rect(rect.into_rect()).unwrap();
	}
}
pub fn fill_rect(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		if camera.is_in_scope(rect) {
			canvas.set_draw_color(color);
			canvas.fill_rect(rect.into_rect()).unwrap();
		};
	} else {
		canvas.set_draw_color(color);
		canvas.fill_rect(rect.into_rect()).unwrap();
	}
}

pub fn draw_rounded_rect(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect, radius: f64) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		let radius = (camera.scale() * radius) as i16;
		if camera.is_in_scope(rect) {
			let (x1, x2) = (rect.left() as i16, rect.right() as i16 - 1);
			let (y1, y2) = (rect.top() as i16, rect.bottom() as i16 - 1);
			DrawRenderer::rounded_rectangle(canvas, x1, y1, x2, y2, radius, color).unwrap();
		};
	} else {
		let (x1, x2) = (rect.left() as i16, rect.right() as i16 - 1);
		let (y1, y2) = (rect.top() as i16, rect.bottom() as i16 - 1);
		DrawRenderer::rounded_rectangle(canvas, x1, y1, x2, y2, radius as i16, color).unwrap();
	}
}
pub fn fill_rounded_rect(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect, radius: f64) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		let radius = (camera.scale() * radius) as i16;
		if camera.is_in_scope(rect) {
			let (x1, x2) = (rect.left() as i16, rect.right() as i16 - 1);
			let (y1, y2) = (rect.top() as i16, rect.bottom() as i16 - 1);
			DrawRenderer::rounded_box(canvas, x1, y1, x2, y2, radius, color).expect("Negative Radius");
		};
	} else {
		let (x1, x2) = (rect.left() as i16, rect.right() as i16 - 1);
		let (y1, y2) = (rect.top() as i16, rect.bottom() as i16 - 1);
		DrawRenderer::rounded_box(canvas, x1, y1, x2, y2, radius as i16, color).expect("Negative Radius");
	}
}

pub fn draw_ellipse(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		if camera.is_in_scope(Rect::from(rect.position - rect.size, 2.0 * rect.size)) {
			DrawRenderer::ellipse(canvas, rect.x() as i16, rect.y() as i16, rect.width() as i16, rect.height() as i16, color)
				.unwrap();
		};
	} else {
		DrawRenderer::ellipse(canvas, rect.x() as i16, rect.y() as i16, rect.width() as i16, rect.height() as i16, color)
			.unwrap();
	}
}
pub fn fill_ellipse(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, rect: Rect) {
	if let Some(camera) = camera {
		let rect = camera.transform() * rect;
		if camera.is_in_scope(Rect::from(rect.position - rect.size, 2.0 * rect.size)) {
			DrawRenderer::filled_ellipse(
				canvas,
				rect.x() as i16,
				rect.y() as i16,
				rect.width() as i16,
				rect.height() as i16,
				color,
			)
			.unwrap();
		};
	} else {
		DrawRenderer::filled_ellipse(canvas, rect.x() as i16, rect.y() as i16, rect.width() as i16, rect.height() as i16, color)
			.unwrap();
	}
}

pub fn draw_circle(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform() * position;
		let radius = camera.scale() * radius;
		let rect = Rect::new(position.x - radius, position.y - radius, 2.0 * radius, 2.0 * radius);
		if camera.is_in_scope(rect) {
			DrawRenderer::circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
		};
	} else {
		DrawRenderer::circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
	}
}
pub fn fill_circle(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, position: Point2<f64>, radius: f64) {
	if let Some(camera) = camera {
		let position = camera.transform() * position;
		let radius = camera.scale() * radius;
		let rect = Rect::new(position.x - radius, position.y - radius, 2.0 * radius, 2.0 * radius);
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
		};
	} else {
		DrawRenderer::filled_circle(canvas, position.x as i16, position.y as i16, radius as i16, color).unwrap()
	}
}

pub fn draw_polygon(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, vertices: &[Point2<f64>]) {
	if let Some(camera) = camera {
		let vertices = vertices.iter().map(|point| camera.transform() * point).collect::<Vec<Point2<f64>>>();
		let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
		let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
		let x_min = *vx.iter().min().unwrap() as f64;
		let y_min = *vy.iter().min().unwrap() as f64;
		let x_max = *vx.iter().max().unwrap() as f64;
		let y_max = *vy.iter().max().unwrap() as f64;
		let rect = Rect::new(x_min, y_min, x_max - x_min, y_max - y_min);
		if camera.is_in_scope(rect) {
			DrawRenderer::polygon(canvas, &vx, &vy, color).unwrap();
		}
	} else {
		let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
		let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
		DrawRenderer::polygon(canvas, &vx, &vy, color).unwrap();
	}
}
pub fn fill_polygon(canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, vertices: &[Point2<f64>]) {
	if let Some(camera) = camera {
		let vertices = vertices.iter().map(|point| camera.transform() * point).collect::<Vec<Point2<f64>>>();
		let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
		let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
		let x_min = *vx.iter().min().unwrap() as f64;
		let y_min = *vy.iter().min().unwrap() as f64;
		let x_max = *vx.iter().max().unwrap() as f64;
		let y_max = *vy.iter().max().unwrap() as f64;
		let rect = Rect::new(x_min, y_min, x_max - x_min, y_max - y_min);
		if camera.is_in_scope(rect) {
			DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
		}
	} else {
		let vx: Vec<i16> = vertices.iter().map(|point| point.x as i16).collect();
		let vy: Vec<i16> = vertices.iter().map(|point| point.y as i16).collect();
		DrawRenderer::filled_polygon(canvas, &vx, &vy, color).unwrap();
	}
}

pub fn draw_arrow(
	canvas: &mut Canvas<Surface>, camera: Option<&Camera>, color: Color, start: Point2<f64>, end: Point2<f64>, width: f64,
) {
	if start == end {
		return;
	}
	// TODO clean up
	let x_dir = end - start;
	let y_dir = x_dir.perpendicular() * width / 2.0;
	let transform =
		Transform2::from_matrix_unchecked(Matrix3::new(x_dir.x, y_dir.x, start.x, x_dir.y, y_dir.y, start.y, 0.0, 0.0, 1.0));

	let head_back: f64 = 1.0 - 3.0 * width / x_dir.norm();

	let mut vertices = Vec::from([
		Point2::new(head_back, -1.0),
		Point2::new(head_back, -3.0),
		Point2::new(1.0, 0.0),
		Point2::new(head_back, 3.0),
		Point2::new(head_back, 1.0),
	]);
	if x_dir.norm() > 3.0 * width {
		vertices.append(&mut Vec::from([Point2::new(0.0, 1.0), Point2::new(0.0, -1.0)]));
	}
	vertices.iter_mut().for_each(|v| *v = transform * *v);

	fill_polygon(canvas, camera, color, &vertices);
	draw_polygon(canvas, camera, Colors::BLACK, &vertices);
}

/// Returns the text size in it's space
pub fn get_text_size(
	camera: Option<&Camera>, text_drawer: &mut TextDrawer, text: &str, font_size: f64, style: &TextStyle,
) -> Vector2<f64> {
	if let Some(camera) = camera {
		text_drawer.size_of::<u32>(text, (camera.scale() * font_size) as FontSize, style).cast() / camera.scale()
	} else {
		text_drawer.size_of::<u32>(text, font_size as FontSize, style).cast()
	}
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text(
	canvas: &mut Canvas<Surface>, camera: Option<&Camera>, text_drawer: &mut TextDrawer, position: Point2<f64>, text: &str,
	font_size: f64, style: &TextStyle, align: Align,
) {
	if text.is_empty() {
		return;
	}
	if let Some(camera) = camera {
		let position = camera.transform() * position;
		let font_size = camera.scale() * font_size;
		let size = text_drawer.size_of::<u32>(text, font_size as FontSize, style);
		let rect = Rect::from(position, size.cast());
		if camera.is_in_scope(rect) {
			text_drawer.draw(canvas, position, text, font_size as FontSize, style, align);
		}
	} else {
		text_drawer.draw(canvas, position, text, font_size as FontSize, style, align);
	}
}
