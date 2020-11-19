use glium::{Display, Frame, DrawParameters, VertexBuffer, IndexBuffer};
use crate::draw::{Vertex, ObjDef, load_data_to_gpu};

const POSITIONS: [Vertex; 24] = [
	// back face
	Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], texcoords: [0., 0.] },
	// front face
	Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], texcoords: [0., 0.] },
	// left face
	Vertex { position: [-0.5, -0.5, 0.5], normal: [-1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, 0.5, -0.5], normal: [-1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, 0.5, 0.5], normal: [-1.0, 0.0, 0.0], texcoords: [0., 0.] },
	// right face
	Vertex { position: [0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, -0.5], normal: [1.0, 0.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, 0.5], normal: [1.0, 0.0, 0.0], texcoords: [0., 0.] },
	// top face
	Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], texcoords: [0., 0.] },
	// bottom face
	Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], texcoords: [0., 0.] },
	Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], texcoords: [0., 0.] },
];

const INDICES: [u32; 36] = [
	// back face
	0, 1, 2,
	1, 3, 2,
	// front face
	6, 5, 4,
	6, 7, 5,
	// left face
	8, 9, 10,
	11, 8, 10,
	// right face
	14, 13, 12,
	15, 14, 12,
	// top face
	18, 17, 16,
	19, 16, 17,
	// bottom face
	20, 21, 22,
	21, 20, 23
];

pub fn load_cube(display: &Display) -> ObjDef {
	load_data_to_gpu(display, &POSITIONS, &INDICES)
}
