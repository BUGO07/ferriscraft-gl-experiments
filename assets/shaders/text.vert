#version 330 core

layout(location = 0) in vec2 pos;
layout(location = 1) in uint char;

out vec2 v_uv;

uniform vec2 u_size;

vec2 offsets[4] = vec2[4](
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0));

void main() {
    vec2 local_uv = offsets[gl_VertexID % 4];
    vec2 cr = vec2(float(char % 13u), float(char / 13u));
    vec2 font_size = vec2(6.0, 10.0);
    v_uv = (font_size * (cr + local_uv)) / vec2(78.0, -70.0);
    gl_Position = vec4(pos + u_size * local_uv, 0.0, 1.0);
}
