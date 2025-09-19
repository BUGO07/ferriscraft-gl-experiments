#version 330 core

#include common.glsl

layout (location = 0) in vec3 pos;

out vec3 v_uv;

uniform mat4 projection;
uniform mat4 view;
uniform float time;

void main() {
    v_uv = (rotate_y(time / secs_in_day * -pi * 2) * vec4(pos, 1.0)).xyz;
    vec4 p = projection * mat4(mat3(view)) * vec4(pos, 1.0);
    gl_Position = p.xyww;
}