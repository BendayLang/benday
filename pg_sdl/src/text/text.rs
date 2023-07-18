use crate::color::Colors;
use sdl2::pixels::Color;
use std::path::Path;

// TODO
static FONT_PATH: &str = "./fonts";
static DEFAULT_FONT_NAME: &str = "Vera.ttf";

pub struct TextStyle {
	pub color: Color,
	pub font_name: String,
	pub font_style: sdl2::ttf::FontStyle,
}

impl TextStyle {
	pub fn new(font_name: Option<&str>, color: Color, font_style: sdl2::ttf::FontStyle) -> Self {
		let font_name = if let Some(font_name) = font_name {
			let font_name = format!("{}/Vera.ttf", FONT_PATH);
			if !Path::new(&font_name).exists() {
				format!("{}/DejaVuSans.ttf", FONT_PATH);
			}
			font_name
		} else {
			format!("{}/{}", FONT_PATH, DEFAULT_FONT_NAME)
		};

		Self { font_name, color, font_style }
	}
}

impl Default for TextStyle {
	fn default() -> Self {
		Self {
			font_name: format!("{}/{}", FONT_PATH, DEFAULT_FONT_NAME),
			color: Colors::BLACK,
			font_style: sdl2::ttf::FontStyle::NORMAL,
		}
	}
}
