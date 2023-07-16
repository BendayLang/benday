mod text;

use crate::style::Align;
use nalgebra::{Point2, Vector2};
use sdl2::render::TextureQuery;
use sdl2::{render::Canvas, video::Window};
use std::path::Path;
pub use text::TextStyle;

/*
// pub fn get_text_<'a>(text_style: &TextStyle, text: &str) -> (u32, u32) {
//     let TextStyle {
//         // text,
//         font_name: font_path,
//         font_size,
//         font_style,
//         color,
//     } = text_style;
//
//     let ttf_context = sdl2::ttf::init().unwrap();
//
//     let mut font: sdl2::ttf::Font = ttf_context
//         .load_font(Path::new(&font_path), *font_size)
//         .unwrap();
//
//     font.set_style(*font_style);
//
//     // render a surface, and convert it to a texture bound to the canvas
//     let surface = font
//         .render(text)
//         .blended(*color)
//         .map_err(|e| e.to_string())
//         .unwrap();
//
//     let canvas = sdl2::init()
//         .unwrap()
//         .video()
//         .unwrap()
//         .window("", 0, 0)
//         .build()
//         .unwrap()
//         .into_canvas()
//         .build()
//         .unwrap();
//     let texture_creator = canvas.texture_creator();
//
//     let TextureQuery { height, width, .. } = texture_creator
//         .create_texture_from_surface(&surface)
//         .expect("")
//         .query();
//     return (height, width);
// }
 */

pub struct TextDrawer {
	pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
	ttf_context: sdl2::ttf::Sdl2TtfContext,
}

impl TextDrawer {
	pub fn new(texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Self {
		TextDrawer { texture_creator, ttf_context: sdl2::ttf::init().map_err(|e| e.to_string()).unwrap() }
	}
	
	fn get_texture(&self, text: &str, style: &TextStyle, font_size: u16) -> sdl2::render::Texture {
		let TextStyle {
			font_name: font_path,
			font_style,
			color,
		} = style;
		
		let mut font: sdl2::ttf::Font = self.ttf_context.load_font(Path::new(&font_path), font_size).unwrap();
		
		font.set_style(*font_style);
		
		// render a surface, and convert it to a texture bound to the canvas
		let surface = font.render(text).blended(*color).map_err(|e| e.to_string()).expect("text texture rendering error");
		
		self.texture_creator.create_texture_from_surface(&surface).map_err(|e| e.to_string()).ok().unwrap()
	}
	
	pub fn text_size(&self, text: &str, font_size: f64, style: &TextStyle) -> Vector2<u32> {
		if text.is_empty() { return Vector2::zeros(); }
		let TextureQuery { width, height, .. } = self.get_texture(text, style, font_size as u16).query();
		Vector2::new(width, height)
	}
	
	pub fn draw(&self, canvas: &mut Canvas<Window>, position: Point2<f64>, text: &str, font_size: f64, style: &TextStyle, align: Align) {
		let texture = self.get_texture(text, style, font_size as u16);
		let TextureQuery { width, height, .. } = texture.query();
		let size: Vector2<f64> = Vector2::new(width as f64, height as f64);
		let target_position = match align {
			Align::TopLeft => position,
			Align::Top => position - Vector2::new(size.x / 2.0, 0.),
			Align::TopRight => position - Vector2::new(size.x, 0.),
			Align::Left => position - Vector2::new(0., size.y / 2.),
			Align::Center => position - size / 2.0,
			Align::Right => position - Vector2::new(size.x, size.y / 2.),
			Align::BottomLeft => position - Vector2::new(0., size.y),
			Align::Bottom => position - Vector2::new(size.x / 2., size.y),
			Align::BottomRight => position - size,
		};
		let target = sdl2::rect::Rect::new(target_position.x as i32, target_position.y as i32, width as u32, height as u32);
		
		canvas.copy(&texture, None, Some(target)).unwrap();
	}
}
