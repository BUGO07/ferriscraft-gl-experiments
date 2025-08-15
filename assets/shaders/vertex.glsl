#version 140

in vec3 position;
in vec2 uvs;
out vec2 v_uvs;

uniform mat4 matrix; // for rotation or smth

void main() {
    v_uvs = uvs;
    gl_Position = matrix * vec4(position, 1.0);
}