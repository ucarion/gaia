#version 150 core

in vec2 v_TexCoord;
out vec4 o_Color;
uniform sampler2D t_color;

void main() {
    o_Color = texture(t_color, v_TexCoord);
}
