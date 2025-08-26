#version 330 core

in vec2 v_uv;

out vec4 color;

// for later
uniform sampler2D tex;
uniform vec4 base_color;

void main() {
    color = texture(tex, v_uv) * base_color;
}
