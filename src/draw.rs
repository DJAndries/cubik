use std::collections::HashMap;
use glium::{Display, Frame, Surface, DrawParameters, Program,
	VertexBuffer, IndexBuffer, texture::{Texture2d, SrgbTexture2d}, uniforms::{Uniforms, UniformValue}};
use crate::math::{mult_matrix, mult_matrix3};

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
	pub lights: [[f32; 3]; MAX_LIGHTS],
	pub light_count: usize
}

pub struct ObjDrawInfo {
	pub position: [f32; 3],
	pub rotation: [f32; 3],
	pub scale: [f32; 3],
	pub color: [f32; 3],
	pub model_mat: Option<[[f32; 4]; 4]>
}

pub struct UIDrawInfo {
	pub position: (f32, f32),
	pub scale: (f32, f32),
	pub left_clip: f32,
	pub screen_left_clip: f32,
	pub screen_dim: (u32, u32),
	pub color: [f32; 4],
	pub model_mat: Option<[[f32; 3]; 3]>,
	pub translate_after_scale: bool
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

impl UIDrawInfo {
	pub fn new(position: (f32, f32), scale: (f32, f32)) -> UIDrawInfo {
		UIDrawInfo {
			model_mat: None,
			position: position,
			scale: scale,
			color: [1., 1., 1., 1.],
			left_clip: -4.,
			screen_left_clip: -4.,
			screen_dim: (0, 0),
			translate_after_scale: false
		}
	}

	pub fn generate_matrix(&mut self, target: &mut Frame) {
		if self.screen_dim == target.get_dimensions() {
			return;
		}
		self.screen_dim = target.get_dimensions();
		let x_scale = self.screen_dim.1 as f32 / self.screen_dim.0 as f32;
		let mut translate_mat = [
			[1., 0., 0.],
			[0., 1., 0.],
			[self.position.0, self.position.1, 1.0f32]
		];
		let scale_mat = [
			[self.scale.0 * x_scale, 0., 0.],
			[0., self.scale.1, 0.],
			[0., 0., 1.]
		];
		self.screen_left_clip = self.left_clip * x_scale;
		self.model_mat = Some(if !self.translate_after_scale {
			mult_matrix3(&translate_mat, &scale_mat)
		} else {
			translate_mat[2][0] *= x_scale;
			mult_matrix3(&scale_mat, &translate_mat)
		});
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
		light_count: env_info.light_count as i32,
		shape_color: obj_info.color,
		texcoord_displacement: texcoord_displacement.unwrap_or([0., 0.]),
		tex: textures.get(obj_def.material.as_ref().unwrap().diffuse_texture.as_ref().unwrap()).unwrap()
	};
	target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, env_info.params).unwrap();
}

pub fn ui_draw(target: &mut Frame, obj_def: &ObjDef, ui_draw_info: &UIDrawInfo, program: &Program, texture: &SrgbTexture2d) {
	let uniforms = uniform! {
		tex: texture.sampled()
			.magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
			.minify_filter(glium::uniforms::MinifySamplerFilter::Linear),
		ui_color: ui_draw_info.color,
		model: ui_draw_info.model_mat.unwrap(),
		left_clip: ui_draw_info.screen_left_clip
	};
	let params = DrawParameters {
		blend: glium::draw_parameters::Blend::alpha_blending(),
		clip_planes_bitmask: 1,
		..Default::default()
	};
	target.draw(&obj_def.vertices, &obj_def.indices, program, &uniforms, &params);
}
