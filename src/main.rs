mod shaders;
mod camera;
mod draw;
mod cube;
mod math;
mod input;
mod wavefront;
mod quadoctree;
mod collision;
mod textures;
mod skybox;
mod animation;

#[macro_use]
extern crate glium;

use glium::{glutin, Surface, texture::Texture2d};
use crate::draw::{ObjDef, ObjDrawInfo, EnvDrawInfo, basic_render, MtlInfo};
use crate::camera::{perspective_matrix, Camera};
use crate::cube::load_cube;
use crate::input::InputState;
use crate::quadoctree::{QuadOctreeNode, BoundingBox};
use crate::collision::check_player_collision;
use crate::math::add_vector;
use crate::skybox::Skybox;
use crate::animation::ObjAnimation;
use std::collections::HashMap;

fn main() {
	let mut event_loop = glutin::event_loop::EventLoop::new();
	let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize { width: 1280, height: 720});
	let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
	let display = glium::Display::new(wb, cb, &event_loop).unwrap();

	let main_program = shaders::main_program(&display);
	let skybox_program = shaders::skybox_program(&display);

	let light_loc = [10.0, 9.0, 0.0f32];

	let mut map_info = ObjDrawInfo {
		position: [0.0, 0.0, 0.0f32],
		color: [1.0, 1.0, 1.0],
		rotation: [0.0, 0.0, 0.0f32],
		scale: [1.0, 1.0, 1.0],
		model_mat: None 
	};
	map_info.generate_matrix();

	let mut wolf_info = ObjDrawInfo {
		position: [0.4, 0.05, 0.0f32],
		color: [1.0, 1.0, 1.0],
		rotation: [0.0, 0.0, 0.0f32],
		scale: [1.0, 1.0, 1.0],
		model_mat: None 
	};
	wolf_info.generate_matrix();

	let params = glium::DrawParameters {
		depth: glium::Depth {
			test: glium::draw_parameters::DepthTest::IfLess,
			write: true,
			..Default::default()
		},
		blend: glium::draw_parameters::Blend::alpha_blending(),
		// smooth: Some(glium::draw_parameters::Smooth::Nicest),
		backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
		..Default::default()
	};

	let mut t = 0.0f32;
	let mut last_frame_time = std::time::Instant::now();
	let mut input_state: InputState = Default::default();
	let mut camera = Camera::new([0.0, 0.7, 0.0]);
	let mut textures: HashMap<String, Texture2d> = HashMap::new();
	
	let mut quadoctree = QuadOctreeNode::new_tree(BoundingBox {
		start_pos: [-25., -25., -25.],
		end_pos: [25., 25., 25.]
	}, false);

	let map_obj = crate::wavefront::load_obj("models/map2.obj", &display, &mut textures,
		&[1., 1., 1.], Some(&mut quadoctree)).unwrap();

	let mut wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", &display, &mut textures, 0.041).unwrap();

	let skybox = Skybox::new(&display, "skybox1", 512, 50.).unwrap();

	event_loop.run(move |ev, _, control_flow| {
		// let next_frame_time = std::time::Instant::now() + 
		// 	std::time::Duration::from_nanos(16_666_667);
		
		// *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
		*control_flow = glutin::event_loop::ControlFlow::Poll;
		match ev {
			glutin::event::Event::WindowEvent { event, .. } => match event {
				glutin::event::WindowEvent::CloseRequested => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
					return;
				},
				glutin::event::WindowEvent::KeyboardInput { input, .. } => {
					if let Some(glutin::event::VirtualKeyCode::Escape) = input.virtual_keycode {
						*control_flow = glutin::event_loop::ControlFlow::Exit;
						return;
					}
					input_state.update_keyboard_state(&input);
					return;
				},
				glutin::event::WindowEvent::CursorMoved { position, .. } => {
					input_state.update_mouse_state(&position, &display);
				},
				_ => return
			},
			glutin::event::Event::NewEvents(cause) => match cause {
				glutin::event::StartCause::ResumeTimeReached { .. } => (),
				glutin::event::StartCause::Init => {
					let gl_window = display.gl_window();
					let window = gl_window.window();
					let winsize = window.inner_size();
					let middle = ((winsize.width / 2) as f64, (winsize.height / 2) as f64);
					window.set_cursor_position(glium::glutin::dpi::PhysicalPosition::new(middle.0, middle.1));
					window.set_cursor_visible(false);
				},
				glutin::event::StartCause::Poll => (),
				_ => return
			},
			_ => return
		}
		let new_frame_time = std::time::Instant::now();
		let time_delta = new_frame_time.duration_since(last_frame_time).as_secs_f32();
		last_frame_time = new_frame_time;


		// t += 0.160 * time_delta;
		// cube_info.rotation[1] = t;
		// cube_info.rotation[0] = t;
		camera.update(time_delta, &mut input_state);
		wolf_anim.update(time_delta);

		// println!("pos: {:?}", camera.position[2]);
		let collide_result = check_player_collision(&quadoctree, &camera.position, &camera.player_cube);
		for poly_collide in &collide_result.polygons {
			camera.position = add_vector(&camera.position, &poly_collide, 1.);
		}
		if let Some(tri_intersect) = collide_result.triangle {
			camera.position[1] = tri_intersect[1] + 0.38;
		}

		let mut target = display.draw();

		let perspective_mat = perspective_matrix(&mut target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: camera.view_matrix(),
			light_loc: light_loc,
			params: &params
		};

		target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

		for o in map_obj.values() {
			basic_render(&mut target, &env_info, &map_info, &o, &main_program, &textures);
		}

		for o in wolf_anim.get_keyframe().values() {
			basic_render(&mut target, &env_info, &wolf_info, &o, &main_program, &textures);
		}

		skybox.draw(&mut target, &env_info, &skybox_program);

		target.finish().unwrap();
	});
}
