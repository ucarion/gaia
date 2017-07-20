#version 150 core

in vec3 a_pos;
in vec2 a_tex_coord;
out vec2 v_TexCoord;
uniform mat4 u_model_view_proj;
uniform vec2 u_offset;

void main() {
    v_TexCoord = a_tex_coord;

    vec4 position = vec4(a_pos.xy + u_offset, a_pos.z, 1.0);
    gl_Position = u_model_view_proj * position;
}
