#version 450
// this shader generates a skybox from an equirectangular texture

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 sample_spherical_map(vec3 v) {
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = sample_spherical_map(normalize(in_pos));
    vec3 color = texture(sampler2D(u_texture, u_sampler), uv).rgb;
    out_color = vec4(color, 1.0);
}
