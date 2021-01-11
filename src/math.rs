
pub fn mult_matrix(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
	let mut result = [[0.0f32; 4]; 4];
	for i in 0..4 {
		for j in 0..4 {
			for k in 0..4 {
				result[i][j] += a[i][k] * b[k][j];
			}
		}
	}
	result
}

pub fn mult_matrix3(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
	let mut result = [[0.0f32; 3]; 3];
	for i in 0..3 {
		for j in 0..3 {
			for k in 0..3 {
				result[i][j] += a[i][k] * b[k][j];
			}
		}
	}
	result
}


pub fn vector_length(input: &[f32; 3]) -> f32 {
	(input[0] * input[0] + input[1] * input[1] + input[2] * input[2]).sqrt()
}

pub fn normalize_vector(input: &[f32; 3]) -> [f32; 3] {
	let len = vector_length(input);
	[input[0] / len, input[1] / len, input[2] / len]
}

pub fn cross_product(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
	[
		a[1] * b[2] - a[2] * b[1],
		a[2] * b[0] - a[0] * b[2],
		a[0] * b[1] - a[1] * b[0]
	]
}

pub fn dot_product(a: &[f32; 3], b: &[f32; 3]) -> f32 {
	a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn add_vector(a: &[f32; 3], b: &[f32; 3], b_mult: f32) -> [f32; 3] {
	[
		a[0] + b[0] * b_mult,
		a[1] + b[1] * b_mult,
		a[2] + b[2] * b_mult
	]
}

pub fn mult_vector(input: &[f32; 3], factor: f32) -> [f32; 3] {
	[
		input[0] * factor,
		input[1] * factor,
		input[2] * factor
	]
}
