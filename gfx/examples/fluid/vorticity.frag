#version 450

layout(location = 0) out vec2 out_velocity;

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec2 in_uv_l;
layout(location = 2) in vec2 in_uv_r;
layout(location = 3) in vec2 in_uv_t;
layout(location = 4) in vec2 in_uv_b;

layout(set = 1, binding = 0) uniform VorticityParams {
    float curl;
    float dt;
} u_params;

layout(set = 1, binding = 1) uniform texture2D u_velocity;
layout(set = 1, binding = 2) uniform texture2D u_curl;
layout(set = 1, binding = 3) uniform sampler u_sampler;

void main() {
    float L = texture(sampler2D(u_curl, u_sampler), in_uv_l).x;
    float R = texture(sampler2D(u_curl, u_sampler), in_uv_r).x;
    float T = texture(sampler2D(u_curl, u_sampler), in_uv_t).x;
    float B = texture(sampler2D(u_curl, u_sampler), in_uv_b).x;
    float C = texture(sampler2D(u_curl, u_sampler), in_uv).x;

    vec2 force = 0.5 * vec2(abs(T) - abs(B), abs(R) - abs(L));
    force /= length(force) + 0.0001;
    force *= u_params.curl * C;
    force.y *= -1.0;

    vec2 velocity = texture(sampler2D(u_velocity, u_sampler), in_uv).xy;
    velocity += force * u_params.dt;
    velocity = min(max(velocity, -1000.0), 1000.0);

    out_velocity = velocity;
}