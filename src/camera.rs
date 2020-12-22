use glium::{Frame, Surface};
use crate::math::{normalize_vector, cross_product};

#[derive(Copy, Clone)]
pub struct Camera {
	pub position: [f32; 3],
	pub height: f32,
	pub direction: [f32; 3],

	pub pitch_yaw: (f32, f32)
}

pub const UP: [f32; 3] = [0.0, 1.0, 0.0];

impl Camera {
	pub fn new(position: [f32; 3], height: f32) -> Self {
		Camera {
			position: position,
			height: height,
			direction: [0.0, 0.0, 0.0],
			pitch_yaw: (0.0, 0.0)
		}
	}

	pub fn update_direction(&mut self) {
		self.direction = normalize_vector(&[
			self.pitch_yaw.1.cos() * self.pitch_yaw.0.cos(),
			self.pitch_yaw.0.sin(),
			self.pitch_yaw.1.sin() * self.pitch_yaw.0.cos()
		]);
	}

	pub fn view_matrix(&self) -> [[f32; 4]; 4] {
		let s = normalize_vector(&cross_product(&UP, &self.direction));

		let u = cross_product(&self.direction, &s);

		let position = [self.position[0], self.position[1] + self.height, self.position[2]];

		let p = [-position[0] * s[0] - position[1] * s[1] - position[2] * s[2],
				-position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
				-position[0] * self.direction[0] - position[1] * self.direction[1] - position[2] * self.direction[2]];

		[
			[s[0], u[0], self.direction[0], 0.0],
			[s[1], u[1], self.direction[1], 0.0],
			[s[2], u[2], self.direction[2], 0.0],
			[p[0], p[1], p[2], 1.0]
		]
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
