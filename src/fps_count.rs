use std::time::Instant;

pub struct DebugFPSCounter {
	pub enabled: bool,
	pub frames_in_period: usize,
	pub period_start: Instant
}

impl DebugFPSCounter {
	pub fn new(enabled: bool) -> Self {
		Self {
			enabled: enabled,
			frames_in_period: 0,
			period_start: Instant::now()
		}
	}

	pub fn update(&mut self) {
		if !self.enabled { return; }
		self.frames_in_period += 1;
		if self.period_start.elapsed().as_secs_f32() >= 1. {
			println!("{} fps", self.frames_in_period);
			self.frames_in_period = 0;
			self.period_start = Instant::now();
		}
	}
}
