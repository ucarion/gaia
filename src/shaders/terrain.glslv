#version 150 core

in vec2 a_coord;
uniform mat4 u_mvp;
uniform vec2 u_offset;
uniform float u_width;
uniform usampler2D t_elevation;

out vec2 v_tex_coord;

// TODO: Find a better name for this variable
const float ELEVATION_COMPRESSION_FACTOR = 0.0001;
const float MAX_Z = 30.0;

float elevation_to_z(float elevation) {
    float t = 1.0 - 1.0 / (1.0 + ELEVATION_COMPRESSION_FACTOR * elevation);
    return t * MAX_Z;
}

void main() {
    v_tex_coord = a_coord;

    float x =  a_coord.x * u_width + u_offset.x;
    float y = -a_coord.y * u_width - u_offset.y;

    uint elevation = texture(t_elevation, a_coord).r;
    float z = elevation_to_z(float(elevation));

    vec4 position = vec4(x, y, z, 1.0);
    gl_Position = u_mvp * position;
}
