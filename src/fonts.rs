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
use crate::math::mult_matrix3;
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
	pub size: f32,
	pos: (f32, f32),
	model_matrix: Option<[[f32; 3]; 3]>,
	pub left_clip: f32
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

				let txt = Texture2d::new(display,
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
		Self {
			pos: pos,
			text: text,
			align: align,
			size: size,
			last_font_hash: 0,
			screen_dim: (0, 0),
			chars: Vec::with_capacity(text_len),
			left_clip: -1.,
			model_matrix: None
		}
	}
	
	pub fn measure_width(&mut self, font: &LoadedFont) -> Result<f32, FontError> {
		let mut width = 0.0f32;
		for c in self.text.chars() {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;
			
			width += ch.h_metrics.advance_width / font.font_size * self.size;
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

			pos.0 += ch.h_metrics.left_side_bearing / font.font_size * self.size;

			if let Some(bbox) = ch.bbox {
				let ch_size = (bbox.width() as f32, bbox.height() as f32);
				let real_size = (ch_size.0 / font.font_size * self.size, ch_size.1 / font.font_size * self.size);

				pos.1 -= (bbox.max.y as f32) / font.font_size * self.size;

				let vertices = [
					Vertex { position: [pos.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [0., 0.] },
					Vertex { position: [pos.0 + real_size.0, pos.1, 0.], normal: [0., 0., -1.], texcoords: [1., 0.] },
					Vertex { position: [pos.0 + real_size.0, pos.1 + real_size.1, 0.], normal: [0., 0., -1.], texcoords: [1., 1.] },
					Vertex { position: [pos.0, pos.1 + real_size.1, 0.], normal: [0., 0., -1.], texcoords: [0., 1.] }
				];
				
				let indices = [0, 1, 2, 0, 2, 3];

				char_result = Some(load_data_to_gpu(display, &vertices, &indices));

				pos.1 += (bbox.max.y as f32) / font.font_size * self.size;
			}
			self.chars.push((c, char_result));

			pos.0 += (ch.h_metrics.advance_width - ch.h_metrics.left_side_bearing) / font.font_size * self.size;
		}
		Ok(())
	}

	fn gen_model_matrix(&mut self, target: &mut Frame) {
		self.screen_dim = target.get_dimensions();
		let x_scale = self.screen_dim.1 as f32 / self.screen_dim.0 as f32;
		self.model_matrix = Some(mult_matrix3(&[
			[1., 0., 0.],
			[0., 1., 0.],
			[self.pos.0, self.pos.1, 1.0f32]
		], &[
			[x_scale, 0., 0.],
			[0., 1., 0.],
			[0., 0., 1.]
		]));
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, program: &glium::Program, font: &LoadedFont, color: [f32; 4]) -> Result<(), FontError> {
		if self.screen_dim != target.get_dimensions() {
			self.gen_model_matrix(target);
		}
		if self.last_font_hash != font.hash {
			self.chars.clear();
			self.prepare_chars(target, display, font)?;
			self.last_font_hash = font.hash;
		}

		let params = DrawParameters {
			blend: glium::draw_parameters::Blend::alpha_blending(),
			clip_planes_bitmask: 1,
			..Default::default()
		};

		let dim = target.get_dimensions();

		for (c, obj_def) in &self.chars {
			let ch = font.chars.get(&c).ok_or(FontError::CharNotAvailable)?;

			if let Some(obj_def) = obj_def {
				let uniforms = uniform! {
					left_clip: self.left_clip,
					tex: ch.texture.as_ref().unwrap().sampled()
						.magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
						.minify_filter(glium::uniforms::MinifySamplerFilter::Linear),
					text_color: color,
					model: self.model_matrix.unwrap()
				};

				target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, &params);
			}
		}
		Ok(())
	}
}
