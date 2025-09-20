#version 330 core

#include common.glsl

in vec3 v_pos;
in vec3 v_normal;
in vec2 v_uv;
in float v_ao;

flat in uint v_block_id;

out vec4 color;

uniform sampler2D tex;
uniform float u_light;
uniform vec4 base_color;
uniform float time;

const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
    vec3 light_dir = normalize((rotate_x(time / secs_in_day * -pi * 2.0) * vec4(0.2, -1.0, 0.0, 0.0)).xyz);
    float diffuse = max(dot(normalize(v_normal), light_dir), 0.0);

    vec3 camera_dir = normalize(-v_pos);
    vec3 half_direction = normalize(light_dir + camera_dir);
    vec3 diffuse_color = texture(tex, v_uv).xyz;

    vec3 ambient_color = diffuse_color * 0.3;

    vec3 final_color = ambient_color + diffuse * diffuse_color * v_ao * u_light / 800.0;

    // check if block is water or something and only then apply specular reflection;
    if (v_block_id == 6u) {
        float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
        final_color += specular * specular_color * u_light / 2500.0; // idk idc

        color = vec4(final_color, 0.6) * base_color;
    } else {
        color = vec4(final_color, 1.0) * base_color;
    }
}