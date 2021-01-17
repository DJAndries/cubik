pub trait Interpolate {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self;
}

impl Interpolate for f32 {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		*a + ((*b - *a) * progress)
	}
}

impl Interpolate for (f32, f32) {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		(
			a.0 + ((b.0 - a.0) * progress),
			a.1 + ((b.1 - a.1) * progress)
		)
	}
}

impl Interpolate for [f32; 3] {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		[
			a[0] + ((b[0] - a[0]) * progress),
			a[1] + ((b[1] - a[1]) * progress),
			a[2] + ((b[2] - a[2]) * progress),
		]
	}
}

pub struct InterpolationHelper<T: Interpolate + Copy> {
	pub updates: Vec<T>,
	last_update_duration: f32,
	time_count: f32
}

impl<T: Interpolate + Copy> InterpolationHelper<T> {
	pub fn new() -> Self {
		Self {
			updates: Vec::new(),
			time_count: 0.,
			last_update_duration: 0.
		}
	}

	pub fn post_update(&mut self, update: T) {
		self.updates.push(update);
		if self.updates.len() > 2 {
			self.updates.remove(0);
		}
		self.last_update_duration = self.time_count.max(0.1);
		self.time_count = 0.
	}

	pub fn value(&mut self, time_delta: f32) -> Option<T> {
		self.time_count += time_delta;
		let len = self.updates.len();
		match len {
			0 => None,
			1 => Some(*self.updates.first().unwrap()),
			_ => {
				let first = self.updates.first().unwrap();
				let last = self.updates.last().unwrap();
				Some(T::linear_interpolate(&first, &last, self.time_count / self.last_update_duration))
			}
		}
	}
}
