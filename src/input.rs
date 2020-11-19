use glium::Display;
use glium::glutin::event::{KeyboardInput, VirtualKeyCode, ElementState};
use glium::glutin::dpi::PhysicalPosition;

#[derive(Default)]
pub struct InputState {
	pub w: bool,
	pub a: bool, 
	pub s: bool,
	pub d: bool,

	pub mouse_diff: Option<(f32, f32)>
}

impl InputState {
	pub fn update_keyboard_state(&mut self, ev: &KeyboardInput) {
		let is_pressed = ev.state == ElementState::Pressed;
		match ev.virtual_keycode {
			None => return,
			Some(key) => {
				match key {
					VirtualKeyCode::A => self.a = is_pressed,
					VirtualKeyCode::W => self.w = is_pressed,
					VirtualKeyCode::S => self.s = is_pressed,
					VirtualKeyCode::D => self.d = is_pressed,
					_ => ()
				};
			}
		};
	}

	pub fn update_mouse_state(&mut self, new_pos: &PhysicalPosition<f64>, display: &Display) {
		let gl_window = display.gl_window();
		let window = gl_window.window();
		let winsize = window.inner_size();
		let middle = ((winsize.width / 2) as f64, (winsize.height / 2) as f64);

		// if (new_pos.x >= (middle.0 - 1.) && new_pos.x <= (middle.0 + 1.)) && (new_pos.y >= (middle.1 - 1.) && new_pos.y <= (middle.1 + 1.)) {
		// 	return;
		// }
		// if (new_pos.x.abs() > 50.0f64 || new_pos.y.abs() > 50.0f64) {
		// 	return;
		// }

		self.mouse_diff = Some(((new_pos.x - middle.0) as f32, (new_pos.y - middle.1) as f32));
		window.set_cursor_position(PhysicalPosition::new(middle.0, middle.1));
	}
}
