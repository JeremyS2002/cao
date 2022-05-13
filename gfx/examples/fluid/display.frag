#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 1, binding = 0) uniform texture2D u_ink;
layout(set = 1, binding = 1) uniform sampler u_sampler;

void main() {
    out_color = vec4(texture(sampler2D(u_ink, u_sampler), in_uv).xyz, 1.0);
}