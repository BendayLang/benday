mod text_style;

use crate::style::Align;
use itertools::Itertools;
use nalgebra::{Point2, Scalar, Vector2};
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::surface::SurfaceContext;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{render::Canvas, video::Window};
use std::collections::HashMap;
use std::path::PathBuf;
pub use text_style::TextStyle;
pub use text_style::{DEFAULT_FONT_NAME, FONT_PATH};

type Key = (String, TextStyle, u16);
pub type FontSize = u16;
pub type FontInfos = (PathBuf, FontSize);
pub type TextureCache<'surface> = HashMap<Key, Surface<'surface>>;

lazy_static! {
	pub static ref TTF_CONTEXT: Sdl2TtfContext = sdl2::ttf::init().expect("SDL2 ttf could not be initialized");
}

#[derive(Default)]
pub struct TextDrawer<'ttf, 'surface> {
	// texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
	fonts: HashMap<FontInfos, sdl2::ttf::Font<'ttf, 'static>>,
	texture_cache: TextureCache<'surface>,
}

macro_rules! get_font_or_add {
	($self:ident, $path:ident, $size:ident) => {{
		if !$self.fonts.contains_key(&($path.to_path_buf(), $size)) {
			let font: sdl2::ttf::Font = TTF_CONTEXT.load_font($path, $size).unwrap();
			$self.fonts.insert(($path.to_path_buf(), $size), font);
		}
		$self.fonts.get_mut(&($path.to_path_buf(), $size)).unwrap()
	}};
}

impl<'ttf, 'texture> TextDrawer<'ttf, 'texture> {
	pub fn size_of<T: Scalar + num_traits::NumCast + num_traits::Zero>(
		&mut self, text: &str, font_size: FontSize, style: &TextStyle,
	) -> Vector2<T> {
		if text.is_empty() {
			return Vector2::zeros();
		}
		let TextStyle { font_path, .. } = style;
		let font = get_font_or_add!(self, font_path, font_size);
		let (width, height) = font.size_of(text).unwrap();
		Vector2::new(num_traits::cast(width).unwrap(), num_traits::cast(height).unwrap())
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
		&mut self, canvas: &mut Canvas<Surface>, position: Point2<f64>, text: &str, font_size: FontSize, style: &TextStyle,
		align: Align,
	) {
		let TextStyle { font_path, color, font_style, .. } = style;
		let target = {
			let size = self.size_of::<f64>(text, font_size, style);
			let target_position = position - Self::shift_from_align(align, size);
			sdl2::rect::Rect::new(target_position.x as i32, target_position.y as i32, size.x as u32, size.y as u32)
		};
		let texture_creator = canvas.texture_creator();
		let texture = {
			let font = get_font_or_add!(self, font_path, font_size);
			font.set_style(*font_style);
			let key: Key = (text.to_string(), style.clone(), font_size);

			if let Some(surface) = self.texture_cache.get(&key) {
				texture_creator.create_texture_from_surface(surface).unwrap()
			} else {
				println!("new len of texture cache: {}. Created for '{}'", self.texture_cache.len(), text);
				let surface = font.render(text).blended(*color).expect("text texture rendering error");
				let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
				self.texture_cache.insert(key, surface);
				texture
			}
		};

		canvas.copy(&texture, None, Some(target)).unwrap();
	}
}
