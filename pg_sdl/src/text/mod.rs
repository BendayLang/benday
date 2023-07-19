mod text_style;

use crate::style::Align;
use nalgebra::{Point2, Vector2};
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;
use sdl2::ttf::FontStyle;
use sdl2::{render::Canvas, video::Window};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub use text_style::TextStyle;
pub use text_style::{DEFAULT_FONT_NAME, FONT_PATH};

pub type FontSize = u16;
pub type FontInfos = (PathBuf, FontSize);
pub type TextureCache<'texture> = HashMap<(String, TextStyle, FontSize), sdl2::render::Texture<'texture>>;

pub struct TextDrawer<'ttf, 'texture> {
	texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
	pub fonts: HashMap<FontInfos, sdl2::ttf::Font<'ttf, 'static>>,
	texture_cache: Option<TextureCache<'texture>>,
}

impl<'ttf, 'texture> TextDrawer<'ttf, 'texture> {
	pub fn new(texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Self {
		TextDrawer { texture_creator, fonts: HashMap::new(), texture_cache: Some(HashMap::new()) }
	}

	pub fn size_of_u32(&self, text: &str, font_size: FontSize, style: &TextStyle) -> Vector2<u32> {
		if text.is_empty() {
			return Vector2::zeros();
		}
		let TextStyle { font_path, .. } = style;
		let font = self.fonts.get(&(font_path.to_path_buf(), font_size)).unwrap();
		let (width, height) = font.size_of(text).unwrap();
		Vector2::new(width, height)
	}

	pub fn size_of_f64(&self, text: &str, font_size: FontSize, style: &TextStyle) -> Vector2<f64> {
		if text.is_empty() {
			return Vector2::zeros();
		}
		let TextStyle { font_path, .. } = style;
		let font = self.fonts.get(&(font_path.to_path_buf(), font_size)).unwrap();
		let (width, height) = font.size_of(text).unwrap();
		Vector2::new(width as f64, height as f64)
	}

	fn shift_from_align(align: Align, size: Vector2<f64>) -> Vector2<f64> {
		match align {
			Align::TopLeft => Vector2::zeros(),
			Align::Top => Vector2::new(size.x / 2.0, 0.),
			Align::TopRight => Vector2::new(size.x, 0.),
			Align::Left => Vector2::new(0., size.y / 2.),
			Align::Center => size / 2.0,
			Align::Right => Vector2::new(size.x, size.y / 2.),
			Align::BottomLeft => Vector2::new(0., size.y),
			Align::Bottom => Vector2::new(size.x / 2., size.y),
			Align::BottomRight => size,
		}
	}

	pub fn draw(
		&mut self, canvas: &mut Canvas<Window>, position: Point2<f64>, text: &str, font_size: FontSize, style: &TextStyle,
		align: Align,
	) {
		let TextStyle { font_path, font_style, color } = style;
		let font = self.fonts.get_mut(&(font_path.to_path_buf(), font_size)).expect("font not loaded at init");
		font.set_style(font_style.clone());
		let surface: sdl2::surface::Surface<'_> = font.render(text).blended(*color).expect("text texture rendering error");
		let texture = self.texture_creator.create_texture_from_surface(&surface).unwrap();

		// let mut texture_cache: Option<TextureCache> = None;
		// std::mem::swap(&mut self.texture_cache, &mut texture_cache);

		// let key = (text.to_string(), style.clone(), font_size);
		// if texture_cache.as_ref().unwrap().get(&key).is_none() {
		// 	let surface = font.render(text).blended(*color).expect("text texture rendering error");
		// 	// let texture: sdl2::render::Texture<'_> = self.texture_creator.create_texture_from_surface(&surface).unwrap();
		// 	texture_cache
		// 		.as_mut()
		// 		.unwrap()
		// 		.insert(key.clone(), self.texture_creator.create_texture_from_surface(&surface).unwrap());
		// 	println!("new len of texture cache: {}", texture_cache.as_ref().unwrap().len());
		// }
		// let texture = texture_cache.as_ref().unwrap().get(&key).unwrap();

		let size: Vector2<f64> = self.size_of_f64(text, font_size, &style);
		let target_position: nalgebra::Point2<f64> = position - Self::shift_from_align(align, size);
		let target = sdl2::rect::Rect::new(target_position.x as i32, target_position.y as i32, size.x as u32, size.y as u32);

		canvas.copy(&texture, None, Some(target)).unwrap();
	}
}
