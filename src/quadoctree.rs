use derive_more::{Display, Error};
use crate::draw::Vertex;

const BUCKET_CAPACITY: usize = 384;

#[derive(Debug, Display, Error)]
pub enum QuadOctreeError {
	BucketFull
}

pub struct BoundingBox {
	pub start_pos: [f32; 3],
	pub end_pos: [f32; 3]
}

#[derive(Clone)]
pub struct CollisionObj {
	// bbox: Option<BoundingBox>,
	pub triangle: Option<[[f32; 3]; 3]>
}

pub struct QuadOctreeNode {
	child_nodes: Option<Vec<QuadOctreeNode>>,
	items: Vec<CollisionObj>,
	bbox: BoundingBox,

	is_octree: bool
}

impl QuadOctreeNode {
	pub fn new(bbox: BoundingBox, is_octree: bool) -> QuadOctreeNode {
		QuadOctreeNode {
			child_nodes: None,
			items: Vec::with_capacity(BUCKET_CAPACITY),
			bbox: bbox,
			is_octree: is_octree
		}
	}

	pub fn new_tree(is_octree: bool) -> QuadOctreeNode {
		QuadOctreeNode {
			child_nodes: None,
			items: Vec::with_capacity(BUCKET_CAPACITY),
			bbox: BoundingBox {
				start_pos: [0., 0., 0.],
				end_pos: [0., 0., 0.]
			},
			is_octree: is_octree
		}
	}
}

fn create_sub_nodes(node: &mut QuadOctreeNode) {
	let mid_pos = [
		node.bbox.start_pos[0] + ((node.bbox.end_pos[0] - node.bbox.start_pos[0]) / 2.),
		node.bbox.start_pos[1] + ((node.bbox.end_pos[1] - node.bbox.start_pos[1]) / 2.),
		node.bbox.start_pos[2] + ((node.bbox.end_pos[2] - node.bbox.start_pos[2]) / 2.)
	];

	node.child_nodes = Some(if node.is_octree {
		vec![
			// back left lower
			QuadOctreeNode::new(BoundingBox {
				start_pos: node.bbox.start_pos,
				end_pos: mid_pos
			}, true),
			// back right lower
			QuadOctreeNode::new(BoundingBox {
				start_pos: [mid_pos[0], node.bbox.start_pos[1], node.bbox.start_pos[2]],
				end_pos: [node.bbox.end_pos[0], mid_pos[1], mid_pos[2]]
			}, true),
			// front left lower
			QuadOctreeNode::new(BoundingBox {
				start_pos: [node.bbox.start_pos[0], node.bbox.start_pos[1], mid_pos[2]],
				end_pos: [mid_pos[0], mid_pos[1], node.bbox.end_pos[2]]
			}, true),
			// front right lower
			QuadOctreeNode::new(BoundingBox {
				start_pos: [mid_pos[0], node.bbox.start_pos[1], mid_pos[2]],
				end_pos: [node.bbox.end_pos[0], mid_pos[1], node.bbox.end_pos[2]]
			}, true),
			// back left upper
			QuadOctreeNode::new(BoundingBox {
				start_pos: [node.bbox.start_pos[0], mid_pos[1], node.bbox.start_pos[2]],
				end_pos: [mid_pos[0], node.bbox.end_pos[1], mid_pos[1]]
			}, true),
			// back right upper
			QuadOctreeNode::new(BoundingBox {
				start_pos: [mid_pos[0], mid_pos[1], node.bbox.start_pos[2]],
				end_pos: [node.bbox.end_pos[0], node.bbox.end_pos[1], mid_pos[2]]
			}, true),
			// front left upper
			QuadOctreeNode::new(BoundingBox {
				start_pos: [node.bbox.start_pos[0], mid_pos[1], mid_pos[2]],
				end_pos: [mid_pos[0], node.bbox.end_pos[1], node.bbox.end_pos[2]]
			}, true),
			// front right upper
			QuadOctreeNode::new(BoundingBox {
				start_pos: mid_pos,
				end_pos: node.bbox.end_pos
			}, true)
		]
	} else {
		vec![
			// back left
			QuadOctreeNode::new(BoundingBox {
				start_pos: node.bbox.start_pos,
				end_pos: [mid_pos[0], node.bbox.end_pos[1], mid_pos[2]]
			}, false),
			// back right
			QuadOctreeNode::new(BoundingBox {
				start_pos: [mid_pos[0], node.bbox.start_pos[1], node.bbox.start_pos[2]],
				end_pos: [node.bbox.end_pos[0], node.bbox.end_pos[1], mid_pos[2]]
			}, false),
			// front left
			QuadOctreeNode::new(BoundingBox {
				start_pos: [node.bbox.start_pos[0], node.bbox.start_pos[1], mid_pos[2]],
				end_pos: [mid_pos[0], node.bbox.end_pos[1], node.bbox.end_pos[2]]
			}, false),
			// front right
			QuadOctreeNode::new(BoundingBox {
				start_pos: [mid_pos[0], node.bbox.start_pos[1], mid_pos[2]],
				end_pos: [node.bbox.end_pos[0], node.bbox.end_pos[1], node.bbox.end_pos[2]]
			}, false)
		]
	});
}

