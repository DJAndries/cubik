use std::time::Instant;

pub trait Interpolate {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self;
}

impl Interpolate for f32 {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		*a + ((*b - *a) * progress)
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
	pub updates: Vec<(T, Instant)>
}

impl<T: Interpolate + Copy> InterpolationHelper<T> {
	pub fn new() -> Self {
		Self {
			updates: Vec::new()
		}
	}

	pub fn post_update(&mut self, update: T) {
		self.updates.push((update, Instant::now()));
		if self.updates.len() > 2 {
			self.updates.remove(0);
		}
	}

	pub fn value(&self) -> Option<T> {
		match self.updates.len() {
			0 => None,
			1 => Some(self.updates.first().as_ref().unwrap().0),
			_ => {
				let first = self.updates.first().unwrap();
				let last = self.updates.last().unwrap();

				let total_duration = last.1.duration_since(first.1).as_secs_f32();
				let elapsed = Instant::now().duration_since(last.1).as_secs_f32();

				Some(T::linear_interpolate(&first.0, &last.0, elapsed / total_duration))
			}
		}
	}
}
