#version 450

layout(location = 0) out float out_divergence;

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec2 in_uv_l;
layout(location = 2) in vec2 in_uv_r;
layout(location = 3) in vec2 in_uv_t;
layout(location = 4) in vec2 in_uv_b;

layout(set = 1, binding = 0) uniform texture2D u_velocity;
layout(set = 1, binding = 1) uniform sampler u_sampler;

void main() {
    float L = texture(sampler2D(u_velocity, u_sampler), in_uv_l).x;
    float R = texture(sampler2D(u_velocity, u_sampler), in_uv_r).x;
    float T = texture(sampler2D(u_velocity, u_sampler), in_uv_t).y;
    float B = texture(sampler2D(u_velocity, u_sampler), in_uv_b).y;

    // boundry condition no slip on boundries
    vec2 C = texture(sampler2D(u_velocity, u_sampler), in_uv).xy;
    if (in_uv_l.x < 0.0) { L = -C.x; }
    if (in_uv_r.x > 1.0) { R = -C.x; }
    if (in_uv_t.y > 1.0) { T = -C.y; }
    if (in_uv_b.y < 0.0) { B = -C.y; }

    out_divergence = 0.5 * (R - L + T - B);
}