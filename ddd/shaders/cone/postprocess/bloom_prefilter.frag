#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_color;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(set = 1, binding = 0) uniform Data {
    vec3 curve;
    float threshold;
    float intensity;
} u_data;

void main() {
    vec3 c = texture(sampler2D(u_color, u_sampler), in_uv).rgb;
    float br = max(c.r, max(c.g, c.b));
    float rq = clamp(br - u_data.curve.x, 0.0, u_data.curve.y);
    rq = u_data.curve.z * rq * rq;
    c *= max(rq, br - u_data.threshold) / max(br, 0.0001);
    out_color = vec4(c, 0.0);
}