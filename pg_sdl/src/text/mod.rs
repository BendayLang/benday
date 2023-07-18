mod text;

use crate::style::Align;
use nalgebra::{Point2, Vector2};
use sdl2::render::TextureQuery;
use sdl2::{render::Canvas, video::Window};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub use text::TextStyle;
pub use text::{DEFAULT_FONT_NAME, FONT_PATH};

pub type FontInfos = (PathBuf, u16);

pub struct TextDrawer<'ttf, 'rwops> {
	pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
	// ttf_context: sdl2::ttf::Sdl2TtfContext,
	pub font_cache: HashMap<FontInfos, sdl2::ttf::Font<'ttf, 'rwops>>,
	// font_cache: HashMap<FontInfos, sdl2::ttf::Font<'static, 'static>>,
}

impl<'ttf> TextDrawer<'ttf, '_> {
	pub fn new(texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Self {
		TextDrawer { texture_creator, font_cache: HashMap::new() }
	}

	fn get_texture(&self, text: &str, style: &TextStyle, font_size: u16) -> sdl2::render::Texture {
		let TextStyle { font_path, font_style, color } = style;
		let font = self.font_cache.get(&(font_path.to_path_buf(), font_size)).expect("font not loaded at init");
		let surface = font.render(text).blended(*color).map_err(|e| e.to_string()).expect("text texture rendering error");
		self.texture_creator.create_texture_from_surface(&surface).map_err(|e| e.to_string()).ok().unwrap()
	}

	pub fn text_size(&self, text: &str, font_size: f64, style: &TextStyle) -> Vector2<u32> {
		if text.is_empty() {
			return Vector2::zeros();
		}
		let TextureQuery { width, height, .. } = self.get_texture(text, style, font_size as u16).query();
		Vector2::new(width, height)
	}

	pub fn draw(
		&self, canvas: &mut Canvas<Window>, position: Point2<f64>, text: &str, font_size: f64, style: &TextStyle, align: Align,
	) {
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
