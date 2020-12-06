use std::path::Path;
use crate::fonts::{LoadedFont, FontText, TextAlign, FontError};
use crate::input::InputListener;
use glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use glium::{DrawParameters, Display, Surface, Frame, texture::SrgbTexture2d};
use crate::draw::{Vertex, ObjDef, load_data_to_gpu, UIDrawInfo, ui_draw};
use crate::textures::{load_srgb_texture, TextureLoadError};
use std::collections::HashSet;
use derive_more::{From, Error};

const NORMAL_COLOR: [f32; 4] = [0.94, 0.94, 0.94, 1.];
const HOVER_COLOR: [f32; 4] = [1., 1., 0.2, 1.];
const WHITE: [f32; 4] = [1., 1., 1., 1.];

#[derive(Debug, derive_more::Display, From, Error)]
pub enum UIError {
	FontError(FontError),
	TextureError(TextureLoadError)
}

pub struct TextButton {
	text: FontText,
	pos: (f32, f32),
	half_size: (f32, f32),
	normal_color: [f32; 4],
	hover_color: [f32; 4],
	is_hovering: bool
}

impl TextButton {
	pub fn new(text: String, size: f32, pos: (f32, f32), half_size: (f32, f32), normal_color: [f32; 4], hover_color: [f32; 4]) -> Self {
		Self {
			text: FontText::new(text, size, pos, TextAlign::Center),
			pos: pos,
			half_size: half_size,
			normal_color: normal_color,
			hover_color: hover_color,
			is_hovering: false
		}
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<(), UIError> {
		Ok(self.text.draw(target, display, ui_program, font)?)
	}
}

impl InputListener for TextButton {
	fn handle_char_ev(&mut self, ch: char) -> bool {
		false
	}

	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), display: &Display) -> bool {
		self.is_hovering = mouse_pos.0 >= (self.pos.0 - self.half_size.0)
			&& mouse_pos.0 < (self.pos.0 + self.half_size.0)
			&& mouse_pos.1 >= (self.pos.1 - self.half_size.1)
			&& mouse_pos.1 < (self.pos.1 + self.half_size.1);
		self.text.ui_draw_info.color = if self.is_hovering { self.hover_color } else { self.normal_color };
		false
	}

	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool {
		if self.is_hovering && state == ElementState::Released && button == MouseButton::Left {
			return true;
		}
		false
	}
}

pub struct TextInput {
	pos: (f32, f32),
	size: (f32, f32),
	color: [f32; 4],
	pub text: String,
	display_text: FontText,
	is_hovering: bool,
	pub active: bool,
	text_x_offset: f32
}

impl TextInput {
	pub fn new(pos: (f32, f32), size: (f32, f32), color: [f32; 4]) -> Self {
		let mut display_text = FontText::new("".to_string(), size.1, pos, TextAlign::Left);
		display_text.ui_draw_info.color = color;
		display_text.ui_draw_info.left_clip = pos.0;
		Self {
			pos: pos,
			size: size,
			color: color,
			text: String::new(),
			display_text: display_text,
			is_hovering: false,
			active: false,
			text_x_offset: 0.
		}
	}

