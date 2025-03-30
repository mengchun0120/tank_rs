#version 330 core

out vec4 out_frag_color;

in vec2 my_tex_coord;

uniform sampler2D texture1;

void main()
{
	out_frag_color = texture(texture1, my_tex_coord);
}
