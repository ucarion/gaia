#version 150 core

in vec2 a_coord;
uniform mat4 u_mvp;
uniform usampler2D t_elevation;

out vec2 v_tex_coord;

// TODO: Find a better name for this variable
const float ELEVATION_COMPRESSION_FACTOR = 0.0001;

// This needs to be the same value as in constants.rs
const float MAX_Z = 0.03;

float elevation_to_z(float elevation) {
    float t = 1.0 - 1.0 / (1.0 + ELEVATION_COMPRESSION_FACTOR * elevation);
    return t * MAX_Z;
}

void main() {
    v_tex_coord = a_coord;

    uint elevation = texture(t_elevation, a_coord).r;
    float z = elevation_to_z(float(elevation));
    vec4 position = vec4(a_coord, z, 1.0);

    gl_Position = u_mvp * position;
}
