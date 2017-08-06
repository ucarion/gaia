#version 150 core

uniform mat4 u_mvp;
uniform usampler2D t_elevation;
in vec2 a_coord;

out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_coord;

    uint elevation = texture(t_elevation, a_coord).r;
    float z = float(elevation) / 100.0;
    vec4 position = vec4(a_coord.x * 1000.0, -a_coord.y * 1000.0, z, 1.0);
    gl_Position = u_mvp * position;
}
