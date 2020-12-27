mod support;

use cubik::glium::{self, glutin, Surface, texture::Texture2d};
use cubik::draw::{ObjDef, ObjDrawInfo, EnvDrawInfo, basic_render, MtlInfo, MAX_LIGHTS};
use cubik::camera::perspective_matrix;
use cubik::cube::load_cube;
use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::math::add_vector;
use cubik::skybox::Skybox;
use cubik::animation::ObjAnimation;
use cubik::player::{Player, PlayerControlType};
use cubik::peer_player::PeerPlayer;
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use support::constants::APP_ID;
use cubik::audio::{buffer_sound, get_sound_stream, play_sound_from_file};
use cubik::shaders;
use cubik::container::RenderContainer;
use std::collections::HashMap;
use cubik::client::ClientContainer;
use support::msg::AppMessage;

const PORT: u16 = 27020;

fn net_update(client_container: &mut ClientContainer<AppMessage>, peer_map: &mut HashMap<u8, PeerPlayer>, player: &mut Player) {
	let pids = client_container.pids();
	peer_map.retain(|&k, _| pids.contains(&k));

	client_container.update().unwrap();

	for msg in client_container.get_msgs() {
		if let AppMessage::PlayerChange { msg, player_id } = msg {
			if client_container.player_id.unwrap_or(0) == player_id {
				client_container.send(AppMessage::PlayerChange {
					player_id: 0,
					msg: player.update(0., None, None, Some(msg)).unwrap()
				}).unwrap();
			} else {
				let mut peer_player = peer_map.entry(player_id)
					.or_insert(PeerPlayer::new());

				peer_player.update(msg);
			}
		}
	}
}

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

	let sound_stream = get_sound_stream().unwrap();

	let mut peer_map: HashMap<u8, PeerPlayer> = HashMap::new();

	let mut client_container: ClientContainer<AppMessage> = ClientContainer::new(format!("127.0.0.1:{}", PORT).as_str()).unwrap();
	let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::MultiplayerClient,
		[0.0, 0.275, 0.0], [0.44, 0.275, 0.08]);

	let mut lights: Vec<[f32; 3]> = Vec::new();

	player.walking_sound = Some(buffer_sound("./audio/running.wav", APP_ID).unwrap());
	
	let map_obj = cubik::wavefront::load_obj("models/map2.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
		&[1., 1., 1.], None, Some(&mut lights)).unwrap();

	let wolf_standing = cubik::wavefront::load_obj("models/wolf_standing.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
		&[1., 1., 1.], None, None).unwrap();
	let mut wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", APP_ID, &ctr.display, &mut ctr.textures, 0.041).unwrap();

	let skybox = Skybox::new(&ctr.display, "skybox1", APP_ID, 512, 50.).unwrap();

	let mut lights_arr: [[f32; 3]; MAX_LIGHTS] = Default::default();
	for i in 0..lights.len() { lights_arr[i] = lights[i]; }

	let mut displace = 0.0f32;

	let mut start_time = std::time::Instant::now();
	let mut last_frame_time = std::time::Instant::now();

	let mut input_enabled = true;

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut InputListener> = vec![&mut player];
		*control_flow = glutin::event_loop::ControlFlow::Poll;
		match ev {
			glutin::event::Event::WindowEvent { event, .. } => match event {
				glutin::event::WindowEvent::CloseRequested => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
					return;
				},
				glutin::event::WindowEvent::KeyboardInput { input, .. } => {
					if let Some(keycode) = input.virtual_keycode {
						match keycode {
							glutin::event::VirtualKeyCode::Escape => {
								*control_flow = glutin::event_loop::ControlFlow::Exit;
								return;
							},
							glutin::event::VirtualKeyCode::T => {
								if input.state == glutin::event::ElementState::Released {
									input_enabled = !input_enabled;
									let gl_window = ctr.display.gl_window();
									let window = gl_window.window();
									window.set_cursor_visible(!input_enabled);
								}
								return;
							}
							_ => ()
						};
					}
					if !input_enabled { return; }
					process_input_event(event, listeners, &ctr.display);
					return;
				},
				_ => {
					if !input_enabled { return; }
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
		
		net_update(&mut client_container, &mut peer_map, &mut player);

		let new_frame_time = std::time::Instant::now();
		let time_delta = new_frame_time.duration_since(last_frame_time).as_secs_f32();
		last_frame_time = new_frame_time;

		displace += time_delta;

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

		target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

		for (key, o) in &map_obj {
			let text_displace = if key.starts_with("water") {
				Some([displace.sin() * 0.005, displace.sin() * 0.005])
			} else { None };
			basic_render(&mut target, &env_info, &map_info, &o, &ctr.main_program, text_displace);
		}

		for peer_player in peer_map.values_mut() {
			peer_player.draw(&mut target, &env_info, &ctr.main_program, &wolf_anim, &wolf_standing, wolf_anim.get_keyframe_by_index(5));
		}

		skybox.draw(&mut target, &env_info, &ctr.skybox_program);

		target.finish().unwrap();
	});
}