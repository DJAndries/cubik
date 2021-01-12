use glium::{Frame, Program};
use std::time::Instant;
use std::collections::{BTreeMap};
use crate::player::PlayerControlMessage;
use crate::animation::ObjAnimation;
use crate::draw::{basic_render, ObjDef, ObjDrawInfo, EnvDrawInfo};
use crate::interpolation::{InterpolationHelper, Interpolate};

pub struct PeerPlayer {
	pub obj_draw_info: ObjDrawInfo,

	pub is_colliding: bool,
	pub is_moving: bool,

	animation_time: Instant,
	interpolation: InterpolationHelper<PosYawValue>
}

#[derive(Copy, Clone)]
pub struct PosYawValue {
	pos: [f32; 3],
	yaw: f32
}

impl Interpolate for PosYawValue {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		Self {
			pos: <[f32; 3]>::linear_interpolate(&a.pos, &b.pos, progress),
			yaw: f32::linear_interpolate(&a.yaw, &b.yaw, progress)
		}
	}
}

impl PeerPlayer {

	pub fn new() -> Self {
		Self {
			obj_draw_info: Default::default(),
			is_colliding: false,
			is_moving: false,
			animation_time: Instant::now(),
			interpolation: InterpolationHelper::new()
		}
	}

	pub fn update(&mut self, incoming_msg: Option<PlayerControlMessage>, time_delta: f32) {
		if let Some(incoming_msg) = incoming_msg {
			let was_walking = self.is_colliding && self.is_moving;
			if let PlayerControlMessage::Server { position, yaw, is_colliding, is_moving } = incoming_msg {
				self.interpolation.post_update(PosYawValue {
					pos: position,
					yaw: yaw
				});
				
				self.is_colliding = is_colliding;
				self.is_moving = is_moving;
				if !was_walking && is_colliding && is_moving {
					self.animation_time = Instant::now();
				}
			}
			return;
		}

		if let Some(pos_yaw) = self.interpolation.value(time_delta) {
			self.obj_draw_info.position = pos_yaw.pos;
			self.obj_draw_info.rotation[1] = pos_yaw.yaw;
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
