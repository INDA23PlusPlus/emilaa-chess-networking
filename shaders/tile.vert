#version 460
layout(location = 0) in vec2 i_pos;

uniform mat4 model;
uniform mat4 projection;

void main() {
    gl_Position = projection * model * vec4(i_pos, 0, 1.0);
}