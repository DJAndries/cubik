use cubik::glium::{self, glutin, Surface, texture::Texture2d};
use cubik::draw::{ObjDef, ObjDrawInfo, EnvDrawInfo, basic_render, MtlInfo, MAX_LIGHTS};
use cubik::camera::perspective_matrix;
use cubik::cube::load_cube;
use cubik::input::{InputListener, process_input_event};
use cubik::quadoctree::{QuadOctreeNode, BoundingBox};
use cubik::math::add_vector;
use cubik::skybox::Skybox;
use cubik::animation::ObjAnimation;
use cubik::player::{Player, PlayerControlType};
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use cubik::ui::{MainMenu, MainMenuAction};
use cubik::audio::{buffer_sound, get_sound_stream, play_sound_from_file};
use cubik::shaders;
use cubik::container::RenderContainer;
use std::collections::HashMap;

fn main() {
	let event_loop = glutin::event_loop::EventLoop::new();
	let ctr = RenderContainer::new(&event_loop, 1280, 720, true);

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

	let sound_stream = get_sound_stream().unwrap();

	let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::Singleplayer);
	let mut textures: HashMap<String, Texture2d> = HashMap::new();

	play_sound_from_file(&sound_stream, "./audio/ding.wav").unwrap();
	player.walking_sound = Some(buffer_sound("./audio/running.wav").unwrap());
	
	let mut quadoctree = QuadOctreeNode::new_tree(BoundingBox {
		start_pos: [-25., -25., -25.],
		end_pos: [25., 25., 25.]
	}, false);
	let mut lights: Vec<[f32; 3]> = Vec::new();

	let map_obj = cubik::wavefront::load_obj("models/map2.obj", Some(&ctr.display), &mut textures,
		&[1., 1., 1.], Some(&mut quadoctree), Some(&mut lights)).unwrap();

	let mut wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", &ctr.display, &mut textures, 0.041).unwrap();

	let skybox = Skybox::new(&ctr.display, "skybox1", 512, 50.).unwrap();

	let mut lights_arr: [[f32; 3]; MAX_LIGHTS] = Default::default();
	for i in 0..lights.len() { lights_arr[i] = lights[i]; }

	let mut displace = 0.0f32;

	let mut main_menu = MainMenu::new(&ctr.display).unwrap();
	main_menu.enabled = false;

	let mut start_time = std::time::Instant::now();
	let mut last_frame_time = std::time::Instant::now();

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut InputListener> = vec![&mut main_menu, &mut player];
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
					process_input_event(event, listeners, &ctr.display);
					return;
				},
				_ => {
					process_input_event(event, listeners, &ctr.display);
					return;
				}
			},
			glutin::event::Event::NewEvents(cause) => match cause {
				glutin::event::StartCause::ResumeTimeReached { .. } => (),
				glutin::event::StartCause::Init => {
					let gl_window = ctr.display.gl_window();
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

		displace += time_delta;

		player.update(time_delta, &quadoctree, &sound_stream, None);

		let mut target = ctr.display.draw();

		let perspective_mat = perspective_matrix(&mut target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: player.camera.view_matrix(),
			lights: lights_arr,
			light_count: lights.len(),
			params: &ctr.params
		};

		if (main_menu.enabled) {
			target.clear_color_and_depth((0., 0., 0., 1.0), 1.0); 

			if let Some(result) = main_menu.draw(&mut target, &ctr.display, &ctr.ui_program).unwrap() {
				match result {
					MainMenuAction::Quit => {
						target.finish().unwrap();
						*control_flow = glutin::event_loop::ControlFlow::Exit;
						return;
					},
					MainMenuAction::Start => main_menu.enabled = false
				};
			}
		} else {
			target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

			for (key, o) in &map_obj {
				let text_displace = if key.starts_with("water") {
					Some([displace.sin() * 0.005, displace.sin() * 0.005])
				} else { None };
				basic_render(&mut target, &env_info, &map_info, &o, &ctr.main_program, &textures, text_displace);
			}

			for o in wolf_anim.get_keyframe(start_time.elapsed().as_secs_f32()).values() {
				basic_render(&mut target, &env_info, &wolf_info, &o, &ctr.main_program, &textures, None);
			}

			skybox.draw(&mut target, &env_info, &ctr.skybox_program);
		}

		target.finish().unwrap();
	});
}
