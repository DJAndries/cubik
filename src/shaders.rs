use glium::Display;

pub fn main_program(display: &Display) -> glium::Program {
	let vertex_shader_src = r#"
	#version 330 core

	#define MAX_LIGHTS 4

	in vec3 position;
	in vec3 normal;
	in vec2 texcoords;

	smooth out vec3 v_normal;
	out vec3 v_position;
	out vec3 v_light;
	out vec2 v_texcoords;
	out vec3 v_lights[MAX_LIGHTS];

	uniform mat4 model;
	uniform mat4 view;
	uniform mat4 perspective;
	uniform vec3 lights[MAX_LIGHTS];
	uniform int light_count;

	void main() {
		mat4 modelview = view * model;
		v_normal = transpose(inverse(mat3(modelview))) * normal;
		gl_Position = perspective * modelview * vec4(position, 1.0);
		v_position = vec3(modelview * vec4(position, 1.0));

		for (int i = 0; i < light_count; i++) {
			v_lights[i] = vec3(view * vec4(lights[i], 1.0));
		}
		v_texcoords = texcoords;
	}
	"#;

	let fragment_shader_src = r#"
	#version 330 core

	#define MAX_LIGHTS 4

	smooth in vec3 v_normal;
	in vec3 v_position;
	in vec3 v_light;
	in vec2 v_texcoords;
	in vec3 v_lights[MAX_LIGHTS];

	out vec4 color;
	uniform vec3 shape_color;

	uniform sampler2D tex;
	uniform vec2 texcoord_displacement;
	uniform int light_count;

	const float ambient_val = 0.1;
	const float diffuse_val = 0.6;
	const float specular_val = 0.1;

	void main() {
		vec4 text_val = texture(tex, v_texcoords + texcoord_displacement);

		color = vec4(0.0, 0.0, 0.0, text_val.a);
		for (int i = 0; i < light_count; i++) {
			vec3 norm = normalize(v_normal);
			vec3 light_dir = normalize(v_lights[i] - v_position);
			float diffuse = max(dot(norm, light_dir), 0.0) * 0.7;

			vec3 view_dir = normalize(-v_position);
			vec3 reflect_dir = reflect(-light_dir, norm);
			float specular = pow(max(dot(view_dir, reflect_dir), 0.0), 256.0);

			float distance = length(v_lights[i] - v_position);
			float attenuation = 1.0 / (distance * distance);

			color += vec4((diffuse * shape_color * text_val.rgb) + (specular * specular_val), 0.0) * attenuation;
		}
	}
	"#;
	glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
}

pub fn skybox_program(display: &Display) -> glium::Program {
	let vertex_shader_src = r#"
	#version 330 core

	in vec3 position;

	out vec3 v_texcoords;

	uniform mat4 view;
	uniform mat4 perspective;

	void main() {
		v_texcoords = position;
		gl_Position = perspective * view * vec4(position, 1.0);
	}
	"#;

	let fragment_shader_src = r#"
	#version 330 core

	in vec3 v_texcoords;

	out vec4 color;

	uniform samplerCube cubemap;

	void main() {
		color = texture(cubemap, v_texcoords);
	}
	"#;
	glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
}

pub fn ui_program(display: &Display) -> glium::Program {
	let vertex_shader_src = r#"
	#version 330 core

	in vec3 position;
	in vec2 texcoords;

	out vec2 v_texcoords;
	uniform float left_clip;
	uniform mat3 model;

	void main() {
		v_texcoords = texcoords;
		gl_Position = vec4(model * vec3(position.xy, 1.0), 1.0);
		gl_ClipDistance[0] = gl_Position.x - (model * vec3(left_clip, 0.0, 0.0)).x;
	}
	"#;

	let fragment_shader_src = r#"
	#version 330 core

	in vec2 v_texcoords;

	out vec4 color;

	uniform sampler2D tex;
	uniform vec4 text_color;

	void main() {
		vec4 tex_val = texture(tex, v_texcoords);
		color = vec4(text_color.rgb * tex_val.rgb, tex_val.a * text_color.a);
	}
	"#;
	glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
}
