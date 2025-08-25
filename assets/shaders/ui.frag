#version 330 core

in vec2 v_uv;

out vec4 color;

// for later
uniform sampler2D tex;
uniform vec4 base_color;

void main() {
    vec4 sampled = texture(tex, v_uv);
    if (sampled == vec4(0.0,0.0,0.0,1.0)) {
        color = vec4(0.0,0.0,0.0,0.0);
    } else {
        color = sampled * base_color;
    }
}
