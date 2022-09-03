#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 in_uv;
// layout(location = 1) in vec2 in_uv_l;
// layout(location = 2) in vec2 in_uv_r;
// layout(location = 3) in vec2 in_uv_t;
// layout(location = 4) in vec2 in_uv_b;

layout(set = 0, binding = 0) uniform texture2D u_color;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform Data {
    vec2 texel_size;
    float strength;
};

void main() {
    vec4 sum = vec4(0.0);
    sum += texture(sampler2D(u_color, u_sampler), in_uv);
    // sum += texture(sampler2D(u_color, u_sampler), in_uv - vec2(texel_size.x, 0.0));
    // sum += texture(sampler2D(u_color, u_sampler), in_uv + vec2(texel_size.x, 0.0));
    // sum += texture(sampler2D(u_color, u_sampler), in_uv + vec2(0.0, texel_size.y));
    // sum += texture(sampler2D(u_color, u_sampler), in_uv - vec2(0.0, texel_size.y));
    // sum += texture(sampler2D(u_color, u_sampler), in_uv_l);
    // sum += texture(sampler2D(u_color, u_sampler), in_uv_r);
    // sum += texture(sampler2D(u_color, u_sampler), in_uv_t);
    // sum += texture(sampler2D(u_color, u_sampler), in_uv_b);
    // sum *= 0.2;
    out_color = sum;
}