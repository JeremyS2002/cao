#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    float i = texture(sampler2D(u_texture, u_sampler), in_uv).x;
    out_color = vec4(vec3(i), 1.0);
}