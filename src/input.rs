use glium::Display;
use glium::glutin::event::{VirtualKeyCode, ElementState, WindowEvent, MouseButton};
use glium::glutin::dpi::PhysicalPosition;

pub trait InputListener {
	fn handle_char_ev(&mut self, ch: char) -> bool;
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool;
	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool;
	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool;
}

pub fn process_input_event(ev: WindowEvent, listeners: Vec<&mut dyn InputListener>, display: &Display) -> bool {
	for listener in listeners {
		if match ev {
			WindowEvent::ReceivedCharacter(ch) => {
				listener.handle_char_ev(ch)
			},
			WindowEvent::KeyboardInput { input, .. } => {
				listener.handle_key_ev(input.virtual_keycode, input.state == ElementState::Pressed)
			},
			WindowEvent::CursorMoved { position, .. } => {
				let dim = display.gl_window().window().inner_size();
				let ev_pos = ((position.x as f32) / (dim.width as f32) * 2. - 1.,
					-((position.y as f32) / (dim.height as f32) * 2. - 1.));
				listener.handle_mouse_pos_ev(ev_pos, display)
			},
			WindowEvent::MouseInput { state, button, .. } => {
				listener.handle_mouse_ev(button, state)
			},
			_ => false
		} {
			return true;
		}
	}
	false
}

pub fn center_cursor(display: &Display, cursor_visible: bool) {
	let gl_window = display.gl_window();
	let window = gl_window.window();
	let winsize = window.inner_size();
	let middle = ((winsize.width / 2) as f32, (winsize.height / 2) as f32);
	window.set_cursor_position(PhysicalPosition::new(middle.0, middle.1)).unwrap();
	window.set_cursor_visible(cursor_visible);
}
