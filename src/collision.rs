use crate::quadoctree::{QuadOctreeNode, CollisionObj, traverse_quadoctree};
use crate::math::{dot_product, cross_product, add_vector};
use crate::draw::Vertex;

const EPSILON: f32 = 0.000001;
const MAX_POLY_COLLIDE: usize = 4;

pub struct CollisionResult {
	pub triangle: Option<[f32; 3]>,
	pub polygons: Vec<[f32; 3]>
}

fn moller_trumbore(triangle: &[[f32; 3]; 3], ray_origin: &[f32; 3], ray_direction: &[f32; 3]) -> Option<[f32; 3]> {
	let edge1 = add_vector(&triangle[1], &triangle[0], -1.);
	let edge2 = add_vector(&triangle[2], &triangle[0], -1.);

	let pvec = cross_product(ray_direction, &edge2);
	let det = dot_product(&edge1, &pvec);

	if det > -EPSILON && det < EPSILON {
		return None;
	}

	let inv_det = 1.0f32 / det;
	let tvec = add_vector(ray_origin, &triangle[0], -1.); 
	
	let u = dot_product(&tvec, &pvec) * inv_det;
	if u < 0. || u > 1. {
		return None;
	}

	let qvec = cross_product(&tvec, &edge1);

	let v = dot_product(ray_direction, &qvec) * inv_det;
	if v < 0. || (u + v) > 1. {
		return None;
	}

	let t = dot_product(&edge2, &qvec) * inv_det;
	if t <= EPSILON {
		return None;
	}
	Some(add_vector(ray_origin, ray_direction, t))
}

fn sat_axis_projection(vertices: &[Vertex], axis: &[f32; 3]) -> (f32, f32) {
	let mut result = (f32::MAX, f32::MIN);
	for vertex in vertices {
		let vertex_project = dot_product(axis, &vertex.position);
		if vertex_project < result.0 {
			result.0 = vertex_project;
		}
		if vertex_project > result.1 {
			result.1 = vertex_project;
		}
	}
	result
}

fn sat_polypoly(a_vertices: &[Vertex], a_center: &[f32; 3], b_vertices: &[Vertex], b_center: &[f32; 3]) -> Option<[f32; 3]> {
	let mut axes: Vec<[f32; 3]> = Vec::with_capacity(a_vertices.len() + b_vertices.len());
	for a_vertex in a_vertices {
		if !axes.contains(&a_vertex.normal) {
			axes.push(a_vertex.normal);
		}
	}
	for b_vertex in b_vertices { 
		if !axes.contains(&b_vertex.normal) {
			axes.push(b_vertex.normal);
		}
	}
	let mut min_range_diff = f32::MAX;
	let mut min_axis = [0., 0., 0.0f32];

	for axis in axes {
		let a_project = sat_axis_projection(a_vertices, &axis);
		let b_project = sat_axis_projection(b_vertices, &axis);

		if b_project.0 > a_project.1 || b_project.1 < a_project.0 {
			return None;
		}
		let range_diff = b_project.1.min(a_project.1) - b_project.0.max(a_project.0);
		
		let direction_validation = dot_product(&add_vector(b_center, a_center, -1.), &axis);
		if range_diff < min_range_diff && direction_validation > 0. {
			min_range_diff = range_diff;
			min_axis = axis;
		}
	}
	Some([min_axis[0] * min_range_diff, min_axis[1] * min_range_diff, min_axis[2] * min_range_diff])
}

pub fn check_player_collision(tree: &QuadOctreeNode, point: &[f32; 3], player_box: &CollisionObj) -> CollisionResult {
	let mut result = CollisionResult {
		triangle: None,
		polygons: Vec::with_capacity(MAX_POLY_COLLIDE)
	};

	traverse_quadoctree(tree, point, &mut |obj: &CollisionObj| -> bool {
		if let CollisionObj::Polygon(p_vertices, p_center) = player_box {
			match obj {
				CollisionObj::Triangle(triangle) => {
					if result.triangle.is_none() {
						let point = [point[0], point[1] + 0.25, point[2]];
						result.triangle = moller_trumbore(&triangle, &point, &[0., -1., 0.]);
					}
				},
				CollisionObj::Polygon(o_vertices, o_center) => {
					if result.polygons.len() < MAX_POLY_COLLIDE {
						let sat_result = sat_polypoly(o_vertices, o_center, p_vertices, p_center);
						if let Some(vector) = sat_result {
							result.polygons.push(vector);
						}
					}
				}
			}
		};

		result.triangle.is_some() && result.polygons.len() >= MAX_POLY_COLLIDE
	});

	result
}
