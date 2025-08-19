#version 330 core

layout(location = 0) in uint vertex_data;

out vec3 v_pos;
out vec3 v_normal;
out vec2 v_uv;

flat out uint v_block;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

const float ATLAS_SIZE_X = 3.0; 
const float ATLAS_SIZE_Y = 10.0;

const vec3 normals[6] = vec3[6](
    vec3(-1, 0, 0), 
    vec3(1, 0, 0), 
    vec3(0,-1, 0), 
    vec3(0, 1, 0), 
    vec3(0, 0,-1), 
    vec3(0, 0, 1));

vec2 get_uv(int corner, int normal, int block) {
    float face_idx = 1.0;
    if (normal == 3) face_idx = 0.0;
    else if (normal == 2) face_idx = 2.0;

    vec2 pos = vec2(face_idx / ATLAS_SIZE_X, 1.0 - (float(block) / ATLAS_SIZE_Y));

    vec2 base[4] = vec2[4](
        vec2(pos.x, pos.y + 1.0 / ATLAS_SIZE_Y),
        vec2(pos.x, pos.y),
        vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y),
        vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y + 1.0 / ATLAS_SIZE_Y)
    );

    int rotated_corner = corner;
    if (normal == 5 || normal == 4) rotated_corner = (corner + 2) % 4;
    else if (normal == 0) rotated_corner = (corner + 1) % 4;
    else rotated_corner = (corner + 3) % 4;

    return base[rotated_corner];
}

void main() {
    uint normal = (vertex_data >> 18) & 7u;
    uint corner = (vertex_data >> 21) & 3u;
    uint block  = (vertex_data >> 23) & 63u;

    v_block = block;

    vec3 pos = vec3(float(vertex_data & 63u), float((vertex_data >> 6)  & 63u), float((vertex_data >> 12) & 63u));
    vec3 n = normals[int(normal)];

    v_uv = get_uv(int(corner), int(normal), int(block));

    mat4 modelview = view * model;
    v_normal = normalize(transpose(inverse(mat3(modelview))) * n);

    gl_Position = perspective * modelview * vec4(pos, 1.0);
    v_pos = gl_Position.xyz / gl_Position.w;
}
