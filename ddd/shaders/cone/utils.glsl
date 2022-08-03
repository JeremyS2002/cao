const float PI = 3.14159265395;

vec3 fresnelSchlick(float cos_theta, vec3 f0) {
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

vec3 fresnelSchlickRoughness(float cos_theta, vec3 f0, float roughness) {
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(max(1.0 - cos_theta, 0.0), 5.0);
}

float distributionGGX(vec3 n, vec3 h, float roughness) {
    float a2 = roughness * roughness * roughness * roughness;
    float n_dot_h = max(dot(n, h), 0.0);
    float denom = (n_dot_h * n_dot_h * (a2 - 1.0) + 1.0);
    return a2 / (PI * denom * denom);
}

float geometry_schlickGGX(float n_dot_v, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

float geometry_smith(vec3 n, vec3 v, vec3 l, float roughness) {
    float n_dot_v = max(dot(n, v), 0.0);
    float n_dot_l = max(dot(n, l), 0.0);
    float ggx2 = geometry_schlickGGX(n_dot_v, roughness);
    float ggx1 = geometry_schlickGGX(n_dot_l, roughness);

    return ggx1 * ggx2;
}