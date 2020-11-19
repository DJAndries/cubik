use std::io;
use std::str::Split;
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::draw::{ObjDef, Vertex, load_data_to_gpu, MtlInfo};
use glium::{Display, texture::{Texture2d, RawImage2d, TextureCreationError}};
use derive_more::{Error, From};
use crate::quadoctree::{QuadOctreeNode, QuadOctreeError, build_quadoctree_from_triangles};

#[derive(Debug, derive_more::Display, Error, From)]
pub enum WavefrontLoadError {
	#[from(ignore)]
	FormatError { msg: &'static str },
	#[from(ignore)]
	BadIndexError { msg: &'static str },
	IOError(io::Error),
	FloatParseError(std::num::ParseFloatError),
	IntParseError(std::num::ParseIntError),
	TextureImageLoadError(image::error::ImageError),
	TextureUploadError(TextureCreationError),
	QuadOctreeCreateError(QuadOctreeError)
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
	texcoord_info: &Vec<[f32; 2]>, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
	face_index_map: &mut HashMap<[u32; 3], u32>) -> Result<(), WavefrontLoadError> {

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

		face_indices.push(match face_index_map.get(&face_index_ref) {
			None => {
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
				(vertices.len() - 1) as u32
			},
			Some(face_index) => *face_index
		});
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

fn load_texture(display: &Display, path: &Path) -> Result<Texture2d, WavefrontLoadError> {
	let f = File::open(path.clone())?;
	let f = BufReader::new(f);

	let image = image::load(f, image::ImageFormat::from_path(path.clone())?)?.to_rgba();
	let image_dim = image.dimensions();
	let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);
	Ok(Texture2d::new(display, image)?)
}

fn load_mtl(display: &Display, obj_split: &mut Split<char>, obj_parent_dir: &Path,
	materials: &mut Vec<MtlInfo>, mtl_name_map: &mut HashMap<String, u16>) -> Result<(), WavefrontLoadError> {
	let filename = obj_split.next()
		.ok_or(WavefrontLoadError::FormatError { msg: "mtllib does not have filename" })?;
	
	let f = File::open(obj_parent_dir.join(filename.trim()).as_path())?;
	let mut f = BufReader::new(f);

	let mut line = String::new();

	let mut material_index: Option<u16> = None;

	while f.read_line(&mut line)? != 0 {
		let mut split = line.split(' ');

		let key = split.next().unwrap().trim();

		if key == "newmtl" {
			materials.push(Default::default());
			material_index = Some((materials.len() - 1) as u16);
			let name = split.next()
					.ok_or(WavefrontLoadError::FormatError { msg: "newmtl does not have a name" })?.to_string();
			mtl_name_map.insert(name, (materials.len() - 1) as u16);
		}

		if let Some(i) = material_index {
			match key {
				"map_Kd" => {
					let img_filename = split.next()
						.ok_or(WavefrontLoadError::FormatError { msg: "map_Kd does not have a filename" })?;
					let img_path = obj_parent_dir.join(img_filename.trim());
					materials[i as usize].diffuse_texture = Some(load_texture(display, img_path.as_path())?);
				},
				&_ => ()
			}
		}
		
		line.clear();
	}

	Ok(())
}

pub fn load_obj(filename: &str, display: &Display, materials: &mut Vec<MtlInfo>,
	scale: &[f32; 3], quadoctree: Option<&mut QuadOctreeNode>) -> Result<ObjDef, WavefrontLoadError> {
	let f = File::open(filename)?;
	let mut f = BufReader::new(f);

	let mut line = String::new();

	let mut vertex_info: Vec<[f32; 3]> = Vec::new();
	let mut normal_info: Vec<[f32; 3]> = Vec::new();
	let mut texcoord_info: Vec<[f32; 2]> = Vec::new();

	let mut vertices: Vec<Vertex> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	let mut face_index_map: HashMap<[u32; 3], u32> = HashMap::new();
	let mut mtl_name_map: HashMap<String, u16> = HashMap::new();
	let mut current_mtl: Option<u16> = None;

	while f.read_line(&mut line)? != 0 {
		let mut split = line.split(' ');

		match split.next().unwrap() {
			"mtllib" => {
				let parent_dir = Path::new(filename).parent().unwrap();
				load_mtl(display, &mut split, &parent_dir, materials, &mut mtl_name_map)?;
			},
			"usemtl" => {
				let mtl_name = split.next()
					.ok_or(WavefrontLoadError::FormatError { msg: "usemtl does not have a name" })?;
				current_mtl = Some(mtl_name_map.get(mtl_name)
					.ok_or(WavefrontLoadError::FormatError { msg: "Material does not exist" })?.clone());
			},
			"v" => vertex_info.push(parse_vertex_or_normal(&mut split, scale)?),
			"vn" => normal_info.push(parse_vertex_or_normal(&mut split, &[1., 1., 1.])?),
			"vt" => texcoord_info.push(parse_texcoords(&mut split)?),
			"f" => parse_face(&mut split, &vertex_info, &normal_info, &texcoord_info, &mut vertices,
				&mut indices, &mut face_index_map)?,
			&_ => ()
		}
		line.clear();
	}

	let mut result = load_data_to_gpu(display, &vertices, &indices);

	result.material_index = current_mtl;

	if let Some(quadoctree) = quadoctree {
		build_quadoctree_from_triangles(quadoctree, &vertices, &indices)?;
	}
	
	Ok(result)
}
