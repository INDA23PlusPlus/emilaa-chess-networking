#version 460
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 o_color;

uniform sampler2D text;
uniform vec4 color;

void main() {    
    vec4 tex_col = vec4(1.0, 1.0, 1.0, texture(text, uv).r);

    o_color = color * tex_col;
}  
