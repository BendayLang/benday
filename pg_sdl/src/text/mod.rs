mod text_style;

use crate::style::Align;
use nalgebra::{Point2, Vector2};
use sdl2::render::TextureQuery;
use sdl2::{render::Canvas, video::Window};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub use text_style::TextStyle;
pub use text_style::{DEFAULT_FONT_NAME, FONT_PATH};

pub type FontSize = u16;
pub type FontInfos = (PathBuf, FontSize);

pub struct TextDrawer<'ttf> {
	pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
	pub font_cache: HashMap<FontInfos, sdl2::ttf::Font<'ttf, 'static>>,
}

impl<'ttf> TextDrawer<'ttf> {
	pub fn new(texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Self {
		TextDrawer { texture_creator, font_cache: HashMap::new() }
	}

	pub fn text_size(&self, text: &str, font_size: FontSize, style: &TextStyle) -> Vector2<u32> {
		if text.is_empty() {
			return Vector2::zeros();
		}
		let TextStyle { font_path, .. } = style;
		let (width, height) = self.font_cache.get(&(font_path.to_path_buf(), font_size)).unwrap().size_of(text).unwrap();
		Vector2::new(width, height)
	}

	pub fn draw(
		&mut self, canvas: &mut Canvas<Window>, position: Point2<f64>, text: &str, font_size: FontSize, style: &TextStyle,
		align: Align,
	) {
		let TextStyle { font_path, font_style, color } = style;
		let font = self.font_cache.get_mut(&(font_path.to_path_buf(), font_size)).expect("font not loaded at init");

		font.set_style(font_style.clone());
		let surface = font.render(text).blended(*color).expect("text texture rendering error");

		let texture = self.texture_creator.create_texture_from_surface(&surface).unwrap();
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
