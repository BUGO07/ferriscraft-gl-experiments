#version 330 core

in vec3 v_uv;

out vec4 color;

uniform samplerCube skybox;

void main() {
    color = texture(skybox, v_uv);
}