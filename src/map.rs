use std::io::{self, Read};
use crate::wavefront::{WavefrontLoadError, load_obj};
use std::fs::File;
use crate::quadoctree::{QuadOctreeNode, BoundingBox};
use crate::draw::{Light, ObjDef};
use crate::assets::find_asset;
use std::collections::{HashMap, BTreeMap};
use glium::{Display, texture::Texture2d};
use derive_more::{From, Error};

const DEFAULT_TREE_STARTPOS: [f32; 3] = [-50.0f32; 3];
const DEFAULT_TREE_ENDPOS: [f32; 3] = [50.0f32; 3];
const DEFAULT_TREE_BUCKET_CAPACITY: usize = 300;

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

	fn parse_lights(&mut self) -> Result<(), GameMapError> {
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

	fn parse_quadoctree(meta: &toml::Value) -> QuadOctreeNode {
		let mut start_pos = DEFAULT_TREE_STARTPOS;
		let mut end_pos = DEFAULT_TREE_ENDPOS;
		let mut is_octree = false;
		let mut bucket_capacity = DEFAULT_TREE_BUCKET_CAPACITY;
		if let Some(quadoctree_params) = meta.get("quadoctree") {
			if let Some(start_pos_p) = quadoctree_params.get("start_pos") {
				if let Ok(start_pos_p) = start_pos_p.clone().try_into::<[f32; 3]>() {
					start_pos = start_pos_p;
				}
			}
			if let Some(end_pos_p) = quadoctree_params.get("end_pos") {
				if let Ok(end_pos_p) = end_pos_p.clone().try_into::<[f32; 3]>() {
					end_pos = end_pos_p;
				}
			}
			if let Some(is_octree_p) = quadoctree_params.get("is_octree") {
				if let Some(is_octree_p) = is_octree_p.as_bool() {
					is_octree = is_octree_p;
				}
			}
			if let Some(bucket_capacity_p) = quadoctree_params.get("bucket_capacity") {
				if let Some(bucket_capacity_p) = bucket_capacity_p.as_integer() {
					bucket_capacity = bucket_capacity_p as usize;
				}
			}
		}
		
		QuadOctreeNode::new_tree(BoundingBox { start_pos: start_pos, end_pos: end_pos }, is_octree, bucket_capacity)
	}

	pub fn load_map(path: &str, app_id: &str, display: Option<&Display>, textures: Option<&mut HashMap<String, Texture2d>>,
		create_quadoctree: bool) -> Result<GameMap, GameMapError> {
		let mut lights: HashMap<String, Light> = HashMap::new();
		let mut misc_objs: HashMap<String, [f32; 3]> = HashMap::new();

		let meta = Self::load_meta(path, app_id)?;

		let mut quadoctree = if create_quadoctree {
			Some(Self::parse_quadoctree(&meta))
		} else {
			None
		};

		let objects = load_obj(format!("{}{}", path, ".obj").as_str(), app_id, display, textures, &[1., 1., 1.],
			quadoctree.as_mut(), Some(&mut lights), Some(&mut misc_objs))?;

		let mut result = Self {
			lights: lights,
			quadoctree: quadoctree,
			objects: objects,
			meta: meta,
			misc_objs: misc_objs
		};

		result.parse_lights()?;

		Ok(result)
	}
}