fn obj_is_in_bbox(bbox: &BoundingBox, obj: &CollisionObj) -> bool {
	let triangle = obj.triangle.as_ref().unwrap();
	for vert in triangle {
		if !(vert[0] >= bbox.start_pos[0] && vert[0] < bbox.end_pos[0]
			&& vert[1] >= bbox.start_pos[1] && vert[1] < bbox.end_pos[1]
			&& vert[2] >= bbox.start_pos[2] && vert[2] < bbox.end_pos[2]) {
			return false;
		}
	}
	true
}

fn vert_is_in_bbox(bbox: &BoundingBox, vert: &[f32; 3]) -> bool {
	vert[0] >= bbox.start_pos[0] && vert[0] < bbox.end_pos[0]
		&& vert[1] >= bbox.start_pos[1] && vert[1] < bbox.end_pos[1]
		&& vert[2] >= bbox.start_pos[2] && vert[2] < bbox.end_pos[2]
}

fn insert_helper(node: &mut QuadOctreeNode, obj: CollisionObj) -> Result<(), QuadOctreeError> {
	let child_nodes = node.child_nodes.as_mut().unwrap();
	for child_node in child_nodes {
		if obj_is_in_bbox(&child_node.bbox, &obj) {
			return insert_quadoctree_item(child_node, obj);
		}
	}
	if node.items.len() >= BUCKET_CAPACITY {
		return Err(QuadOctreeError::BucketFull);
	}
	node.items.push(obj);
	Ok(())
}

pub fn insert_quadoctree_item(node: &mut QuadOctreeNode, obj: CollisionObj) -> Result<(), QuadOctreeError> {
	if node.child_nodes.is_none() {
		if node.items.len() < BUCKET_CAPACITY {
			node.items.push(obj);
			return Ok(());
		}

		create_sub_nodes(node);
		let items_clone = node.items.clone();
		node.items.clear();
		for item in items_clone {
			insert_helper(node, item)?;
		}
	}

	insert_helper(node, obj)
}

pub fn traverse_quadoctree<T>(node: &QuadOctreeNode, vertex: &[f32; 3], check_func: &mut T) -> bool where T: FnMut(&CollisionObj) -> bool {
	if let Some(child_nodes) = node.child_nodes.as_ref() {
		for child_node in (*child_nodes).iter() {
			if !vert_is_in_bbox(&child_node.bbox, vertex) {
				continue;
			}
			if traverse_quadoctree(child_node, vertex, check_func) {
				return true;
			}
		}
	}
	for item in &node.items {
		if check_func(&item) {
			return true;
		}
	}
	return false;
}

pub fn build_quadoctree_from_triangles(octree: &mut QuadOctreeNode, vertices: &[Vertex], indices: &[u32]) -> Result<(), QuadOctreeError> {
	for vertex in vertices {
		for i in 0..3 {
			if vertex.position[i] < octree.bbox.start_pos[i] {
				octree.bbox.start_pos[i] = vertex.position[i];
			}
			if vertex.position[i] > octree.bbox.end_pos[i] {
				octree.bbox.end_pos[i] = vertex.position[i] + 0.01;
			}
		}
	}

	// add room for testing the camera vertex
	octree.bbox.end_pos[1] += 1.;

	for i in (0..indices.len()).step_by(3) {
		insert_quadoctree_item(octree, CollisionObj {
			triangle: Some([vertices[indices[i] as usize].position, vertices[indices[i + 1] as usize].position,
						  vertices[indices[i + 2] as usize].position])
		})?;
	}

	Ok(())
}
