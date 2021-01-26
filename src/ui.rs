use crate::fonts::{LoadedFont, FontText, TextAlign, FontError};
use crate::input::InputListener;
use glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use glium::{Display, Frame, texture::SrgbTexture2d};
use crate::draw::{Vertex, ObjDef, load_data_to_gpu, UIDrawInfo, ui_draw};
use crate::textures::{load_srgb_texture, TextureLoadError};
use derive_more::{From, Error};
use crate::assets::find_asset;

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
	fn handle_char_ev(&mut self, _ch: char) -> bool {
		false
	}

	fn handle_key_ev(&mut self, _key: Option<VirtualKeyCode>, _pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), _display: &Display) -> bool {
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

	pub fn reset(&mut self) {
		self.text = String::new();
		self.gen_display_text();
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

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), _display: &Display) -> bool {
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
	pub fn new(display: &Display, image_filename: &str, app_id: &str, pos: (f32, f32), size: (f32, f32)) -> Result<Self, UIError> {
		let vertices = [
			Vertex { position: [-0.5, -0.5, 0.], normal: [0., 0., -1.], texcoords: [0., 0.] },
			Vertex { position: [0.5, -0.5, 0.], normal: [0., 0., -1.], texcoords: [1., 0.] },
			Vertex { position: [0.5, 0.5, 0.], normal: [0., 0., -1.], texcoords: [1., 1.] },
			Vertex { position: [-0.5, 0.5, 0.], normal: [0., 0., -1.], texcoords: [0., 1.] }
		];
		let indices = [0, 1, 2, 0, 2, 3];
		Ok(Self {
			texture: load_srgb_texture(display, find_asset(image_filename, app_id).as_path(), true)?,
			obj_def: load_data_to_gpu(display, &vertices, &indices),
			ui_draw_info: UIDrawInfo::new(pos, size)
		})
	}

	pub fn draw(&mut self, target: &mut Frame, program: &glium::Program) {
		self.ui_draw_info.generate_matrix(target);
		ui_draw(target, &self.obj_def, &self.ui_draw_info, program, &self.texture);
	}
}

pub struct ImageButton {
	background: ImageBackground,
	normal_color: [f32; 4],
	hover_color: [f32; 4],
	half_size: (f32, f32),
	is_hovering: bool
}

impl ImageButton {
	pub fn new(background: ImageBackground, half_size: (f32, f32), normal_color: [f32; 4], hover_color: [f32; 4]) -> Self {
		Self {
			background: background,
			normal_color: normal_color,
			hover_color: hover_color,
			half_size: half_size,
			is_hovering: false
		}
	}

	pub fn draw(&mut self, target: &mut Frame, ui_program: &glium::Program) {
		self.background.draw(target, ui_program);
	}
}

impl InputListener for ImageButton {
	fn handle_char_ev(&mut self, _ch: char) -> bool {
		false
	}

	fn handle_key_ev(&mut self, _key: Option<VirtualKeyCode>, _pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, mouse_pos: (f32, f32), _display: &Display) -> bool {
		let pos = self.background.ui_draw_info.position;
		self.is_hovering = mouse_pos.0 >= (pos.0 - self.half_size.0)
			&& mouse_pos.0 < (pos.0 + self.half_size.0)
			&& mouse_pos.1 >= (pos.1 - self.half_size.1)
			&& mouse_pos.1 < (pos.1 + self.half_size.1);
		self.background.ui_draw_info.color = if self.is_hovering { self.hover_color } else { self.normal_color };
		false
	}

	fn handle_mouse_ev(&mut self, button: MouseButton, state: ElementState) -> bool {
		if self.is_hovering && state == ElementState::Released && button == MouseButton::Left {
			return true;
		}
		false
	}
}
