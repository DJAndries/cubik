use cubik::glium::{self, Display, Frame};
use cubik::fonts::LoadedFont;
use cubik::ui::{ImageBackground, TextButton, UIError, TextInput};
use cubik::input::InputListener;
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use crate::support::constants::APP_ID;

const NORMAL_COLOR: [f32; 4] = [0.94, 0.94, 0.94, 1.];
const HOVER_COLOR: [f32; 4] = [1., 1., 0.2, 1.];
const WHITE: [f32; 4] = [1., 1., 1., 1.];

#[derive(Copy, Clone)]
pub enum MainMenuAction {
	Start,
	Quit
}

pub struct MainMenu {
	pub enabled: bool,
	buttons: Vec<(MainMenuAction, TextButton)>,
	btn_font: LoadedFont,
	bg: ImageBackground,
	result: Option<MainMenuAction>,
	start_dialog: StartDialog
}

impl MainMenu {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			enabled: false,
			buttons: vec![
				(MainMenuAction::Start,
				 	TextButton::new("Start".to_string(), 0.15, (0., -0.3), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR)),
				(MainMenuAction::Quit,
				 	TextButton::new("Quit".to_string(), 0.15, (0., -0.5), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR))
			],
			bg: ImageBackground::new(display, "./textures/mainmenu.jpg", APP_ID, (0., 0.), (3.55, 2.))?,
			btn_font: LoadedFont::load(display, "./fonts/SourceCodePro-Light.otf", APP_ID, 80.)?,
			start_dialog: StartDialog::new(display)?,
			result: None
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program) -> Result<Option<MainMenuAction>, UIError> {
		self.bg.draw(target, ui_program);
		for (_, button) in &mut self.buttons {
			button.draw(target, display, ui_program, &self.btn_font)?;
		}
		if self.start_dialog.enabled { self.result = self.start_dialog.draw(target, display, ui_program, &self.btn_font)?; }
		
		if let Some(result) = self.result {
			self.result = None;
			return Ok(Some(result));
		}
		Ok(None)
	}
}

impl InputListener for MainMenu {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if !self.enabled { return false; }
		self.start_dialog.handle_key_ev(key, pressed)
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		for (_, button) in &mut self.buttons {
			button.handle_mouse_pos_ev(pos, display);
		}
		self.start_dialog.handle_mouse_pos_ev(pos, display);
		true
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		for (name, button) in &mut self.buttons {
			if button.handle_mouse_ev(mouse_button, state) {
				match name {
					MainMenuAction::Start => self.start_dialog.enabled = true,
					_ => self.result = Some(*name)
				};
				return true;
			}
		}
		if self.start_dialog.handle_mouse_ev(mouse_button, state) { return true; }
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		if !self.enabled { return false; }
		self.start_dialog.handle_char_ev(ch)
	}
}

struct StartDialog {
	bg: ImageBackground,
	ip_input: TextInput,
	start_btn: TextButton,
	enabled: bool,
	result: Option<MainMenuAction>
}

impl StartDialog {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			bg: ImageBackground::new(display, "./textures/dialog.png", APP_ID, (0., -0.17), (1.0, 0.7))?,
			start_btn: TextButton::new("Start".to_string(), 0.08, (0.2, -0.2), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR),
			ip_input: TextInput::new((-0.45, -0.15), (0.85, 0.12), WHITE),
			enabled: false,
			result: None
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<Option<MainMenuAction>, UIError> {
		self.bg.draw(target, ui_program);
		self.ip_input.draw(target, display, ui_program, font)?;
		self.start_btn.draw(target, display, ui_program, font)?;
		if let Some(result) = self.result {
			self.result = None;
			return Ok(Some(result));
		}
		Ok(None)
	}
}

impl InputListener for StartDialog {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_key_ev(key, pressed)
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		if self.ip_input.handle_mouse_pos_ev(pos, display) { return true; }
		self.start_btn.handle_mouse_pos_ev(pos, display)
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		if self.ip_input.handle_mouse_ev(mouse_button, state) { return true; }
		if self.start_btn.handle_mouse_ev(mouse_button, state) {
			self.result = Some(MainMenuAction::Start);
			return true;
		}
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_char_ev(ch)
	}
}
