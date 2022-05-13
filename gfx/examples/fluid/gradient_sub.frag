#version 450

layout(location = 0) out vec2 out_velocity;

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec2 in_uv_l;
layout(location = 2) in vec2 in_uv_r;
layout(location = 3) in vec2 in_uv_t;
layout(location = 4) in vec2 in_uv_b;

layout(set = 1, binding = 0) uniform texture2D u_pressure;
layout(set = 2, binding = 0) uniform texture2D u_velocity;
layout(set = 2, binding = 1) uniform sampler u_sampler;

void main() {
    float L = texture(sampler2D(u_pressure, u_sampler), in_uv_l).x;
    float R = texture(sampler2D(u_pressure, u_sampler), in_uv_r).x;
    float T = texture(sampler2D(u_pressure, u_sampler), in_uv_t).x;
    float B = texture(sampler2D(u_pressure, u_sampler), in_uv_b).x;
    vec2 velocity = texture(sampler2D(u_velocity, u_sampler), in_uv).xy;
    velocity -= vec2(R - L, T - B);
    out_velocity = velocity;
}