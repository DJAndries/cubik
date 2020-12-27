use glium::Display;
use crate::draw::{Vertex, ObjDef, load_data_to_gpu};
use crate::quadoctree::CollisionObj;

pub fn generate_cube_vertices(pre_pos: &[f32; 3], post_pos: &[f32; 3], dim: &[f32; 3], yaw: f32) -> [Vertex; 24] {
	let gen_pos = |proto: [f32; 3]| {
		let before_yaw = [proto[0] * dim[0] + pre_pos[0], proto[1] * dim[1] + pre_pos[1], proto[2] * dim[2] + pre_pos[2]];
		let after_yaw = (
			(before_yaw[0] * yaw.cos()) + (before_yaw[2] * yaw.sin()),
			(-before_yaw[0] * yaw.sin()) + (before_yaw[2] * yaw.cos())
		);
		[after_yaw.0 + post_pos[0], before_yaw[1] + post_pos[1], after_yaw.1 + post_pos[2]]
	};
	[
		// back face
		Vertex { position: gen_pos([-1., -1., -1.]), normal: [0., 0., -1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., -1., -1.]), normal: [0., 0., -1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., 1., -1.]), normal: [0., 0., -1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., -1.]), normal: [0., 0., -1.], texcoords: [0., 0.] },
		// front face
		Vertex { position: gen_pos([-1., -1., 1.]), normal: [0., 0., 1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., -1., 1.]), normal: [0., 0., 1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., 1., 1.]), normal: [0., 0., 1.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., 1.]), normal: [0., 0., 1.], texcoords: [0., 0.] },
		// left face
		Vertex { position: gen_pos([-1., -1., 1.]), normal: [-1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., -1., -1.]), normal: [-1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., 1., -1.]), normal: [-1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., 1., 1.]), normal: [-1., 0., 0.], texcoords: [0., 0.] },
		// right face
		Vertex { position: gen_pos([1., -1., 1.]), normal: [1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., -1., -1.]), normal: [1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., -1.]), normal: [1., 0., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., 1.]), normal: [1., 0., 0.], texcoords: [0., 0.] },
		// top face
		Vertex { position: gen_pos([-1., 1., 1.]), normal: [0., 1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., -1.]), normal: [0., 1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., 1., -1.]), normal: [0., 1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., 1., 1.]), normal: [0., 1., 0.], texcoords: [0., 0.] },
		// bottom face
		Vertex { position: gen_pos([-1., -1., 1.]), normal: [0., -1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., -1., -1.]), normal: [0., -1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([-1., -1., -1.]), normal: [0., -1., 0.], texcoords: [0., 0.] },
		Vertex { position: gen_pos([1., -1., 1.]), normal: [0., -1., 0.], texcoords: [0., 0.] },
	]
}

const INDICES: [[u32; 3]; 12] = [
	// back face
	[0, 1, 2],
	[1, 3, 2],
	// front face
	[6, 5, 4],
	[6, 7, 5],
	// left face
	[8, 9, 10],
	[11, 8, 10],
	// right face
	[14, 13, 12],
	[15, 14, 12],
	// top face
	[18, 17, 16],
	[19, 16, 17],
	// bottom face
	[20, 21, 22],
	[21, 20, 23]
];

pub fn load_cube(display: &Display, dim: &[f32; 3], cull_reverse: bool) -> ObjDef {
	let mut indices: Vec<u32> = Vec::new();
	if cull_reverse {
		for tri in &INDICES {
			let mut tri_clone = tri.clone();
			tri_clone.reverse();
			indices.extend(&tri_clone);
		}
	} else {
		INDICES.iter().for_each(|i| indices.extend(i));
	};
	load_data_to_gpu(display, &generate_cube_vertices(&[0., 0., 0.], &[0., 0., 0.], dim, 0.), &indices)
}

pub fn generate_cube_collideobj(pre_pos: &[f32; 3], post_pos: &[f32; 3], dim: &[f32; 3], yaw: f32) -> CollisionObj {
	CollisionObj::Polygon(generate_cube_vertices(pre_pos, post_pos, dim, yaw).to_vec(), *post_pos)
}
