#version 450

#include "utils.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec2 out_color;

layout(push_constant) uniform Data {
    uint sample_count;
};

vec2 integrate_brdf(float n_dot_v, float roughness) {
    vec3 v = vec3(sqrt(1.0 - n_dot_v * n_dot_v), 0.0, n_dot_v);

    float a = 0.0;
    float b = 0.0;

    vec3 n = vec3(0.0, 0.0, 1.0);
    for (uint i = 0u; i < sample_count; i++) {
        vec2 x_i = hammersley(i, sample_count);
        vec3 h = importance_sample_ggx(x_i, n, roughness);
        vec3 l = normalize(2.0 * dot(v, h) * h - v);

        float n_dot_l = max(l.z, 0.0);
        float n_dot_h = max(h.z, 0.0);
        float v_dot_h = max(dot(v, h), 0.0);

        if (n_dot_l > 0.0) {
            float g = geometry_smith(n, v, l, roughness);
            float g_vis = (g * v_dot_h) / (n_dot_h * n_dot_v);
            float fc = pow(1.0 - v_dot_h, 5.0);

            a += (1.0 - fc) * g_vis;
            b += fc * g_vis;
        }
    }
    a /= float(sample_count);
    b /= float(sample_count);
    return vec2(a, b);
}

void main() {
    out_color = integrate_brdf(in_uv.x, in_uv.y);
}