#version 330 core

layout(location = 0) in uint corner;

uniform vec2 pos;
uniform vec2 size;

void main() {
    vec2 position = vec2(pos.x, pos.y);
    if (corner == 0u) position = vec2(pos.x + size.x, pos.y);
    else if (corner == 1u) position = vec2(pos.x + size.x, pos.y + size.y);
    else if (corner == 2u) position = vec2(pos.x, pos.y + size.y);
    gl_Position = vec4(position, 0.0, 1.0);
}
