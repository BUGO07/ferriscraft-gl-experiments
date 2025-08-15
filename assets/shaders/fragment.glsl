#version 140

in vec2 v_uvs;
out vec4 color;

uniform sampler2D tex;

void main() {
    color = texture(tex, v_uvs);
}