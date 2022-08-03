#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    vec3 hdr = texture(sampler2D(u_texture, u_sampler), in_uv).rgb;
    vec3 mapped = hdr / (hdr + vec3(1.0));
    out_color = vec4(mapped, 1.0);
}