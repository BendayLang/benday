use crate::color::Colors;
use sdl2::pixels::Color;
use std::path::{Path, PathBuf};

// TODO
pub static FONT_PATH: &str = "./fonts";
pub static DEFAULT_FONT_NAME: &str = "Vera.ttf";

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TextStyle {
	pub color: Color,
	pub font_path: PathBuf,
	pub font_style: sdl2::ttf::FontStyle,
}

impl TextStyle {
	pub fn new(font_path: Option<PathBuf>, color: Color, font_style: sdl2::ttf::FontStyle) -> Self {
		let font_path: PathBuf = if let Some(font_path) = font_path {
			let font_path = format!("{}/Vera.ttf", FONT_PATH);
			if !Path::new(&font_path).exists() {
				format!("{}/DejaVuSans.ttf", FONT_PATH);
			}
			font_path
		} else {
			format!("{}/{}", FONT_PATH, DEFAULT_FONT_NAME)
		}
		.into();

		Self { font_path, color, font_style }
	}
}

impl Default for TextStyle {
	fn default() -> Self {
		Self {
			font_path: format!("{}/{}", FONT_PATH, DEFAULT_FONT_NAME).into(),
			color: Colors::BLACK,
			font_style: sdl2::ttf::FontStyle::NORMAL,
		}
	}
}
