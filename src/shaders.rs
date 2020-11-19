use glium::Display;

pub fn main_program(display: &Display) -> glium::Program {
	let vertex_shader_src = r#"
	#version 330 core

	in vec3 position;
	in vec3 normal;
	in vec2 texcoords;

	smooth out vec3 v_normal;
	out vec3 v_position;
	out vec3 v_light;
	out vec2 v_texcoords;

	uniform mat4 model;
	uniform mat4 view;
	uniform mat4 perspective;
	uniform vec3 u_light;

	void main() {
		mat4 modelview = view * model;
		v_normal = transpose(inverse(mat3(modelview))) * normal;
		gl_Position = perspective * modelview * vec4(position, 1.0);
		v_position = gl_Position.xyz / gl_Position.w;
		vec4 light_loc = view * vec4(u_light, 1.0);
		v_light = light_loc.xyz / light_loc.w;
		v_texcoords = texcoords;
	}
	"#;

	let fragment_shader_src = r#"
	#version 330 core

	smooth in vec3 v_normal;
	in vec3 v_position;
	in vec3 v_light;
	in vec2 v_texcoords;

	out vec4 color;
	uniform vec3 shape_color;

	uniform sampler2D tex;

	const float ambient_val = 0.1;
	const float diffuse_val = 0.6;
	const float specular_val = 0.1;

	void main() {
		float diffuse = max(dot(normalize(v_normal), normalize(v_light)), 0.0) * 0.7;

		vec3 camera_dir = normalize(-v_position);
		vec3 half_direction = normalize(normalize(v_light) + camera_dir);
		float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 256.0);

		color = vec4(((ambient_val + diffuse) * shape_color * texture(tex, v_texcoords).rgb) + (specular * specular_val), 1.0);
	}
	"#;
	glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
}
