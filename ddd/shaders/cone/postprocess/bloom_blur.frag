#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_color;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform PushData {
    vec2 texel_size;
    float intensity;
};

void main() {
    vec4 sum = vec4(0.0);
    sum += texture(sampler2D(u_color, u_sampler), in_uv - vec2(texel_size.x, 0.0));
    sum += texture(sampler2D(u_color, u_sampler), in_uv + vec2(texel_size.x, 0.0));
    sum += texture(sampler2D(u_color, u_sampler), in_uv + vec2(0.0, texel_size.y));
    sum += texture(sampler2D(u_color, u_sampler), in_uv - vec2(0.0, texel_size.y));
    sum *= 0.25;
    out_color = sum * intensity;
}