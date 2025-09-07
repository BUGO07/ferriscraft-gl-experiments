#version 330 core

layout (location = 0) in vec3 pos;

out vec3 v_uv;

uniform mat4 projection;
uniform mat4 view;

void main() {
    v_uv = pos;
    vec4 p = projection * view * vec4(pos, 1.0);
    gl_Position = p.xyww;
}