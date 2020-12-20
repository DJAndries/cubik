use glium::{Frame, Program, Texture2d};
use std::time::Instant;
use std::collections::{HashMap, BTreeMap};
use crate::player::PlayerControlMessage;
use crate::animation::ObjAnimation;
use crate::draw::{basic_render, ObjDef, ObjDrawInfo, EnvDrawInfo};

pub struct PeerPlayer {
	pub obj_draw_info: ObjDrawInfo,

	pub is_colliding: bool,
	pub is_moving: bool,

	animation_time: Instant
}

impl PeerPlayer {

	pub fn new() -> Self {
		Self {
			obj_draw_info: Default::default(),
			is_colliding: false,
			is_moving: false,
			animation_time: Instant::now()
		}
	}

	pub fn update(&mut self, incoming_msg: PlayerControlMessage) {
		let was_walking = self.is_colliding && self.is_moving;
		if let PlayerControlMessage::Server { position, pitch_yaw, is_colliding, is_moving } = incoming_msg {
			self.obj_draw_info.position = position;
			self.obj_draw_info.rotation = [pitch_yaw.0, pitch_yaw.1, 0.];
			self.is_colliding = is_colliding;
			self.is_moving = is_moving;
			if !was_walking && is_colliding && is_moving {
				self.animation_time = Instant::now();
			}
			self.obj_draw_info.generate_matrix();
		}
	}

	pub fn draw(&mut self, target: &mut Frame, env_info: &EnvDrawInfo, program: &Program,
		moving_animation: &ObjAnimation, stand_model: &BTreeMap<String, ObjDef>, jump_model: &BTreeMap<String, ObjDef>) {
		
		let model = if self.is_moving {
			if self.is_colliding {
				moving_animation.get_keyframe(self.animation_time.elapsed().as_secs_f32())
			} else {
				jump_model
			}
		} else {
			stand_model
		};
			
		for def in model.values() {
			basic_render(target, env_info, &self.obj_draw_info, def, program, None);
		}
	}

}
