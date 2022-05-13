#version 450

layout(location = 0) out float out_pressure;

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec2 in_uv_l;
layout(location = 2) in vec2 in_uv_r;
layout(location = 3) in vec2 in_uv_t;
layout(location = 4) in vec2 in_uv_b;

layout(set = 1, binding = 0) uniform texture2D u_pressure;
layout(set = 1, binding = 1) uniform texture2D u_divergence;
layout(set = 1, binding = 2) uniform sampler u_sampler;

void main() {
    float L = texture(sampler2D(u_pressure, u_sampler), in_uv_l).x;
    float R = texture(sampler2D(u_pressure, u_sampler), in_uv_r).x;
    float T = texture(sampler2D(u_pressure, u_sampler), in_uv_t).x;
    float B = texture(sampler2D(u_pressure, u_sampler), in_uv_b).x;
    float C = texture(sampler2D(u_pressure, u_sampler), in_uv).x;
    float divergence = texture(sampler2D(u_divergence, u_sampler), in_uv).x;
    out_pressure = (L + R + B + T - divergence) * 0.25;
}