use glium::{Frame, Surface};
use crate::input::InputState;
use crate::math::{normalize_vector, cross_product, add_vector};
use crate::draw::Vertex;
use crate::cube::generate_cube_collideobj;
use crate::quadoctree::CollisionObj;

pub struct Camera {
	pub position: [f32; 3],
	pub direction: [f32; 3],

	pub camera_pitch_yaw: (f32, f32),
	pub player_cube: CollisionObj
}

const UP: [f32; 3] = [0.0, 1.0, 0.0];
const MOVE_RATE: f32 = 0.64;
const MOUSE_SENSITIVITY: f32 = 0.05;
const PLAYER_CUBE_DIM: [f32; 3] = [0.15, 0.2, 0.15];

impl Camera {
	pub fn new(position: [f32; 3]) -> Self {
		Camera {
			position: position,
			direction: [0.0, 0.0, 0.0],
			camera_pitch_yaw: (0.0, 0.0),
			player_cube: generate_cube_collideobj(&position, &PLAYER_CUBE_DIM)
		}
	}

	pub fn view_matrix(&self) -> [[f32; 4]; 4] {
		let s = normalize_vector(&cross_product(&UP, &self.direction));

		let u = cross_product(&self.direction, &s);

		let p = [-self.position[0] * s[0] - self.position[1] * s[1] - self.position[2] * s[2],
				-self.position[0] * u[0] - self.position[1] * u[1] - self.position[2] * u[2],
				-self.position[0] * self.direction[0] - self.position[1] * self.direction[1] - self.position[2] * self.direction[2]];

		[
			[s[0], u[0], self.direction[0], 0.0],
			[s[1], u[1], self.direction[1], 0.0],
			[s[2], u[2], self.direction[2], 0.0],
			[p[0], p[1], p[2], 1.0]
		]
	}

	pub fn update(&mut self, time_delta: f32, input_state: &mut InputState) {
		let move_len = MOVE_RATE * time_delta;
		let mut movement_vec = [0., 0., 0.0f32];
		let right = normalize_vector(&cross_product(&UP, &self.direction));
		let up = cross_product(&self.direction, &right);
		let direction_perp = cross_product(&self.direction, &up);
		if input_state.w { movement_vec = add_vector(&movement_vec, &self.direction, 1.0); }
		if input_state.s { movement_vec = add_vector(&movement_vec, &self.direction, -1.0); }
		if input_state.a { movement_vec = add_vector(&movement_vec, &direction_perp, 1.0); }
		if input_state.d { movement_vec = add_vector(&movement_vec, &direction_perp, -1.0); }
		self.position = add_vector(&self.position, &movement_vec, move_len);
		self.player_cube = generate_cube_collideobj(&self.position, &PLAYER_CUBE_DIM);

		if let Some(mouse_diff) = input_state.mouse_diff {
			self.camera_pitch_yaw.1 -= mouse_diff.0 * MOUSE_SENSITIVITY;
			self.camera_pitch_yaw.0 -= (mouse_diff.1 * MOUSE_SENSITIVITY).min(1.57).max(-1.57);
			self.camera_pitch_yaw.0 = self.camera_pitch_yaw.0.min(1.57).max(-1.57);
			self.direction = normalize_vector(&[
				self.camera_pitch_yaw.1.cos() * self.camera_pitch_yaw.0.cos(),
				self.camera_pitch_yaw.0.sin(),
				self.camera_pitch_yaw.1.sin() * self.camera_pitch_yaw.0.cos()
			]);
			// println!("{:?}", self.camera_pitch_yaw);
			input_state.mouse_diff = None;
			// println!("{:?}", self.direction);
		}
	}
}

pub fn perspective_matrix(target: &mut Frame) -> [[f32; 4]; 4] {
	let (width, height) = target.get_dimensions();
	let aspect_ratio = height as f32 / width as f32;

	let fov: f32 = 3.141592 / 3.0;
	let zfar = 1024.0;
	let znear = 0.1;

	let f = 1.0 / (fov / 2.0).tan();

	[
		[f * aspect_ratio, 0.0, 0.0, 0.0],
		[0.0, f, 0.0, 0.0],
		[0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
		[0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]
	]
}
