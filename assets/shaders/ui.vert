#version 330 core

layout(location = 0) in uint char;

out vec2 v_uv;

uniform vec2 u_pos;
uniform vec2 u_size;

vec2 offsets[4] = vec2[4](
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0));

vec2 get_font_uv(int c, vec2 local_uv) {
    int col = c % 16;
    int row = c / 16;
    vec2 pixel_pos = vec2(
        float(col) * 9.0, 
        float(row) * 17.0);
    vec2 glyph_size = vec2(8.0, 16.0);
    vec2 pixel_offset = local_uv * glyph_size;
    vec2 final_pixel = pixel_pos + pixel_offset;
    return final_pixel / vec2(144.0,-136.0);
}

void main() {
    vec2 local_uv = offsets[gl_VertexID % 4];
    v_uv = get_font_uv(int(char) + 34, local_uv); // 33 empty spaces + 1 from rust
    gl_Position = vec4(u_pos + u_size * local_uv, 0.0, 1.0);
}
