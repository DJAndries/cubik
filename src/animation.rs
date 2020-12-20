use std::io;
use std::fs;
use std::path::Path;
use glium::{Display, texture::Texture2d};
use std::collections::{HashMap, BTreeMap};
use crate::draw::ObjDef;
use crate::wavefront::{WavefrontLoadError, load_obj};
use derive_more::{Error, From};

#[derive(Debug, derive_more::Display, Error, From)]
pub enum ObjAnimationError {
	IOError(io::Error),
	WavefrontLoadError(WavefrontLoadError)
}

pub struct ObjAnimation {
	pub keyframes: Vec<BTreeMap<String, ObjDef>>,
	pub keyframe_time: f32,
}

impl ObjAnimation {
	pub fn load_wavefront(name: &str, display: &Display, textures: &mut HashMap<String, Texture2d>, keyframe_time: f32) -> Result<ObjAnimation, ObjAnimationError> {
		let animation_path = Path::new(name);
		let read_dir = fs::read_dir(&animation_path)?;

		let mut keyframe_files: Vec<String> = Vec::new();

		for entry in fs::read_dir(&animation_path)? {
			let path = entry?.path();
			if path.is_dir() { continue; }

			if let Some(ext) = path.extension() {
				if ext != "obj" { continue; }

				keyframe_files.push(path.to_str().unwrap().to_string());
			}
		}

		keyframe_files.sort();
		let mut result = ObjAnimation {
			keyframes: Vec::with_capacity(keyframe_files.len()),
			keyframe_time: keyframe_time
		};

		for keyframe_file in keyframe_files {
			let obj = load_obj(keyframe_file.as_str(), display, textures, &[1., 1., 1.], None, None)?;
			result.keyframes.push(obj);
		}

		Ok(result)
	}

	pub fn get_keyframe(&self, current_time: f32) -> &BTreeMap<String, ObjDef> {
		&self.keyframes[((current_time / self.keyframe_time) as usize) % self.keyframes.len()]
	}
}
