#version 330 core

#include common.glsl

layout(location = 0) in uint vertex_data;

out vec3 v_pos;
out vec3 v_normal;
out vec2 v_uv;
out float v_ao;

flat out uint v_block_id;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

vec2 get_uv(int normal, int block_id) {
    float face_idx = 1.0;
    if (normal == 3) face_idx = 0.0;
    else if (normal == 2) face_idx = 2.0;

    vec2 pos = vec2(face_idx / ATLAS_SIZE_X, 1.0 - (float(block_id) / ATLAS_SIZE_Y));

    vec2 base[4] = vec2[4](
        vec2(pos.x, pos.y + 1.0 / ATLAS_SIZE_Y),
        vec2(pos.x, pos.y),
        vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y),
        vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y + 1.0 / ATLAS_SIZE_Y));

    if (normal == 5 || normal == 4) return base[(gl_VertexID + 2) % 4];
    else if (normal == 0) return base[(gl_VertexID + 1) % 4];
    // my formatter breaks if i dont put the curly braces {}
    else {
        return base[(gl_VertexID + 3) % 4];
    }
}

void main() {
    vec3 pos = vec3(vertex_data & 63u, (vertex_data >> 6) & 63u, (vertex_data >> 12) & 63u);
    uint normal = (vertex_data >> 18) & 7u;
    uint ao = (vertex_data >> 21) & 3u;
    uint block_id = (vertex_data >> 23) & 63u;

    vec3 n = normals[int(normal)];
    mat4 modelview = view * model;

    gl_Position = projection * modelview * vec4(pos, 1.0);

    v_pos = gl_Position.xyz / gl_Position.w;
    v_normal = normalize(transpose(inverse(mat3(modelview))) * n);
    v_block_id = block_id;
    v_ao = ao_values[ao];
    v_uv = get_uv(int(normal), int(block_id));
}
