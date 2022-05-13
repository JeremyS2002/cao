#version 450

layout(location = 0) out float out_pressure;

layout(location = 0) in vec2 in_uv;

layout(set = 1, binding = 0) uniform Params {
    float pressure;
} u_params;

layout(set = 1, binding = 1) uniform texture2D u_pressure;
layout(set = 1, binding = 2) uniform sampler u_sampler;

void main() {
    out_pressure = u_params.pressure * texture(sampler2D(u_pressure, u_sampler), in_uv).x;
}