	fn gen_display_text(&mut self) {
		self.display_text = FontText::new(self.text.clone(),
			self.size.1, (self.pos.0 - self.text_x_offset, self.pos.1 + 0.03), TextAlign::Left);
		self.display_text.ui_draw_info.color = self.color;
		self.display_text.ui_draw_info.left_clip = self.pos.0;
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<(), UIError> {
		let text_width = self.display_text.measure_width(font)? * self.size.1;
		if text_width > (self.size.0 + self.text_x_offset) {
			self.text_x_offset = text_width - self.size.0;
			self.gen_display_text();
		}
		Ok(self.display_text.draw(target, display, ui_program, font)?)
	}
}

impl InputListener for TextInput {
	fn handle_char_ev(&mut self, ch: char) -> bool {
		if self.active && ch.is_ascii() && !ch.is_ascii_control() {
			self.text.push(ch);
			self.text_x_offset = 0.;
			self.gen_display_text();
			return true;
		}
		false
	}

	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if let Some(key) = key {
			if self.active && !pressed && key == VirtualKeyCode::Back && self.text.len() > 0 {
				self.text.remove(self.text.len() - 1);
				self.text_x_offset = 0.;
				self.gen_display_text();
				return true;
			}
		}
		false
	}

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), display: &Display) -> bool {
		self.is_hovering = mouse_pos.0 >= self.pos.0
			&& mouse_pos.0 < (self.pos.0 + self.size.0)
			&& mouse_pos.1 >= self.pos.1
			&& mouse_pos.1 < (self.pos.1 + self.size.1);
		false
	}

	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool {
		if state == ElementState::Released && button == MouseButton::Left {
			self.active = self.is_hovering;
			return self.active;
		}
		false
	}
}

pub struct ImageBackground {
	texture: SrgbTexture2d,
	obj_def: ObjDef,
	ui_draw_info: UIDrawInfo
}

impl ImageBackground {
	pub fn new(display: &Display, image_filename: &str, pos: (f32, f32), size: (f32, f32)) -> Result<Self, UIError> {
		let vertices = [
			Vertex { position: [-0.5, -0.5, 0.], normal: [0., 0., -1.], texcoords: [0., 0.] },
			Vertex { position: [0.5, -0.5, 0.], normal: [0., 0., -1.], texcoords: [1., 0.] },
			Vertex { position: [0.5, 0.5, 0.], normal: [0., 0., -1.], texcoords: [1., 1.] },
			Vertex { position: [-0.5, 0.5, 0.], normal: [0., 0., -1.], texcoords: [0., 1.] }
		];
		let indices = [0, 1, 2, 0, 2, 3];
		Ok(Self {
			texture: load_srgb_texture(display, Path::new(image_filename), true)?,
			obj_def: load_data_to_gpu(display, &vertices, &indices),
			ui_draw_info: UIDrawInfo::new(pos, size)
		})
	}

	pub fn draw(&mut self, target: &mut Frame, program: &glium::Program) {
		self.ui_draw_info.generate_matrix(target);
		ui_draw(target, &self.obj_def, &self.ui_draw_info, program, &self.texture);
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
			bg: ImageBackground::new(display, "./textures/mainmenu.jpg", (0., 0.), (3.55, 2.))?,
			btn_font: LoadedFont::load(display, "./fonts/SourceCodePro-Light.otf", 80.)?,
			start_dialog: StartDialog::new(display)?,
			result: None
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program) -> Result<Option<MainMenuAction>, UIError> {
		self.bg.draw(target, ui_program);
		for (_, button) in &mut self.buttons {
			button.draw(target, display, ui_program, &self.btn_font)?;
		}
		if self.start_dialog.enabled { self.start_dialog.draw(target, display, ui_program, &self.btn_font); }
		
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
		self.start_dialog.handle_char_ev(ch)
	}
}

struct StartDialog {
	bg: ImageBackground,
	ip_input: TextInput,
	enabled: bool
}

impl StartDialog {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			bg: ImageBackground::new(display, "./textures/dialog.png", (0., -0.17), (1.0, 0.7))?,
			ip_input: TextInput::new((-0.45, -0.15), (0.85, 0.12), WHITE),
			enabled: false
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<(), UIError> {
		self.bg.draw(target, ui_program);
		self.ip_input.draw(target, display, ui_program, font)?;
		Ok(())
	}
}

impl InputListener for StartDialog {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_key_ev(key, pressed)
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_mouse_pos_ev(pos, display)
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_mouse_ev(mouse_button, state)
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		if !self.enabled { return false; }
		self.ip_input.handle_char_ev(ch)
	}
}
