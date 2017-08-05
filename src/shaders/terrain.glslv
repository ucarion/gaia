#version 150 core

uniform mat4 u_mvp;
in vec2 a_coord;

out vec2 v_tex_coord;

/* in vec3 a_pos; */
/* in vec2 a_tex_coord; */
/* out vec2 v_TexCoord; */
/* uniform mat4 u_model_view_proj; */
/* uniform vec2 u_offset; */

void main() {
    v_tex_coord = a_coord;

    vec4 position = vec4(a_coord, 0.0, 1.0);
    gl_Position = u_mvp * position;

    /* vec4 position = vec4(a_pos.xy + u_offset, a_pos.z, 1.0); */
    /* gl_Position = u_model_view_proj * position; */
}
