mod support;

use cubik::glium::{glutin, Surface};
use cubik::draw::{ObjDrawInfo, EnvDrawInfo, basic_render, MAX_LIGHTS, Light};
use cubik::camera::perspective_matrix;
use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::skybox::Skybox;
use cubik::animation::ObjAnimation;
use cubik::player::{Player, PlayerControlType};
use cubik::peer_player::PeerPlayer;
use support::constants::APP_ID;
use cubik::audio::{buffer_sound, get_sound_stream, SoundStream};
use cubik::container::RenderContainer;
use std::collections::HashMap;
use cubik::client::ClientContainer;
use cubik::map::GameMap;
use support::msg::AppMessage;

const PORT: u16 = 27020;

fn net_update(client_container: &mut ClientContainer<AppMessage>, peer_map: &mut HashMap<u8, PeerPlayer>, player: &mut Player, sound_stream: &SoundStream, time_delta: f32) {
	let pids = client_container.pids();
	peer_map.retain(|&k, _| pids.contains(&k));

	client_container.update().unwrap();

	for msg in client_container.get_msgs() {
		if let AppMessage::PlayerChange { msg, player_id } = msg {
			if client_container.player_id.unwrap_or(0) == player_id {
				player.update(0., None, Some(sound_stream), Some(msg));
			} else {
				let peer_player = peer_map.entry(player_id)
					.or_insert(PeerPlayer::new());

				peer_player.update(Some(msg), time_delta);
			}
		}
	}

	for peer_player in peer_map.values_mut() {
		peer_player.update(None, time_delta);
	}

	if let Some(out_msg) = player.update(time_delta, None, Some(sound_stream), None) {
		client_container.send(AppMessage::PlayerChange {
			player_id: 0,
			msg: out_msg
		}).unwrap();
	}
}

fn main() {
	let event_loop = glutin::event_loop::EventLoop::new();
	let mut ctr = RenderContainer::new(&event_loop, 1280, 720, "Example", false);

	let mut map_info: ObjDrawInfo = Default::default();
	map_info.generate_matrix();

	let sound_stream = get_sound_stream().unwrap();

	let mut peer_map: HashMap<u8, PeerPlayer> = HashMap::new();

	let mut client_container: ClientContainer<AppMessage> = ClientContainer::new(format!("127.0.0.1:{}", PORT).as_str()).unwrap();
	let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::MultiplayerClient,
		[0.0, 0.275, 0.0], [0.44, 0.275, 0.08]);

	player.walking_sound = Some(buffer_sound("./audio/running.wav", APP_ID).unwrap());
	
	let map = GameMap::load_map("models/map2", APP_ID,
		Some(&ctr.display), Some(&mut ctr.textures), false).unwrap();

	let wolf_standing = cubik::wavefront::load_obj("models/wolf_standing.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
		&[1., 1., 1.], None, None, None).unwrap();
	let wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", APP_ID, &ctr.display, &mut ctr.textures, 0.041).unwrap();

	let skybox = Skybox::new(&ctr.display, "skybox1", APP_ID, 512, 50.).unwrap();

	let mut lights_arr: [Light; MAX_LIGHTS] = Default::default();
	let mut light_iter = map.lights.values();
	for i in 0..map.lights.len() { lights_arr[i] = *light_iter.next().unwrap(); }

	let mut displace = 0.0f32;

	let mut last_frame_time = std::time::Instant::now();

	let mut input_enabled = true;

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut dyn InputListener> = vec![&mut player];
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
		
		let new_frame_time = std::time::Instant::now();
		let time_delta = new_frame_time.duration_since(last_frame_time).as_secs_f32();
		last_frame_time = new_frame_time;

		displace += time_delta;

		net_update(&mut client_container, &mut peer_map, &mut player, &sound_stream, time_delta);

		let mut target = ctr.display.draw();

		let perspective_mat = perspective_matrix(&mut target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: player.camera.view_matrix(),
			lights: lights_arr,
			light_count: map.lights.len(),
			params: &ctr.params,
			textures: &ctr.textures
		};

		target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

		for (key, o) in &map.objects {
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
