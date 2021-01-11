use std::io::{self, Read};
use crate::wavefront::{WavefrontLoadError, load_obj};
use std::fs::File;
use crate::quadoctree::QuadOctreeNode;
use crate::draw::{Light, ObjDef};
use crate::assets::find_asset;
use std::collections::{HashMap, BTreeMap};
use glium::{Display, texture::Texture2d};
use derive_more::{From, Error};

#[derive(From, Error, derive_more::Display, Debug)]
pub enum GameMapError {
	WavefrontLoadError(WavefrontLoadError),
	IOError(io::Error),
	BadTomlFile,
	BadLightDesc
}

pub struct GameMap {
	pub quadoctree: Option<QuadOctreeNode>,
	pub lights: HashMap<String, Light>,
	pub misc_objs: HashMap<String, [f32; 3]>,
	pub objects: BTreeMap<String, ObjDef>,
	pub meta: toml::Value
}

impl GameMap {
	fn load_meta(path: &str, app_id: &str) -> Result<toml::Value, GameMapError> {
		let mut file = File::open(find_asset(format!("{}{}", path, ".toml").as_str(), app_id))?;
		let mut contents = String::new();
		file.read_to_string(&mut contents)?;
		Ok(contents.parse::<toml::Value>().map_err(|_| GameMapError::BadTomlFile)?)
	}

	fn parse_meta(&mut self) -> Result<(), GameMapError> {
		if let Some(lights) = self.meta.get("lights") {
			if let Some(lights_table) = lights.as_table() {
				for (name, light_info) in lights_table.iter() {
					let mut light_meta: Light = light_info.clone().try_into().map_err(|_| GameMapError::BadLightDesc)?;
					if let Some(map_light) = self.lights.get(name) {
						light_meta.position = map_light.position;
						self.lights.insert(name.clone(), light_meta);
					}
				}
			}
		}
		
		Ok(())
	}

	pub fn load_map(path: &str, app_id: &str, display: Option<&Display>, textures: Option<&mut HashMap<String, Texture2d>>,
		mut quadoctree: Option<QuadOctreeNode>) -> Result<GameMap, GameMapError> {
		let mut lights: HashMap<String, Light> = HashMap::new();
		let mut misc_objs: HashMap<String, [f32; 3]> = HashMap::new();

		let objects = load_obj(format!("{}{}", path, ".obj").as_str(), app_id, display, textures, &[1., 1., 1.],
			quadoctree.as_mut(), Some(&mut lights), Some(&mut misc_objs))?;

		let meta = Self::load_meta(path, app_id)?;

		let mut result = Self {
			lights: lights,
			quadoctree: quadoctree,
			objects: objects,
			meta: meta,
			misc_objs: misc_objs
		};

		result.parse_meta()?;

		Ok(result)
	}
}
