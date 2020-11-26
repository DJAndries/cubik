use std::collections::HashMap;
use glium::{Display, Frame, Surface, DrawParameters, Program,
	VertexBuffer, IndexBuffer, texture::Texture2d, uniforms::{Uniforms, UniformValue}};
use crate::math::mult_matrix;

pub const MAX_LIGHTS: usize = 4;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub texcoords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, texcoords);

#[derive(Default, Clone)]
pub struct MtlInfo {
	pub diffuse_texture: Option<String>
}

pub struct EnvDrawInfo<'a> {
	pub view_mat: [[f32; 4]; 4],
	pub perspective_mat: [[f32; 4]; 4],
	pub params: &'a DrawParameters<'a>,
	pub lights: [[f32; 3]; MAX_LIGHTS]
}

pub struct ObjDrawInfo {
	pub position: [f32; 3],
	pub rotation: [f32; 3],
	pub scale: [f32; 3],
	pub color: [f32; 3],
	pub model_mat: Option<[[f32; 4]; 4]>
}

pub struct ObjDef {
	pub vertices: VertexBuffer<Vertex>,
	pub indices: IndexBuffer<u32>,
	pub material: Option<MtlInfo>,
}

struct BasicDrawUniforms<'a> {
	lights: [[f32; 3]; MAX_LIGHTS],
	light_count: i32,
	model: [[f32; 4]; 4],
	view: [[f32; 4]; 4],
	perspective: [[f32; 4]; 4],
	shape_color: [f32; 3],
	texcoord_displacement: [f32; 2],
	tex: &'a Texture2d
}

impl Uniforms for BasicDrawUniforms<'_> {
	fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut func: F) {
		for i in 0..MAX_LIGHTS {
			func(&format!("lights[{}]", i).to_string(), UniformValue::Vec3(self.lights[i]));
		}
		func("light_count", UniformValue::SignedInt(self.light_count));
		func("model", UniformValue::Mat4(self.model));
		func("view", UniformValue::Mat4(self.view));
		func("perspective", UniformValue::Mat4(self.perspective));
		func("shape_color", UniformValue::Vec3(self.shape_color));
		func("texcoord_displacement", UniformValue::Vec2(self.texcoord_displacement));
		func("tex", UniformValue::Texture2d(self.tex, None));
	}
}

impl ObjDrawInfo {
	pub fn generate_matrix(&mut self) {
		let rotation_matrix = [
			[
				self.rotation[2].cos() * self.rotation[1].cos(),
				(self.rotation[2].cos() * self.rotation[1].sin() * self.rotation[0].sin())
					- (self.rotation[2].sin() * self.rotation[0].cos()),
				(self.rotation[2].cos() * self.rotation[1].sin() * self.rotation[0].cos())
					+ (self.rotation[2].sin() * self.rotation[0].sin()),
				0.0
			],
			[
				self.rotation[2].sin() * self.rotation[1].cos(),
				(self.rotation[2].sin() * self.rotation[1].sin() * self.rotation[0].sin())
					+ (self.rotation[2].cos() * self.rotation[0].cos()),
				(self.rotation[2].sin() * self.rotation[1].sin() * self.rotation[0].cos())
					- (self.rotation[2].cos() * self.rotation[0].sin()),
				0.0
			],
			[
				-self.rotation[1].sin(),
				self.rotation[1].cos() * self.rotation[0].sin(),
				self.rotation[1].cos() * self.rotation[0].cos(),
				0.0
			],
			[0.0, 0.0, 0.0, 1.0f32]
		];

		let scale_matrix = [
			[self.scale[0], 0.0, 0.0, 0.0],
			[0.0, self.scale[1], 0.0, 0.0],
			[0.0, 0.0, self.scale[2], 0.0],
			[0.0, 0.0, 0.0, 1.0]
		];

		let translate_matrix = [
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[self.position[0], self.position[1], self.position[2], 1.0f32]
		];

		self.model_mat = Some(mult_matrix(&mult_matrix(&rotation_matrix, &scale_matrix), &translate_matrix));
	}
}

pub fn load_data_to_gpu(display: &Display, vertices: &[Vertex], indices: &[u32]) -> ObjDef {
	ObjDef {
		vertices: glium::VertexBuffer::new(display, &vertices).unwrap(),
		indices: glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap(),
		material: None
	}
}

pub fn basic_render(target: &mut Frame, env_info: &EnvDrawInfo, obj_info: &ObjDrawInfo, obj_def: &ObjDef,
	program: &Program, textures: &HashMap<String, Texture2d>, texcoord_displacement: Option<[f32; 2]>) {
	let uniforms = BasicDrawUniforms {
		model: *obj_info.model_mat.as_ref().unwrap(),
		view: env_info.view_mat,
		perspective: env_info.perspective_mat,
		lights: env_info.lights,
		light_count: env_info.lights.len() as i32,
		shape_color: obj_info.color,
		texcoord_displacement: texcoord_displacement.unwrap_or([0., 0.]),
		tex: textures.get(obj_def.material.as_ref().unwrap().diffuse_texture.as_ref().unwrap()).unwrap()
	};
	target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, env_info.params).unwrap();
}
