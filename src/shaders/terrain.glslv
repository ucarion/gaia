#version 150 core

in vec2 a_coord;
uniform mat4 u_mvp;
uniform vec2 u_offset;
uniform float u_width;
uniform usampler2D t_elevation;

out vec2 v_tex_coord;

const float ELEVATION_SCALE_FACTOR = 200.0;

void main() {
    v_tex_coord = a_coord;

    float x =  a_coord.x * u_width + u_offset.x;
    float y = -a_coord.y * u_width - u_offset.y;

    uint elevation = texture(t_elevation, a_coord).r;
    float z = float(elevation) / ELEVATION_SCALE_FACTOR;

    vec4 position = vec4(x, y, z, 1.0);
    gl_Position = u_mvp * position;
}
