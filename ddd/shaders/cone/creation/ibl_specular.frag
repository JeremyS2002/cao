#version 450

#include "utils.glsl"

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform textureCube u_texture;
layout(set = 1, binding = 1) uniform sampler u_sampler;

layout(set = 1, binding = 2) uniform Data {
    uint sample_count;
    uint width;
    uint height;
} u_data;

layout(push_constant) uniform PushData {
    float roughness;
};

void main() {
    vec3 n = normalize(in_pos);
    vec3 r = n;
    vec3 v = r;

    float weight = 0.0;
    vec3 prefiltered_color = vec3(0.0);

    for (uint i = 0u; i < u_data.sample_count; i++) {
        vec2 x_i = hammersley(i, u_data.sample_count);
        vec3 h = importance_sample_ggx(x_i, n, roughness);
        vec3 l = normalize(2.0 * dot(v, h) * h - v);

        float n_dot_l = max(dot(n, l), 0.0);
        if (n_dot_l > 0.0) {
            float d = distributionGGX(n, h, roughness);
            float n_dot_h = max(dot(n, h), 0.0);
            float h_dot_v = max(dot(h, v), 0.0);
            float pdf = d * n_dot_h / (4.0 * h_dot_v) + 0.0001;

            float sa_texel = 4.0 * PI / (6.0 * float(u_data.width) * float(u_data.height));
            float sa_sample = 1.0 / (float(u_data.sample_count) * pdf + 0.0001);

            float mip_level = roughness == 0.0 ? 0.0 : 0.5 * log2(sa_sample / sa_texel);

            prefiltered_color += textureLod(samplerCube(u_texture, u_sampler), l, mip_level).rgb * n_dot_l;
            weight += n_dot_l;
        }
    }

    prefiltered_color = prefiltered_color / weight;

    out_color = vec4(prefiltered_color, 1.0);
}