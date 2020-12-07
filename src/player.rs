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
use crate::audio::{SoundData, SoundStream, create_sink, sound_decoder_from_data_looped};
use rodio::Sink;

const PLAYER_CUBE_DIM: [f32; 3] = [0.15, 0.2, 0.15];
const MOVE_RATE: f32 = 1.28;
const MOUSE_SENSITIVITY: f32 = 1.8;
const GRAVITY: f32 = 1.8;
const EYE_HEIGHT: f32 = 0.38;
const JUMP_VELOCITY: f32 = 0.9;

pub struct Player {
	pub camera: Camera,
	pub player_cube: CollisionObj,
	pub velocity: [f32; 3],
	pub noclip: bool,

	pub walking_sound: Option<SoundData>,
	walking_sound_sink: Option<Sink>,

	move_forward: bool,
	move_left: bool,
	move_right: bool,
	move_back: bool,
	jump: bool,

	mouse_diff: Option<(f32, f32)>
}

impl Player {
	pub fn new(position: [f32; 3]) -> Self {
		Self {
			camera: Camera::new(position),
			player_cube: generate_cube_collideobj(&position, &PLAYER_CUBE_DIM),
			velocity: [0., 0., 0.],
			noclip: false,
			move_forward: false,
			move_left: false,
			move_right: false,
			move_back: false,
			jump: false,
			mouse_diff: None,
			walking_sound: None,
			walking_sound_sink: None
		}
	}

	fn input_update(&mut self, time_delta: f32, sound_stream: &SoundStream) -> bool {
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
		if self.move_forward { move_vec = add_vector(&move_vec, &move_dir, 1.0); }
		if self.move_back { move_vec = add_vector(&move_vec, &move_dir, -1.0); }
		if self.move_left { move_vec = add_vector(&move_vec, &direction_perp, 1.0); }
		if self.move_right { move_vec = add_vector(&move_vec, &direction_perp, -1.0); }
		self.camera.position = add_vector(&self.camera.position, &move_vec, move_len);
		self.player_cube = generate_cube_collideobj(&self.camera.position, &PLAYER_CUBE_DIM);

		if let Some(mouse_diff) = self.mouse_diff {
			self.camera.pitch_yaw.1 -= mouse_diff.0 * MOUSE_SENSITIVITY;
			self.camera.pitch_yaw.0 += (mouse_diff.1 * MOUSE_SENSITIVITY).min(1.57).max(-1.57);
			self.camera.pitch_yaw.0 = self.camera.pitch_yaw.0.min(1.57).max(-1.57);
			
			self.camera.update_direction();
			self.mouse_diff = None;
		}

		move_vec != [0., 0., 0.0f32]
	}

	fn update_sound(&mut self, is_walking: bool, sound_stream: &SoundStream) {
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
		if !self.jump { return; }
		self.camera.position[1] += EYE_HEIGHT * 0.05;
		self.velocity[1] = JUMP_VELOCITY;
	}

	fn collision_gravity_update(&mut self, time_delta: f32, quadoctree: &QuadOctreeNode) -> bool {
		if self.noclip { return false; }

		let mut is_colliding = false;

		let collide_result = check_player_collision(&quadoctree, &self.camera.position, &self.player_cube);

		self.velocity[1] -= GRAVITY * time_delta;

		for poly_collide in &collide_result.polygons {
			self.camera.position = add_vector(&self.camera.position, &poly_collide, 1.);
			self.fix_velocity(&poly_collide);
			self.maybe_jump();
			is_colliding = true;
		}
		if let Some(tri_intersect) = collide_result.triangle {
			if self.camera.position[1] < tri_intersect[1] + EYE_HEIGHT * 1.05 {
				self.camera.position[1] = tri_intersect[1] + EYE_HEIGHT;
				self.fix_velocity(&[0.0, 1.0, 0.0]);
				self.maybe_jump();
				is_colliding = true;
			}
		}

		self.camera.position = add_vector(&self.camera.position, &self.velocity, time_delta);

		is_colliding
	}

	pub fn update(&mut self, time_delta: f32, quadoctree: &QuadOctreeNode, sound_stream: &SoundStream) {
		let is_moving = self.input_update(time_delta, sound_stream);
		let is_colliding = self.collision_gravity_update(time_delta, quadoctree);
		self.update_sound(is_moving && is_colliding, sound_stream);
	}
}

impl InputListener for Player {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if let Some(key) = key {
			match key {
				VirtualKeyCode::W => self.move_forward = pressed,
				VirtualKeyCode::A => self.move_left = pressed,
				VirtualKeyCode::D => self.move_right = pressed,
				VirtualKeyCode::S => self.move_back = pressed,
				VirtualKeyCode::Space => self.jump = pressed,
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
