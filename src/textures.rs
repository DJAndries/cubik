use std::io;
use std::fs::File;
use std::path::Path;
use derive_more::{Error, From};
use std::io::BufReader;
use std::collections::HashMap;
use glium::{Display, texture::{Texture2d, SrgbTexture2d, RawImage2d, TextureCreationError}};

pub const WHITE: &str = "cubik_white";

#[derive(Debug, derive_more::Display, Error, From)]
pub enum TextureLoadError {
	IOError(io::Error),
	TextureImageLoadError(image::error::ImageError),
	TextureUploadError(TextureCreationError)
}

fn load_raw_image(path: &Path, reversed: bool) -> Result<RawImage2d<u8>, TextureLoadError> {
	let f = File::open(path.clone())?;
	let f = BufReader::new(f);

	let image = image::load(f, image::ImageFormat::from_path(path.clone())?)?.to_rgba8();
	let image_dim = image.dimensions();
	Ok(if reversed {
		RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim)
	} else {
		RawImage2d::from_raw_rgba(image.into_vec(), image_dim)
	})
}

pub fn load_texture(display: &Display, path: &Path, reversed: bool) -> Result<Texture2d, TextureLoadError> {
	let raw_image = load_raw_image(path, reversed)?;
	Ok(Texture2d::new(display, raw_image)?)
}

pub fn load_srgb_texture(display: &Display, path: &Path, reversed: bool) -> Result<SrgbTexture2d, TextureLoadError> {
	let raw_image = load_raw_image(path, reversed)?;
	Ok(SrgbTexture2d::new(display, raw_image)?)
}

pub fn create_texture_map(display: &Display) -> Result<HashMap<String, Texture2d>, TextureLoadError> {
	let mut result = HashMap::new();
	result.insert(WHITE.to_string(), Texture2d::new(display, RawImage2d::from_raw_rgba(vec![0, 0, 0, 0], (1, 1)))?);
	Ok(result)
}
