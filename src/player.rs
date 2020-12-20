use crate::cube::generate_cube_collideobj;
use crate::quadoctree::CollisionObj;
use crate::math::{normalize_vector, cross_product, add_vector};
use crate::input::InputListener;
use crate::camera::{Camera, UP};
use crate::collision::check_player_collision;
use crate::quadoctree::QuadOctreeNode;
use glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use glium::Display;
use glium::glutin::dpi::PhysicalPosition;
use serde::{Serialize, Deserialize};
use crate::audio::{SoundData, SoundStream, create_sink, sound_decoder_from_data_looped};
use rodio::Sink;

const PLAYER_CUBE_DIM: [f32; 3] = [0.15, 0.2, 0.15];
const MOVE_RATE: f32 = 1.28;
const MOUSE_SENSITIVITY: f32 = 1.8;
const GRAVITY: f32 = 1.8;
const EYE_HEIGHT: f32 = 0.38;
const JUMP_VELOCITY: f32 = 0.9;

pub enum PlayerControlType {	
	MultiplayerServer,
	MultiplayerClient,
	Singleplayer
}

#[derive(Serialize, Deserialize)]
pub enum PlayerControlMessage {
	Server {
		position: [f32; 3],
		pitch_yaw: (f32, f32),
		is_colliding: bool,
		is_moving: bool
	},
	Client {
		pitch_yaw: (f32, f32),
		input_state: PlayerInputState
	}
}

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct PlayerInputState {
	move_forward: bool,
	move_left: bool,
	move_right: bool,
	move_back: bool,
	jump: bool
}

pub struct Player {
	pub control_type: PlayerControlType,

	pub camera: Camera,
	pub player_cube: CollisionObj,
	pub velocity: [f32; 3],
	pub noclip: bool,

	pub walking_sound: Option<SoundData>,
	walking_sound_sink: Option<Sink>,

	is_colliding: bool,
	is_moving: bool,

	input_state: PlayerInputState,

	mouse_diff: Option<(f32, f32)>
}

impl Player {
	pub fn new(position: [f32; 3], control_type: PlayerControlType) -> Self {
		Self {
			control_type: control_type,
			camera: Camera::new(position),
			player_cube: generate_cube_collideobj(&position, &PLAYER_CUBE_DIM),
			velocity: [0., 0., 0.],
			noclip: false,
			is_colliding: false,
			is_moving: false,
			input_state: Default::default(),
			mouse_diff: None,
			walking_sound: None,
			walking_sound_sink: None
		}
	}

	fn input_update(&mut self, time_delta: f32, sound_stream: &SoundStream) {
		let move_len = MOVE_RATE * time_delta;

		let move_dir = if !self.noclip {
			[self.camera.direction[0], 0., self.camera.direction[2]]
		} else {
			// "no clip" mode
			self.camera.direction
		};
		let mut move_vec = [0., 0., 0.0f32];

		let right = normalize_vector(&cross_product(&UP, &move_dir));
		let up = cross_product(&move_dir, &right);
		let direction_perp = cross_product(&move_dir, &up);
		if self.input_state.move_forward { move_vec = add_vector(&move_vec, &move_dir, 1.0); }
		if self.input_state.move_back { move_vec = add_vector(&move_vec, &move_dir, -1.0); }
		if self.input_state.move_left { move_vec = add_vector(&move_vec, &direction_perp, 1.0); }
		if self.input_state.move_right { move_vec = add_vector(&move_vec, &direction_perp, -1.0); }
		self.camera.position = add_vector(&self.camera.position, &move_vec, move_len);
		self.player_cube = generate_cube_collideobj(&self.camera.position, &PLAYER_CUBE_DIM);

		if let Some(mouse_diff) = self.mouse_diff {
			self.camera.pitch_yaw.1 -= mouse_diff.0 * MOUSE_SENSITIVITY;
			self.camera.pitch_yaw.0 += (mouse_diff.1 * MOUSE_SENSITIVITY).min(1.57).max(-1.57);
			self.camera.pitch_yaw.0 = self.camera.pitch_yaw.0.min(1.57).max(-1.57);
			
			self.camera.update_direction();
			self.mouse_diff = None;
		}

		self.is_moving = move_vec != [0., 0., 0.0f32];
	}

	fn update_sound(&mut self, sound_stream: &SoundStream) {
		let is_walking = self.is_moving && self.is_colliding;
		if let Some(sound) = self.walking_sound.as_ref() {
			if let Some(sink) = self.walking_sound_sink.as_ref() {
				if !is_walking {
					sink.stop();
					self.walking_sound_sink = None;
				}
			} else if is_walking {
				let sink = create_sink(sound_stream).unwrap();
				sink.append(sound_decoder_from_data_looped(sound).unwrap());
				sink.play();
				self.walking_sound_sink = Some(sink);
			}
		}
	}

