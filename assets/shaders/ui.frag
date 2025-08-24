#version 330 core

out vec4 color;

// for later
uniform sampler2D tex;
uniform vec4 base_color;

void main() {
    color = vec4(1.0, 1.0, 1.0, 1.0) * base_color;
}
