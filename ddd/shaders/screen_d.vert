#version 450

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec2 out_uv_l;
layout(location = 2) out vec2 out_uv_r;
layout(location = 3) out vec2 out_uv_t;
layout(location = 4) out vec2 out_uv_b;

layout(push_constant) uniform Data {
    vec2 texel_size;
};

void main() {
    if (gl_VertexIndex == 0) gl_Position = vec4(-1.0, -1.0, 1.0, 1.0);
    if (gl_VertexIndex == 1) gl_Position = vec4(3.0, -1.0, 1.0, 1.0);
    if (gl_VertexIndex == 2) gl_Position = vec4(-1.0, 3.0, 1.0, 1.0);
    out_uv = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
    out_uv_l = out_uv - vec2(texel_size.x, 0.0);
    out_uv_r = out_uv + vec2(texel_size.x, 0.0);
    out_uv_t = out_uv + vec2(0.0, texel_size.y);
    out_uv_b = out_uv - vec2(0.0, texel_size.y);
}