#version 330 core

in vec3 pos;
in vec2 tex_coord;

out vec2 my_tex_coord;

void main()
{
	gl_Position = vec4(pos, 1.0);
	my_tex_coord = tex_coord;
}
