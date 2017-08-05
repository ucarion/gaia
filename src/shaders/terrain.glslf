#version 150 core

in vec2 v_tex_coord;
uniform sampler2D t_color;

out vec4 o_color;

void main() {
    o_color = texture(t_color, v_tex_coord);
}
