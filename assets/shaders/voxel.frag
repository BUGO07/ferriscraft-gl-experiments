#version 330 core

in vec3 v_pos;
in vec3 v_normal;
in vec2 v_uv;
in float v_ao;

flat in uint v_block_id;

out vec4 color;

uniform sampler2D tex;
uniform vec4 u_light;
uniform bool apply_ao;

const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
    vec3 light_pos = u_light.xyz;
    float diffuse = max(dot(normalize(v_normal), light_pos), 0.0);

    vec3 camera_dir = normalize(-v_pos);
    vec3 half_direction = normalize(light_pos + camera_dir);
    vec3 diffuse_color = texture(tex, v_uv).xyz;

    vec3 ambient_color = diffuse_color * 0.7;
    if (apply_ao) {
        ambient_color *= v_ao;
    }

    vec3 final_color = ambient_color + diffuse * diffuse_color * u_light.w / 800.0;
    if (apply_ao) {
        final_color *= v_ao;
    }

    // check if block is water or something and only then apply specular reflection;
    if (v_block_id == 6u) {
        float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
        final_color += specular * specular_color * u_light.w / 2500.0; // idk idc

        color = vec4(final_color, 0.6);
    } else {
        color = vec4(final_color, 1.0);
    }
}