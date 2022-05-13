#version 450

layout(location = 0) out vec3 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 1, binding = 0) uniform SplatParams {
    float aspect_ratio;
    float radius;
    vec2 point;
    vec3 color;
    float mul;
} u_params;

void main() {
    vec2 p = in_uv - u_params.point;
    p.x *= u_params.aspect_ratio;
    vec3 splat = u_params.mul * exp(-dot(p, p) / u_params.radius) * u_params.color;
    out_color = splat;
}