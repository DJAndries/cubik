use crate::quadoctree::{QuadOctreeNode, CollisionObj, traverse_quadoctree};
use crate::math::{dot_product, cross_product, add_vector};

const EPSILON: f32 = 0.000001;

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

pub fn check_triangle_collision(tree: &QuadOctreeNode, point: &[f32; 3]) -> Option<[f32; 3]> {
	let mut result: Option<[f32; 3]> = None;
	
	let mut count = 0;

	traverse_quadoctree(tree, point, &mut |obj: &CollisionObj| -> bool {
		result = moller_trumbore(obj.triangle.as_ref().unwrap(), point, &[0., -1., 0.]);
		count += 1;
		result.is_some()
	});

	result
}
