#version 330 core

layout (location = 0) in vec3 pos;

out vec3 v_uv;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = perspective * view * model * vec4(pos, 1.0);
}