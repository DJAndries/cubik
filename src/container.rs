use glium::glutin::{self, event_loop::EventLoop, window::WindowBuilder, dpi::PhysicalSize};
use glium::{Display, Program, DrawParameters, texture::Texture2d};
use crate::textures::create_texture_map;
use std::collections::HashMap;
use crate::shaders;

#[cfg(windows)]
use glium::glutin::platform::windows::WindowBuilderExtWindows;

pub struct RenderContainer<'a> {
	pub display: Display,

	pub main_program: Program,
	pub skybox_program: Program,
	pub ui_program: Program,
	
	pub params: DrawParameters<'a>,

	pub textures: HashMap<String, Texture2d>
}

impl RenderContainer<'_> {
	#[cfg(windows)]
	fn create_window_builder(init_size: PhysicalSize<u32>, title: &str) -> WindowBuilder {
		glutin::window::WindowBuilder::new()
			.with_drag_and_drop(false)
			.with_title(title)
			.with_inner_size(init_size)
	}

	#[cfg(unix)]
	fn create_window_builder(init_size: PhysicalSize<u32>, title: &str) -> WindowBuilder {
		glutin::window::WindowBuilder::new()
			.with_title(title)
			.with_inner_size(init_size)
	}

	pub fn new(event_loop: &EventLoop<()>, width: usize, height: usize, title: &str, fullscreen: bool) -> Self {
		let init_size = glutin::dpi::PhysicalSize { width: width as u32, height: height as u32 };
		let wb = Self::create_window_builder(init_size, title); 
		let cb = glutin::ContextBuilder::new();
		let display = glium::Display::new(wb, cb, &event_loop).unwrap();

		let main_program = shaders::main_program(&display);
		let skybox_program = shaders::skybox_program(&display);
		let ui_program = shaders::ui_program(&display);

		let textures = create_texture_map(&display).unwrap();

		let result = Self {
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
			},
			textures: textures
		};

		result.update_size_and_mode(width, height, fullscreen);
		result
	}

	pub fn update_size_and_mode(&self, width: usize, height: usize, fullscreen: bool) {
		let gl_window = self.display.gl_window();
		let window = gl_window.window();
		let desired_size = glutin::dpi::PhysicalSize { width: width as u32, height: height as u32 };
		if fullscreen {
			let mut bit_depth = 0u16;
			let mut refresh_rate = 0u16;
			window.current_monitor().unwrap().video_modes().for_each(|v| {
				if v.bit_depth() > bit_depth { bit_depth = v.bit_depth(); }
				if v.refresh_rate() > refresh_rate { refresh_rate = v.refresh_rate(); }
			});
			let fallback_video_mode = window.current_monitor().unwrap().video_modes()
				.find(|v| v.bit_depth() == bit_depth && v.refresh_rate() == refresh_rate).unwrap();
			let video_mode =  window.current_monitor().unwrap().video_modes()
				.filter(|v| v.bit_depth() == bit_depth && v.refresh_rate() == refresh_rate)
				.find(|v| v.size() == desired_size)
				.unwrap_or(fallback_video_mode);
			window.set_fullscreen(Some(glutin::window::Fullscreen::Exclusive(video_mode)));
		} else {
			window.set_inner_size(desired_size);
			window.set_fullscreen(None);
		}
	}
}
