#version 330 core

#include common.glsl

in vec3 v_uv;

out vec4 color;

uniform samplerCube skybox;
uniform float time;

void main() {
    color = mix(mix(vec4(0.0,0.0,0.0,1.0), vec4(1.0,0.37,0.0,1.0), day_factor(time)), texture(skybox, v_uv), day_factor(time));
}