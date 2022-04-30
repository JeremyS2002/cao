#version 450

layout(location = 0) out vec3 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    out_color = texture(sampler2D(u_texture, u_sampler), in_uv).rgb;
}