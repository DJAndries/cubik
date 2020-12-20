use glium::{glutin::{self, event_loop::EventLoop}, Display, Program, DrawParameters};
use crate::shaders;

pub struct RenderContainer<'a> {
	pub display: Display,

	pub main_program: Program,
	pub skybox_program: Program,
	pub ui_program: Program,
	
	pub params: DrawParameters<'a>
}

impl RenderContainer<'_> {
	pub fn new(event_loop: &EventLoop<()>, width: usize, height: usize, fullscreen: bool) -> Self {
		let wb = glutin::window::WindowBuilder::new()
			.with_inner_size(glutin::dpi::PhysicalSize { width: 1280, height: 720 })
			.with_fullscreen(if fullscreen { Some(glutin::window::Fullscreen::Borderless(None)) } else { None });
		let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
		let display = glium::Display::new(wb, cb, &event_loop).unwrap();

		let main_program = shaders::main_program(&display);
		let skybox_program = shaders::skybox_program(&display);
		let ui_program = shaders::ui_program(&display);

		Self {
			display: display,
			main_program: main_program,
			skybox_program: skybox_program,
			ui_program: ui_program,
			params: DrawParameters {
				depth: glium::Depth {
					test: glium::draw_parameters::DepthTest::IfLess,
					write: true,
					..Default::default()
				},
				blend: glium::draw_parameters::Blend::alpha_blending(),
				backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
				..Default::default()
			}
		}
	}
}
