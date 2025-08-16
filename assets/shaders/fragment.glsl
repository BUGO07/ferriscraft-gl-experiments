#version 140

in vec3 v_pos;
in vec3 v_normal;
in vec2 v_uv;

out vec4 color;

uniform sampler2D tex;
uniform vec3 u_light;

// const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
    float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);

    vec3 camera_dir = normalize(-v_pos);
    vec3 half_direction = normalize(normalize(u_light) + camera_dir);
    vec3 diffuse_color = texture(tex, v_uv).xyz;
    vec3 ambient_color = diffuse_color * 0.7; // maybe lower this

    vec3 final_color = ambient_color + diffuse * diffuse_color;

    // check if block is water or something and only then apply specular reflection;
    // float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
    // final_color + specular * specular_color;

    color = vec4(final_color, 1.0);
}