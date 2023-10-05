#version 460
layout(location = 0) out vec4 o_color;

uniform vec4 color;

void main() {
    o_color = color;
}