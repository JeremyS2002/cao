#version 450

layout(location = 0) in vec3 in_uv;

layout(location = 0) out vec3 out_color;

layout(set = 1, binding = 0) uniform textureCube u_skybox;
layout(set = 1, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform PushData {
    float strength;
};

void main() {
    out_color = texture(samplerCube(u_skybox, u_sampler), in_uv).rgb * strength;
}