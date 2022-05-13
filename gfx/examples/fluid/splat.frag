#version 450

layout(location = 0) out vec3 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 1, binding = 0) uniform SplatParams {
    float aspect_ratio;
    float radius;
    vec2 point;
    vec3 color;
} u_params;

layout(set = 2, binding = 0) uniform texture2D u_target;
layout(set = 2, binding = 1) uniform sampler u_sampler;

void main() {
    vec2 p = in_uv - u_params.point;
    p.x *= u_params.aspect_ratio;
    vec3 splat = exp(-dot(p, p) / u_params.radius) * u_params.color;
    vec3 base = texture(sampler2D(u_target, u_sampler), in_uv).xyz;
    out_color = base + splat;
}