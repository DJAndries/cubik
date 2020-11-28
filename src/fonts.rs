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
use glium::{DrawParameters, Display, Frame, Surface, texture::{Texture2d, RawImage2d, TextureCreationError}};

pub enum TextAlign {
	Left,
	Center,
	Right
}

struct FontChar {
	texture: Option<Texture2d>,
	bbox: Option<Rect<i32>>,
	h_metrics: HMetrics
}

pub struct FontText {
	chars: Vec<(char, Option<ObjDef>)>,
	screen_dim: (u32, u32),
	last_font_hash: u64,
	align: TextAlign,
	text: String,
	pos: (f32, f32),
	units_per_pixel: (f32, f32),
	color: [f32; 4]
}

#[derive(Default)]
pub struct LoadedFont {
	hash: u64, 
	chars: HashMap<char, FontChar>
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
		let mut result = Self { hash: hasher.finish(), ..Default::default() };

		for ch in b' '..=b'~' {
			let glyph = font.glyph(ch as char).scaled(scale);
			let h_metrics = glyph.h_metrics();
			let positioned = glyph.positioned(Point { x: 0., y: 0. });
			let mut char_result = FontChar { texture: None, bbox: None, h_metrics: h_metrics };
			if let Some(positioned_box) = positioned.pixel_bounding_box() {
				let glyph_size = (positioned_box.width() as usize, positioned_box.height() as usize);
				let mut glyph_data = vec![0u8; glyph_size.1 * glyph_size.0 * 3];
				positioned.draw(|x, y, v| {
					let val = (v * 255.) as u8;
					let index = ((y * (glyph_size.0 as u32) + x) * 3) as usize;
					glyph_data[index] = val;
					glyph_data[index + 1] = val;
					glyph_data[index + 2] = val;
				});

				let txt = Texture2d::new(display,
					RawImage2d::from_raw_rgb_reversed(&glyph_data, (glyph_size.0 as u32, glyph_size.1 as u32)))?;

				char_result.texture = Some(txt);
				char_result.bbox = Some(positioned_box);
			}

			result.chars.insert(ch as char, char_result);
		}

		Ok(result)
	}
}

impl FontText {
	pub fn new(text: String, pos: (f32, f32), color: [f32; 4], align: TextAlign) -> Self {
		let text_len = text.len();
		Self {
			pos: pos,
			text: text,
			color: color,
			align: align,
			last_font_hash: 0,
			units_per_pixel: (0., 0.),
			screen_dim: (0, 0),
			chars: Vec::with_capacity(text_len)
		}
	}

	fn starting_position(&mut self, font: &LoadedFont) -> Result<(f32, f32), FontError> {
		let mut width = 0.0f32;
		for c in self.text.chars() {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;
			
			width += ch.h_metrics.advance_width * self.units_per_pixel.0;
		}

		let x_offset: f32 = match self.align {
			TextAlign::Left => 0.,
			TextAlign::Right => width,
			TextAlign::Center => width / 2.
		};
		Ok((self.pos.0 - x_offset, self.pos.1))
	}

	fn prepare_chars(&mut self, target: &Frame, display: &Display, font: &LoadedFont) -> Result<(), FontError> {
		self.screen_dim = target.get_dimensions();
		self.units_per_pixel = (2.0f32 / (self.screen_dim.0 as f32), 2.0f32 / (self.screen_dim.1 as f32));

		let mut pos = self.starting_position(font)?;
		for c in self.text.chars() {
			let mut char_result: Option<ObjDef> = None;

			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;

			pos.0 += ch.h_metrics.left_side_bearing * self.units_per_pixel.0;

			if let Some(bbox) = ch.bbox {
				let ch_size = ((bbox.width() as f32) * self.units_per_pixel.0,
					(bbox.height() as f32) * self.units_per_pixel.1);

				pos.1 -= (bbox.max.y as f32) * self.units_per_pixel.1;

				let vertices = [
					Vertex { position: [pos.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [0., 0.] },
					Vertex { position: [pos.0 + ch_size.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [1., 0.] },
					Vertex { position: [pos.0 + ch_size.0, pos.1 + ch_size.1, 0.], normal: [0., 0., -1.], texcoords: [1., 1.] },
					Vertex { position: [pos.0, pos.1 + ch_size.1, 0.], normal: [0., 0., -1.], texcoords: [0., 1.] }
				];
				
				let indices = [0, 1, 2, 0, 2, 3];

				char_result = Some(load_data_to_gpu(display, &vertices, &indices));

				pos.1 += (bbox.max.y as f32) * self.units_per_pixel.1;
			}
			self.chars.push((c, char_result));

			pos.0 += (ch.h_metrics.advance_width - ch.h_metrics.left_side_bearing) * self.units_per_pixel.0;
		}
		Ok(())
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, program: &glium::Program, font: &LoadedFont) -> Result<(), FontError> {
		if self.screen_dim != target.get_dimensions() || self.last_font_hash != font.hash {
			self.chars.clear();
			self.prepare_chars(target, display, font)?;
			self.last_font_hash = font.hash;
		}

		let params = DrawParameters {
			blend: glium::draw_parameters::Blend::alpha_blending(),
			..Default::default()
		};

		for (c, obj_def) in &self.chars {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;

			if let Some(obj_def) = obj_def {
				let uniforms = uniform! {
					tex: ch.texture.as_ref().unwrap().sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
					text_color: self.color
				};

				target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, &params);
			}
		}
		Ok(())
	}
}
