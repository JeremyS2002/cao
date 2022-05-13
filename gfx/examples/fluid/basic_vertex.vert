
#version 450

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec2 out_uv_l;
layout(location = 2) out vec2 out_uv_r;
layout(location = 3) out vec2 out_uv_t;
layout(location = 4) out vec2 out_uv_b;

layout(set = 0, binding = 0) uniform VertexParams {
    vec2 texel_size;
} u_vertex_params;

void main() {
    gl_Position = vec4(in_pos, 0.0, 1.0);
    out_uv = in_uv;
    out_uv_l = in_uv - vec2(u_vertex_params.texel_size.x, 0.0);
    out_uv_r = in_uv + vec2(u_vertex_params.texel_size.x, 0.0);
    out_uv_t = in_uv + vec2(0.0, u_vertex_params.texel_size.y);
    out_uv_b = in_uv - vec2(0.0, u_vertex_params.texel_size.y);
}