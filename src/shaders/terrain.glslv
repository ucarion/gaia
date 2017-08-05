#version 150 core

uniform mat4 u_mvp;
in vec2 a_coord;

out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_coord;

    vec4 position = vec4(a_coord.x * 1000.0, -a_coord.y * 1000.0, 0.0, 1.0);
    gl_Position = u_mvp * position;
}
