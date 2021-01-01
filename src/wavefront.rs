use std::io;
use std::str::Split;
use std::collections::{HashMap, BTreeMap};
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use crate::draw::{ObjDef, Vertex, load_data_to_gpu, MtlInfo, Light};
use glium::{Display, texture::Texture2d};
use derive_more::{Error, From};
use crate::quadoctree::{QuadOctreeNode, QuadOctreeError, add_obj_to_quadoctree};
use crate::textures::{load_texture, TextureLoadError};
use crate::assets::find_asset;

const COLLISION_PREFIX: &str = "collision_";
const LIGHT_PREFIX: &str = "light_";
const TERRAIN_PREFIX: &str = "terrain_";

#[derive(Debug, derive_more::Display, Error, From)]
pub enum WavefrontLoadError {
	#[from(ignore)]
	FormatError { msg: &'static str },
	#[from(ignore)]
	BadIndexError { msg: &'static str },
	IOError(io::Error),
	FloatParseError(std::num::ParseFloatError),
	IntParseError(std::num::ParseIntError),
	TextureLoadError(TextureLoadError),
	QuadOctreeCreateError(QuadOctreeError)
}

#[derive(PartialEq)]
enum MeshType {
	Normal,
	Terrain,
	Collision,
	Light
}

fn parse_vertex_or_normal(split: &mut Split<char>, scale: &[f32; 3]) -> Result<[f32; 3], WavefrontLoadError> {
	let mut components = [0.0, 0.0, 0.0f32];

	for i in 0..3 {
		let str_val = split.next().ok_or(WavefrontLoadError::FormatError { msg: "Vertex or normal has too few components" })?;
		components[i] = str_val.trim().parse()?;
		components[i] *= scale[i];
	}

	Ok(components)
}

fn parse_texcoords(split: &mut Split<char>) -> Result<[f32; 2], WavefrontLoadError> {
	let mut components = [0.0, 0.0f32];
	for i in 0..2 {
		let str_val = split.next().ok_or(WavefrontLoadError::FormatError { msg: "Tex coords have too few components" })?;
		components[i] = str_val.trim().parse()?;
	}

	Ok(components)
}

fn parse_face(split: &mut Split<char>, vertex_info: &Vec<[f32; 3]>, normal_info: &Vec<[f32; 3]>,
	texcoord_info: &Vec<[f32; 2]>, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) -> Result<(), WavefrontLoadError> {

	let mut face_indices: Vec<u32> = Vec::new();

	for i in 0..4 {
		let face_index_txt = split.next();
		let face_index_txt = match face_index_txt {
			None => {
				if i < 3 {
					return Err(WavefrontLoadError::FormatError { msg: "Face has less than 3 vertices" });
				}
				break;
			},
			Some(val) => val
		};
		let mut face_index_split: Split<&str> = face_index_txt.split("/");
		let mut face_index_ref = [0, 0, 0u32];
		for j in 0..3 {
			let face_index_comp = face_index_split.next()
				.ok_or(WavefrontLoadError::FormatError { msg: "Face index ref must contain 3 indices" })?;
			if j == 1 && face_index_comp.len() == 0 {
				continue;
			}
			face_index_ref[j] = face_index_comp.trim().parse()?;
		}

		let mut new_vert = Vertex {
			position: vertex_info.get((face_index_ref[0] - 1) as usize)
				.ok_or(WavefrontLoadError::BadIndexError { msg: "Vertex index does not exist" })?.clone(),
			normal: normal_info.get((face_index_ref[2] - 1) as usize)
				.ok_or(WavefrontLoadError::BadIndexError { msg: "Normal index does not exist" })?.clone(),
			texcoords: [0., 0.]
		};
		if face_index_ref[1] > 0 {
			new_vert.texcoords = texcoord_info.get((face_index_ref[1] - 1) as usize)
				.ok_or(WavefrontLoadError::BadIndexError { msg: "Texcoord index does not exist" })?.clone();
		}
		vertices.push(new_vert);
		
		face_indices.push((vertices.len() - 1) as u32);
	}

	if split.next().is_some() {
		return Err(WavefrontLoadError::FormatError { msg: "Face has more than 4 vertices" });
	}

	indices.extend_from_slice(&[face_indices[2], face_indices[1], face_indices[0]]);
	if face_indices.len() == 4 {
		indices.extend_from_slice(&[face_indices[3], face_indices[2], face_indices[0]]);
	}
	Ok(())
}



fn load_mtl(display: &Display, obj_split: &mut Split<char>, obj_parent_dir: &Path,
	textures: &mut HashMap<String, Texture2d>, mtl_map: &mut HashMap<String, MtlInfo>) -> Result<(), WavefrontLoadError> {
	let filename = obj_split.next()
		.ok_or(WavefrontLoadError::FormatError { msg: "mtllib does not have filename" })?;
	
	let f = File::open(obj_parent_dir.join(filename.trim()).as_path())?;
	let mut f = BufReader::new(f);

	let mut line = String::new();

	let mut current_name: Option<String> = None;
	
	while f.read_line(&mut line)? != 0 {
		let mut split = line.split(' ');

		let key = split.next().unwrap().trim();

		if key == "newmtl" {
			let name = split.next()
					.ok_or(WavefrontLoadError::FormatError { msg: "newmtl does not have a name" })?.to_string();
			mtl_map.insert(name.clone(), Default::default());
			current_name = Some(name);
		}

		if let Some(name) = current_name.as_ref() {
			let mtl = mtl_map.get_mut(name).unwrap();
			match key {
				"map_Kd" => {
					let img_filename = split.next()
						.ok_or(WavefrontLoadError::FormatError { msg: "map_Kd does not have a filename" })?.to_string();
					if !textures.contains_key(&img_filename) {
						let img_path = obj_parent_dir.join(img_filename.trim());
						let txt = load_texture(display, img_path.as_path(), true)?;
						textures.insert(img_filename.clone(), txt);
					}
					mtl.diffuse_texture = Some(img_filename.clone());
				},
				"Kd" => {
					for i in 0..3 {
						mtl.color[i] = split.next()
							.ok_or(WavefrontLoadError::FormatError { msg: "Kd is missing a component" })?
							.trim().parse()?;
					}
				},
				&_ => ()
			}
		}
		
		line.clear();
	}

	Ok(())
}

fn process_obj(display: Option<&&Display>, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
	current_mtl: &Option<MtlInfo>, quadoctree: Option<&mut &mut QuadOctreeNode>, lights: Option<&mut &mut HashMap<String, Light>>,
	o_name: &mut Option<String>, result: &mut BTreeMap<String, ObjDef>) -> Result<(), WavefrontLoadError> {
	let mesh_type = if o_name.as_ref().unwrap().starts_with(COLLISION_PREFIX) {
		MeshType::Collision
	} else if o_name.as_ref().unwrap().starts_with(LIGHT_PREFIX) {
		MeshType::Light
	} else if o_name.as_ref().unwrap().starts_with(TERRAIN_PREFIX) {
		MeshType::Terrain
	} else { MeshType::Normal };

	if let Some(display) = display {
		if MeshType::Normal == mesh_type || MeshType::Terrain == mesh_type {
			let mut def = load_data_to_gpu(*display, &vertices, &indices);
			def.material = Some(current_mtl.as_ref().unwrap().clone());
			result.insert(o_name.as_ref().unwrap().clone(), def);
		}
	}

	if MeshType::Terrain == mesh_type || MeshType::Collision == mesh_type {
		if let Some(quadoctree) = quadoctree {
			add_obj_to_quadoctree(&mut (**quadoctree), &vertices, &indices, MeshType::Collision == mesh_type)?;
		}
	}

	if MeshType::Light == mesh_type {
		if let Some(lights) = lights {
			lights.insert(o_name.as_ref().unwrap().clone(), Light { position: vertices[0].position, ..Default::default() });
		}
	}

	vertices.clear();
	indices.clear();
	*o_name = None;
	Ok(())
}

pub fn load_obj(filename: &str, app_id: &str, display: Option<&Display>, mut textures: Option<&mut HashMap<String, Texture2d>>,
	scale: &[f32; 3], mut quadoctree: Option<&mut QuadOctreeNode>,
	mut lights: Option<&mut HashMap<String, Light>>) -> Result<BTreeMap<String, ObjDef>, WavefrontLoadError> {
	let path = find_asset(filename, app_id);
	let f = File::open(path.as_path())?;
	let mut f = BufReader::new(f);

	let mut line = String::new();

	let mut vertex_info: Vec<[f32; 3]> = Vec::new();
	let mut normal_info: Vec<[f32; 3]> = Vec::new();
	let mut texcoord_info: Vec<[f32; 2]> = Vec::new();

	let mut vertices: Vec<Vertex> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	let mut mtl_map: HashMap<String, MtlInfo> = HashMap::new();
	let mut current_mtl: Option<MtlInfo> = None;
	let mut current_o_name: Option<String> = None;

	let mut result: BTreeMap<String, ObjDef> = BTreeMap::new();

	while f.read_line(&mut line)? != 0 {
		let mut split = line.split(' ');

		match split.next().unwrap() {
			"mtllib" => {
				if let Some(display) = display.as_ref() {
					let parent_dir = path.parent().unwrap();
					load_mtl(*display, &mut split, &parent_dir, *textures.as_mut().unwrap(), &mut mtl_map)?;
				}
			},
			"usemtl" => {
				if display.is_some() {
					let mtl_name = split.next()
						.ok_or(WavefrontLoadError::FormatError { msg: "usemtl does not have a name" })?;
					current_mtl = Some(mtl_map.get(mtl_name)
						.ok_or(WavefrontLoadError::FormatError { msg: "Material does not exist" })?.clone());
				}
			},
			"v" => vertex_info.push(parse_vertex_or_normal(&mut split, scale)?),
			"vn" => normal_info.push(parse_vertex_or_normal(&mut split, &[1., 1., 1.])?),
			"vt" => texcoord_info.push(parse_texcoords(&mut split)?),
			"f" => parse_face(&mut split, &vertex_info, &normal_info, &texcoord_info, &mut vertices,
				&mut indices)?,
			"o" => {
				if current_o_name.is_some() {
					process_obj(display.as_ref(), &mut vertices, &mut indices, &current_mtl,
						quadoctree.as_mut(), lights.as_mut(), &mut current_o_name, &mut result)?;
				}
				current_o_name = Some(split.next()
					.ok_or(WavefrontLoadError::FormatError { msg: "o does not have a name" })?.to_string());
			}
			&_ => ()
		}
		line.clear();
	}

	if current_o_name.is_some() {
		process_obj(display.as_ref(), &mut vertices, &mut indices, &current_mtl,
			quadoctree.as_mut(), lights.as_mut(), &mut current_o_name, &mut result)?;
	}

	Ok(result)
}


