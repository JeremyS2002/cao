#version 450

#include "utils.glsl"

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_position;
layout(set = 0, binding = 1) uniform texture2D u_normal;
layout(set = 0, binding = 2) uniform texture2D u_albedo;
layout(set = 0, binding = 3) uniform texture2D u_roughness;
layout(set = 0, binding = 4) uniform texture2D u_metallic;
layout(set = 0, binding = 5) uniform texture2D u_subsurface;
layout(set = 0, binding = 6) uniform sampler u_sampler;

layout(set = 1, binding = 0) uniform CameraData {
    mat4 projection;
    mat4 view;
    vec3 position;
} u_camera;

layout(set = 2, binding = 0) uniform Data {
    PointLightData light;
} u_light_data;

layout(push_constant) uniform PushData {
    float strength;
    float width;
    float height;
};

void main() {
    vec2 uv = vec2(gl_FragCoord.xy) / vec2(width, height);
    vec3 world_pos = texture(sampler2D(u_position, u_sampler), uv).xyz;
    vec3 normal = texture(sampler2D(u_normal, u_sampler), uv).xyz;
    vec4 albedo = texture(sampler2D(u_albedo, u_sampler), uv);
    float roughness = texture(sampler2D(u_roughness, u_sampler), uv).x;
    float metallic = texture(sampler2D(u_metallic, u_sampler), uv).x;
    float subsurface = texture(sampler2D(u_subsurface, u_sampler), uv).w;

    vec3 lighting = point_light_calc(
        u_light_data.light,
        u_camera.position,
        world_pos,
        normal,
        albedo.rgb,
        roughness,
        metallic
    );
    out_color = vec4(strength * lighting, albedo.a);
}