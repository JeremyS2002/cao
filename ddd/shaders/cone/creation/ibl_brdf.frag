#version 450

const float PI = 3.14159265395;

layout(location = 0) out vec2 out_color;

layout(push_constant) uniform Data {
    uint sample_count;
    float width;
    float height;
};

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

float geometry_schlickGGX(float NdotV, float roughness) {
    float a = roughness;
    float k = (a * a) / 2.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlickGGX(NdotV, roughness);
    float ggx1 = geometry_schlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}  

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
    vec2 uv = vec2(gl_FragCoord.xy) / vec2(width, height);
    out_color = integrate_brdf(uv.x, uv.y);
}