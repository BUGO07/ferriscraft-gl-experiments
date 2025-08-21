#version 330 core

uniform vec2 pos;
uniform vec2 size;

vec2 offsets[4] = vec2[4](
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0));

void main() {
    gl_Position = vec4(pos + size * offsets[gl_VertexID % 4], 0.0, 1.0);
}
