use crate::fonts::{LoadedFont, FontText, TextAlign, FontError};
use crate::input::InputListener;
use glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use glium::{Display, Frame};
use std::collections::HashSet;
use derive_more::{From, Error};

const NORMAL_COLOR: [f32; 4] = [0.94, 0.94, 0.94, 1.];
const HOVER_COLOR: [f32; 4] = [1., 1., 0.6, 1.];

#[derive(Debug, derive_more::Display, From, Error)]
pub enum UIError {
	FontError(FontError)
}

pub struct TextButton {
	text: FontText,
	pos: (f32, f32),
	padding: (f32, f32),
	normal_color: [f32; 4],
	hover_color: [f32; 4],
	is_hovering: bool
}

impl TextButton {
	pub fn new(text: String, pos: (f32, f32), padding: (f32, f32), normal_color: [f32; 4], hover_color: [f32; 4]) -> Self {
		Self {
			text: FontText::new(text, pos, TextAlign::Left),
			pos: pos,
			padding: (0., 0.),
			normal_color: normal_color,
			hover_color: hover_color,
			is_hovering: false
		}
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, font_program: &glium::Program, font: &LoadedFont) -> Result<(), UIError> {
		Ok(self.text.draw(target, display, font_program, font, if self.is_hovering { self.hover_color } else { self.normal_color })?)
	}
}

impl InputListener for TextButton {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), display: &Display) -> bool {
		self.is_hovering = mouse_pos.0 >= (self.pos.0 - self.padding.0)
			&& mouse_pos.0 < (self.pos.0 + self.text.current_size.0 + self.padding.0)
			&& mouse_pos.1 >= (self.pos.1 - self.padding.1)
			&& mouse_pos.1 < (self.pos.1 + self.text.current_size.1 + self.padding.1);
		false
	}

	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool {
		if self.is_hovering && state == ElementState::Released && button == MouseButton::Left {
			return true;
		}
		false
	}
}

#[derive(Copy, Clone)]
pub enum MainMenuAction {
	Start,
	Quit
}

pub struct MainMenu {
	pub enabled: bool,
	buttons: Vec<(MainMenuAction, TextButton)>,
	btn_font: LoadedFont,
	result: Option<MainMenuAction>
}

impl MainMenu {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			enabled: false,
			buttons: vec![
				(MainMenuAction::Start, TextButton::new("Start".to_string(), (-0.9, 0.8), (0.005, 0.005), NORMAL_COLOR, HOVER_COLOR))
			],
			btn_font: LoadedFont::load(display, "./fonts/SourceCodePro-Light.otf", 32.)?,
			result: None
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, font_program: &glium::Program) -> Result<Option<MainMenuAction>, UIError> {
		for (_, button) in &mut self.buttons {
			button.draw(target, display, font_program, &self.btn_font)?;
		}
		
		if let Some(result) = self.result {
			self.result = None;
			return Ok(Some(result));
		}
		Ok(None)
	}
}

impl InputListener for MainMenu {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		for (_, button) in &mut self.buttons {
			button.handle_mouse_pos_ev(pos, display);
		}
		true
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		for (name, button) in &mut self.buttons {
			if button.handle_mouse_ev(mouse_button, state) {
				self.result = Some(*name);
				return true;
			}
		}
		false
	}
}
