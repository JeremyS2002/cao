#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    vec3 hdr = texture(sampler2D(u_texture, u_sampler), in_uv).rgb;
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    // https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
    vec3 mapped = clamp(hdr * (a * hdr + b) / (hdr * (c * hdr + d)+e), vec3(0.0), vec3(1.0));
    out_color = vec4(mapped, 1.0);
}