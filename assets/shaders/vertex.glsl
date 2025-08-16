#version 140

in vec3 pos;
in vec3 normal;
in vec2 uv;

out vec3 v_pos;
out vec3 v_normal;
out vec2 v_uv;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model; // for rotation or smth

void main() {
    v_uv = uv;

    mat4 modelview = view * model;
    v_normal = transpose(inverse(mat3(modelview))) * normal;

    gl_Position = perspective * modelview * vec4(pos, 1.0);
    v_pos = gl_Position.xyz / gl_Position.w;
}