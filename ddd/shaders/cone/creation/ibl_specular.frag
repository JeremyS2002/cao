#version 450

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform textureCube u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(set = 0, binding = 2) uniform Data {
    uint sample_count;
    float roughness;
    uint width;
    uint height;
} u_data;

const float PI = 3.14159265359;

float distribution_ggx(vec3 n, vec3 h, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float n_dot_h = max(dot(n, h), 0.0);
    float n_dot_h_2 = n_dot_h * n_dot_h;

    float nom = a2;
    float denom = (n_dot_h * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

float radical_inverse_vdc(uint bits) {
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}

vec2 hammersley(uint i, uint n) {
    return vec2(float(i)/float(n), radical_inverse_vdc(i));
}

vec3 importance_sample_ggx(vec2 x_i, vec3 n, float roughness) {
    float a = roughness * roughness;

    float phi = 2.0 * PI * x_i.x;
    float cos_theta = sqrt((1.0 - x_i.y) / (1.0 + (a * a - 1.0) * x_i.y));
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    // spherical to cartesian
    vec3 h = vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
    
    // tangent space to world space
    vec3 up = abs(n.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent = normalize(cross(up, n));
    vec3 bitangent = cross(n, tangent);

    vec3 sample_vec = tangent * h.x + bitangent * h.y + n * h.z;
    return normalize(sample_vec);
}

void main() {
    vec3 n = normalize(in_pos);
    vec3 r = n;
    vec3 v = r;

    float weight = 0.0;
    vec3 prefiltered_color = vec3(0.0);

    for (uint i = 0u; i < u_data.sample_count; i++) {
        vec2 x_i = hammersley(i, u_data.sample_count);
        vec3 h = importance_sample_ggx(x_i, n, u_data.roughness);
        vec3 l = normalize(2.0 * dot(v, h) * h - v);

        float n_dot_l = max(dot(n, l), 0.0);
        if (n_dot_l > 0.0) {
            float d = distribution_ggx(n, h, u_data.roughness);
            float n_dot_h = max(dot(n, h), 0.0);
            float h_dot_v = max(dot(h, v), 0.0);
            float pdf = d * n_dot_h / (4.0 * h_dot_v) + 0.0001;

            float sa_texel = 4.0 * PI / (6.0 * float(u_data.width) * float(u_data.height));
            float sa_sample = 1.0 / (float(u_data.sample_count) * pdf + 0.0001);

            float mip_level = u_data.roughness == 0.0 ? 0.0 : 0.5 * log2(sa_sample / sa_texel);

            prefiltered_color += textureLod(samplerCube(u_texture, u_sampler), l, mip_level).rgb * n_dot_l;
            weight += n_dot_l;
        }
    }

    prefiltered_color = prefiltered_color / weight;

    out_color = vec4(prefiltered_color, 1.0);
}