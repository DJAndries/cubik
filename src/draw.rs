use glium::{Display, Frame, Surface, DrawParameters, Program, VertexBuffer, IndexBuffer, texture::Texture2d};
use crate::math::mult_matrix;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub texcoords: [f32; 2]
}

implement_vertex!(Vertex, position, normal, texcoords);

#[derive(Default)]
pub struct MtlInfo {
	pub diffuse_texture: Option<Texture2d>
}

pub struct EnvDrawInfo<'a> {
	pub view_mat: [[f32; 4]; 4],
	pub perspective_mat: [[f32; 4]; 4],
	pub params: &'a DrawParameters<'a>,
	pub light_loc: [f32; 3]
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
	pub material_index: Option<u16>
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
		material_index: None
	}
}

pub fn basic_render(target: &mut Frame, env_info: &EnvDrawInfo, obj_info: &ObjDrawInfo, obj_def: &ObjDef, program: &Program, materials: &Vec<MtlInfo>) {
	let uniforms = uniform! {
		model: *obj_info.model_mat.as_ref().unwrap(),
		view: env_info.view_mat,
		perspective: env_info.perspective_mat,
		u_light: env_info.light_loc,
		shape_color: obj_info.color,
		tex: materials[obj_def.material_index.unwrap() as usize].diffuse_texture.as_ref().unwrap()
	};
	target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, env_info.params).unwrap();
}
