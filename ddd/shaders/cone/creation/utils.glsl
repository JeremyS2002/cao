
#include "../utils.glsl"

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