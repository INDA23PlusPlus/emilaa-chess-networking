#version 460
layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 o_color;

uniform sampler2D sprites;

void main() {
    vec4 col = texture(sprites, uv);

    if (col.w < 0.01) { discard; }

    o_color = col;
}