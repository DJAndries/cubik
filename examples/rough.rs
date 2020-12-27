mod support;

use cubik::glium::{glutin, Surface};
use cubik::draw::{ObjDrawInfo, EnvDrawInfo, basic_render, MAX_LIGHTS};
use cubik::camera::perspective_matrix;

use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::quadoctree::{QuadOctreeNode, BoundingBox};

use cubik::skybox::Skybox;
use cubik::animation::ObjAnimation;
use cubik::player::{Player, PlayerControlType};

use support::ui::{MainMenu, MainMenuAction};
use support::constants::APP_ID;
use cubik::audio::{buffer_sound, get_sound_stream, play_sound_from_file};

use cubik::container::RenderContainer;


fn main() {
	let event_loop = glutin::event_loop::EventLoop::new();
	let mut ctr = RenderContainer::new(&event_loop, 1280, 720, "Example", false);

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

	let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::Singleplayer,
		[-0.28, 0.275, 0.0], [0.44, 0.275, 0.08]);

	play_sound_from_file(&sound_stream, "./audio/ding.wav", APP_ID).unwrap();
	player.walking_sound = Some(buffer_sound("./audio/running.wav", APP_ID).unwrap());
	
	let mut quadoctree = QuadOctreeNode::new_tree(BoundingBox {
		start_pos: [-25., -25., -25.],
		end_pos: [25., 25., 25.]
	}, false);
	let mut lights: Vec<[f32; 3]> = Vec::new();

	let map_obj = cubik::wavefront::load_obj("models/map2.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
		&[1., 1., 1.], Some(&mut quadoctree), Some(&mut lights)).unwrap();

	let wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", APP_ID, &ctr.display, &mut ctr.textures, 0.041).unwrap();

	let skybox = Skybox::new(&ctr.display, "skybox1", APP_ID, 512, 50.).unwrap();

	let mut lights_arr: [[f32; 3]; MAX_LIGHTS] = Default::default();
	for i in 0..lights.len() { lights_arr[i] = lights[i]; }

	let mut displace = 0.0f32;

	let mut main_menu = MainMenu::new(&ctr.display).unwrap();
	main_menu.enabled = false;

	let start_time = std::time::Instant::now();
	let mut last_frame_time = std::time::Instant::now();

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut dyn InputListener> = vec![&mut main_menu, &mut player];
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
					center_cursor(&ctr.display, false);
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

		player.update(time_delta, Some(&quadoctree), Some(&sound_stream), None);

		let mut target = ctr.display.draw();

		let perspective_mat = perspective_matrix(&mut target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: player.camera.view_matrix(),
			lights: lights_arr,
			light_count: lights.len(),
			params: &ctr.params,
			textures: &ctr.textures
		};

		if main_menu.enabled {
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
				basic_render(&mut target, &env_info, &map_info, &o, &ctr.main_program, text_displace);
			}

			for o in wolf_anim.get_keyframe(start_time.elapsed().as_secs_f32()).values() {
				basic_render(&mut target, &env_info, &wolf_info, &o, &ctr.main_program, None);
			}

			skybox.draw(&mut target, &env_info, &ctr.skybox_program);
		}

		target.finish().unwrap();
	});
}
