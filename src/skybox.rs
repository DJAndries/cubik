use glium::texture::{cubemap::Cubemap, CubeLayer, TextureCreationError};
use glium::{Frame, Surface, Display, BlitTarget, framebuffer::{SimpleFrameBuffer, ValidationError}, uniforms::MagnifySamplerFilter};
use derive_more::{Error, From};
use crate::textures::{load_texture, TextureLoadError};
use crate::draw::{EnvDrawInfo, ObjDef};
use crate::cube::load_cube;
use std::path::Path;
use crate::assets::find_asset;

#[derive(Debug, derive_more::Display, Error, From)]
pub enum SkyboxError {
	CubemapCreateError(TextureCreationError),
	TextureLoadError(TextureLoadError),
	FramebufferValidationError(ValidationError)
}

pub struct Skybox {
	cubemap: Cubemap,
	obj_def: ObjDef,
	model_mat: [[f32; 4]; 4]
}

impl Skybox {
	fn load_side(&self, display: &Display, blit_target: &BlitTarget, layer: CubeLayer, skybox_name: &str, img_filename: &str, app_id: &str) -> Result<(), SkyboxError> {
		let path = Path::new("./textures").join(skybox_name).join(img_filename);
		let path = find_asset(path.to_str().unwrap(), app_id);
		let texture = load_texture(display, path.as_path(), false)?;

		let fb = SimpleFrameBuffer::new(display, self.cubemap.main_level().image(layer))?;

		texture.as_surface().blit_whole_color_to(&fb, blit_target, MagnifySamplerFilter::Linear);
		
		Ok(())
	}

	pub fn new(display: &Display, name: &str, app_id: &str, texture_dim: u32, cube_size: f32) -> Result<Self, SkyboxError> {
		let result = Self {
			obj_def: load_cube(display, &[cube_size, cube_size, cube_size], true),
			cubemap: Cubemap::empty(display, texture_dim)?,
			model_mat: [
				[1., 0., 0., 0.],
				[0., 1., 0., 0.],
				[0., 0., 1., 0.],
				[0., 0., 0., 1.]
			]
		};

		let sides = [
			(CubeLayer::PositiveX, "posx.jpg"),
			(CubeLayer::NegativeX, "negx.jpg"),
			(CubeLayer::PositiveY, "posy.jpg"),
			(CubeLayer::NegativeY, "negy.jpg"),
			(CubeLayer::PositiveZ, "posz.jpg"),
			(CubeLayer::NegativeZ, "negz.jpg")
		];

		let blit_target = glium::BlitTarget {
			left: 0,
			bottom: 0,
			width: texture_dim as i32,
			height: texture_dim as i32
		};

		for side in &sides {
			result.load_side(display, &blit_target, side.0, name, side.1, app_id)?;
		}

		Ok(result)
	}

	pub fn draw(&self, target: &mut Frame, env_info: &EnvDrawInfo, program: &glium::Program) {
		let uniforms = uniform! {
			model: self.model_mat,
			view: env_info.view_mat,
			perspective: env_info.perspective_mat,
			cubemap: self.cubemap.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
		};
		target.draw(&self.obj_def.vertices, &self.obj_def.indices, program, &uniforms, env_info.params).unwrap();
	}
}
