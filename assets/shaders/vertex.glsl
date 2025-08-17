#version 140

in uint vertex_data;

out vec3 v_pos;
out vec3 v_normal;
out vec2 v_uv;

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

vec2[4] get_uvs(int normal, int block) {
    float face_idx = 1.0; 
    if (normal == 3) {
        face_idx = 0.0;
    }
    else if (normal == 2) {
        face_idx = 2.0;
    }

    vec2 pos = vec2(face_idx / ATLAS_SIZE_X, 1.0 - (float(block) / ATLAS_SIZE_Y));

    vec2 base[4];
    base[0] = vec2(pos.x, pos.y + 1.0 / ATLAS_SIZE_Y); 
    base[1] = vec2(pos.x, pos.y);                      
    base[2] = vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y); 
    base[3] = vec2(pos.x + 1.0 / ATLAS_SIZE_X, pos.y + 1.0 / ATLAS_SIZE_Y); 

    vec2 rotate_90[4]  = vec2[4](base[3], base[0], base[1], base[2]);
    vec2 rotate_180[4] = vec2[4](base[2], base[3], base[0], base[1]);
    vec2 rotate_270[4] = vec2[4](base[1], base[2], base[3], base[0]);

    if (normal == 5 || normal == 4) {
        return rotate_180;
    }

    else if (normal == 0) {
        return rotate_270;
    }

    else {
        return rotate_90;
    }
}

void main() {
    uint normal = (vertex_data >> 18) & 7u;
    uint corner = (vertex_data >> 21) & 3u;
    uint block  = (vertex_data >> 23) & 63u;

    vec3 pos = vec3(float(vertex_data & 63u), float((vertex_data >> 6)  & 63u), float((vertex_data >> 12) & 63u));
    vec3 n = normals[int(normal)];

    vec2 uvs[4] = get_uvs(int(normal), int(block));
    v_uv = uvs[int(corner)];

    mat4 modelview = view * model;
    v_normal = transpose(inverse(mat3(modelview))) * n;

    gl_Position = perspective * modelview * vec4(pos, 1.0);
    v_pos = gl_Position.xyz / gl_Position.w;
}