	fn fix_velocity(&mut self, correction_vec: &[f32; 3]) {
		for i in 0..3 {
			if (correction_vec[i] > 0.01 && self.velocity[i] < 0.) || (correction_vec[i] < -0.01 && self.velocity[i] > 0.) {
				self.velocity[i] = 0.;
			}
		}
	}

	fn maybe_jump(&mut self) {
		if !self.input_state.jump { return; }
		self.camera.position[1] += EYE_HEIGHT * 0.05;
		self.velocity[1] = JUMP_VELOCITY;
	}

	fn collision_gravity_update(&mut self, time_delta: f32, quadoctree: &QuadOctreeNode) {
		self.is_colliding = false;

		if self.noclip { return; }

		let collide_result = check_player_collision(&quadoctree, &self.camera.position, &self.player_cube);

		self.velocity[1] -= GRAVITY * time_delta;

		for poly_collide in &collide_result.polygons {
			self.camera.position = add_vector(&self.camera.position, &poly_collide, 1.);
			self.fix_velocity(&poly_collide);
			self.maybe_jump();
			self.is_colliding = true;
		}
		if let Some(tri_intersect) = collide_result.triangle {
			if self.camera.position[1] < tri_intersect[1] + EYE_HEIGHT * 1.05 {
				self.camera.position[1] = tri_intersect[1] + EYE_HEIGHT;
				self.fix_velocity(&[0.0, 1.0, 0.0]);
				self.maybe_jump();
				self.is_colliding = true;
			}
		}

		if self.velocity != [0., 0., 0.] {
			self.is_moving = true;
		}
		self.camera.position = add_vector(&self.camera.position, &self.velocity, time_delta);
	}

	pub fn update(&mut self, time_delta: f32, quadoctree: &QuadOctreeNode, sound_stream: &SoundStream, incoming_msg: Option<PlayerControlMessage>) -> Option<PlayerControlMessage> {
		match self.control_type {
			PlayerControlType::MultiplayerServer => {
				if let Some(incoming_msg) = incoming_msg {
					if let PlayerControlMessage::Client { input_state, pitch_yaw } = incoming_msg {
						self.input_state = input_state;
						self.camera.pitch_yaw = pitch_yaw;
					}
				}
				self.input_update(time_delta, sound_stream);
				self.collision_gravity_update(time_delta, quadoctree);
				Some(PlayerControlMessage::Server {
					position: self.camera.position,
					pitch_yaw: self.camera.pitch_yaw,
					is_moving: self.is_moving,
					is_colliding: self.is_colliding
				})
			},
			PlayerControlType::MultiplayerClient => {
				if let Some(incoming_msg) = incoming_msg {
					if let PlayerControlMessage::Server { position, pitch_yaw, is_moving, is_colliding } = incoming_msg {
						self.is_moving = is_moving;
						self.is_colliding = is_colliding;
						self.camera.position = position;
						self.camera.pitch_yaw = pitch_yaw;
					}
				}
				self.update_sound(sound_stream);
				Some(PlayerControlMessage::Client { input_state: self.input_state, pitch_yaw: self.camera.pitch_yaw })
			},
			PlayerControlType::Singleplayer => {
				self.input_update(time_delta, sound_stream);
				self.collision_gravity_update(time_delta, quadoctree);
				self.update_sound(sound_stream);
				None
			}
		}
	}
}

impl InputListener for Player {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if let Some(key) = key {
			match key {
				VirtualKeyCode::W => self.input_state.move_forward = pressed,
				VirtualKeyCode::A => self.input_state.move_left = pressed,
				VirtualKeyCode::D => self.input_state.move_right = pressed,
				VirtualKeyCode::S => self.input_state.move_back = pressed,
				VirtualKeyCode::Space => self.input_state.jump = pressed,
				VirtualKeyCode::N => {
					if !pressed {
						self.noclip = !self.noclip;
					}
				}
				_ => return false
			}
			return true;
		}
		false
	}

	fn handle_mouse_pos_ev(&mut self, new_pos: (f32, f32), display: &Display) -> bool {
		let gl_window = display.gl_window();
		let window = gl_window.window();
		let winsize = window.inner_size();
		let middle = ((winsize.width / 2) as f32, (winsize.height / 2) as f32);

		self.mouse_diff = Some(new_pos);
		window.set_cursor_position(PhysicalPosition::new(middle.0, middle.1));
		return true;
	}

	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool {
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		false
	}
}
