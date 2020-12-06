use image::{DynamicImage, Rgba};
use std::fs::File;
use std::io;
use std::io::Read;
use std::collections::HashMap;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use rusttype::{Font, Scale, Point, Rect, HMetrics};
use derive_more::{Error, From};
use crate::draw::{Vertex, load_data_to_gpu, ObjDef};
use crate::draw::{UIDrawInfo, ui_draw};
use glium::{DrawParameters, Display, Frame, Surface, texture::{SrgbTexture2d, RawImage2d, TextureCreationError}};

const WHITE: [f32; 4] = [1., 1., 1., 1.];

pub enum TextAlign {
	Left,
	Center,
	Right
}

struct FontChar {
	texture: Option<SrgbTexture2d>,
	bbox: Option<Rect<i32>>,
	h_metrics: HMetrics
}

pub struct FontText {
	chars: Vec<(char, Option<ObjDef>)>,
	last_font_hash: u64,
	align: TextAlign,
	text: String,
	pub ui_draw_info: UIDrawInfo
}

#[derive(Default)]
pub struct LoadedFont {
	hash: u64, 
	chars: HashMap<char, FontChar>,
	pub font_size: f32
}

#[derive(Debug, derive_more::Display, Error, From)]
pub enum FontError {
	IOError(io::Error),
	LoadError,
	TextureUploadError(TextureCreationError),
	CharNotAvailable
}

impl LoadedFont {
	pub fn load(display: &Display, filename: &str, font_size: f32) -> Result<Self, FontError> {
		let mut f = File::open(filename)?;
		let mut f_contents = Vec::new();
		f.read_to_end(&mut f_contents)?;

		let font = Font::try_from_vec(f_contents).ok_or(FontError::LoadError)?;
		let scale = Scale::uniform(font_size);

		let mut hasher = DefaultHasher::new();
		hasher.write(filename.as_bytes());
		hasher.write_u32(font_size.to_bits());
		let mut result = Self { hash: hasher.finish(), font_size: font_size, ..Default::default() };

		for ch in b' '..=b'~' {
			let glyph = font.glyph(ch as char).scaled(scale);
			let h_metrics = glyph.h_metrics();
			let positioned = glyph.positioned(Point { x: 0., y: 0. });
			let mut char_result = FontChar { texture: None, bbox: None, h_metrics: h_metrics };
			if let Some(positioned_box) = positioned.pixel_bounding_box() {
				let glyph_size = (positioned_box.width() as usize, positioned_box.height() as usize);
				let mut glyph_data = vec![0u8; glyph_size.1 * glyph_size.0 * 4];
				positioned.draw(|x, y, v| {
					let val = (v * 255.) as u8;
					let index = ((y * (glyph_size.0 as u32) + x) * 4) as usize;
					glyph_data[index] = 255;
					glyph_data[index + 1] = 255;
					glyph_data[index + 2] = 255;
					glyph_data[index + 3] = val;
				});

				let txt = SrgbTexture2d::new(display,
					RawImage2d::from_raw_rgba_reversed(&glyph_data, (glyph_size.0 as u32, glyph_size.1 as u32)))?;

				char_result.texture = Some(txt);
				char_result.bbox = Some(positioned_box);
			}

			result.chars.insert(ch as char, char_result);
		}

		Ok(result)
	}
}

impl FontText {
	pub fn new(text: String, size: f32, pos: (f32, f32), align: TextAlign) -> Self {
		let text_len = text.len();
		let mut ui_draw_info = UIDrawInfo::new(pos, (size, size));
		ui_draw_info.translate_after_scale = true;
		Self {
			text: text,
			align: align,
			last_font_hash: 0,
			chars: Vec::with_capacity(text_len),
			ui_draw_info: ui_draw_info
		}
	}
	
	pub fn measure_width(&mut self, font: &LoadedFont) -> Result<f32, FontError> {
		let mut width = 0.0f32;
		for c in self.text.chars() {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;
			
			width += ch.h_metrics.advance_width / font.font_size;
		}

		Ok(width)
	}

	fn starting_position(&mut self, font: &LoadedFont) -> Result<f32, FontError> {
		let width = self.measure_width(font)?;
		let x_offset: f32 = match self.align {
			TextAlign::Left => 0.,
			TextAlign::Right => width,
			TextAlign::Center => width / 2.
		};
		Ok(-x_offset)
	}

	fn prepare_chars(&mut self, target: &Frame, display: &Display, font: &LoadedFont) -> Result<(), FontError> {
		let mut pos = (self.starting_position(font)?, 0.0);
		for c in self.text.chars() {
			let mut char_result: Option<ObjDef> = None;

			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;

			pos.0 += ch.h_metrics.left_side_bearing / font.font_size;

			if let Some(bbox) = ch.bbox {
				let ch_size = (bbox.width() as f32, bbox.height() as f32);
				let real_size = (ch_size.0 / font.font_size, ch_size.1 / font.font_size);

				pos.1 -= (bbox.max.y as f32) / font.font_size;

				let vertices = [
					Vertex { position: [pos.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [0., 0.] },
					Vertex { position: [pos.0 + real_size.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [1., 0.] },
					Vertex { position: [pos.0 + real_size.0, pos.1 + real_size.1, 0.], normal: [0., 0., -1.], texcoords: [1., 1.] },
					Vertex { position: [pos.0, pos.1 + real_size.1, 0.], normal: [0., 0., -1.], texcoords: [0., 1.] }
				];
				
				let indices = [0, 1, 2, 0, 2, 3];

				char_result = Some(load_data_to_gpu(display, &vertices, &indices));

				pos.1 += (bbox.max.y as f32) / font.font_size;
			}
			self.chars.push((c, char_result));

			pos.0 += (ch.h_metrics.advance_width - ch.h_metrics.left_side_bearing) / font.font_size;
		}
		Ok(())
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, program: &glium::Program, font: &LoadedFont) -> Result<(), FontError> {
		if self.last_font_hash != font.hash {
			self.chars.clear();
			self.prepare_chars(target, display, font)?;
			self.last_font_hash = font.hash;
		}
		self.ui_draw_info.generate_matrix(target);

		let params = DrawParameters {
			blend: glium::draw_parameters::Blend::alpha_blending(),
			clip_planes_bitmask: 1,
			..Default::default()
		};

		let dim = target.get_dimensions();

		for (c, obj_def) in &self.chars {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;

			if let Some(obj_def) = obj_def {
				ui_draw(target, &obj_def, &self.ui_draw_info, program, ch.texture.as_ref().unwrap());
			}
		}
		Ok(())
	}
}
