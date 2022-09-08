#version 450

#include "utils.glsl"

layout(location = 0) in vec2 in_uv;

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
    vec4 position;
    float z_far;
} u_camera;

layout(set = 2, binding = 0) uniform LightData {
    PointLightData light;
} u_light_data;

layout(set = 3, binding = 0) uniform ShadowData {
    PointDepthData depth;
} u_shadow_data;

layout(set = 3, binding = 1) uniform samplerCube u_shadow_map;

layout(set = 4, binding = 0) uniform SubsurfaceData {
    PointDepthData depth;
} u_subsurface_data;

layout(set = 4, binding = 1) uniform samplerCube u_subsurface_map;

layout(set = 4, binding = 2) uniform sampler1D u_subsurface_lut;

layout(push_constant) uniform PushData {
    uint subsurface_samples;
    uint shadow_samples;
    float strength;
    float width;
    float height;
};

void main() {
    vec3 world_pos = texture(sampler2D(u_position, u_sampler), in_uv).xyz;
    vec3 normal = texture(sampler2D(u_normal, u_sampler), in_uv).xyz;
    vec4 albedo = texture(sampler2D(u_albedo, u_sampler), in_uv);
    float roughness = texture(sampler2D(u_roughness, u_sampler), in_uv).x;
    float metallic = texture(sampler2D(u_metallic, u_sampler), in_uv).x; 
    vec4 subsurface = texture(sampler2D(u_subsurface, u_sampler), in_uv);

    float shadow = point_shadow_calc(
        u_shadow_data.depth,
        world_pos,
        normal,
        u_shadow_map,
        int(shadow_samples)
    );

    vec3 lighting_color = vec3(0.0);
    if (shadow != 1.0) {
        lighting_color = point_light_calc(
            u_light_data.light,
            u_camera.position.xyz,
            world_pos,
            normal,
            albedo.rgb,
            roughness,
            metallic
        );
    }

    if (subsurface.a != 0.0) {
        PointDepthData depth = u_subsurface_data.depth;
        PointLightData light = u_light_data.light;

        vec3 light_pos = vec3(light.position_x, light.position_y, light.position_z);
        vec3 to_light = light_pos - world_pos;
        float distance2 = dot(to_light, to_light);
        float attenuation = 1.0 / (0.001 + light.falloff * distance2);

        vec3 subsurface_pos = vec3(depth.pos_x, depth.pos_y, depth.pos_z);
        vec3 to_subsurface = world_pos - subsurface_pos;
        float current_depth = length(to_subsurface);
        vec3 subsurface_sample = to_subsurface;
        subsurface_sample.y *= -1.0;

        vec3 light_color = vec3(u_light_data.light.color_r, u_light_data.light.color_g, u_light_data.light.color_b);

        vec3 subsurf_color = vec3(0.0);

        int i_samples = int(subsurface_samples);
        float disk_radius = depth.strength * (1.0 + (current_depth / depth.z_far));
        for (int i = 0; i < i_samples; i++) {
            float tmp_depth = texture(u_subsurface_map, subsurface_sample + sampleOffsetDirections[i] * disk_radius).r;
            tmp_depth *= depth.z_far;
            float dist = max(current_depth - tmp_depth, 0.0);
            // subsurface_color += exp(-dist) * subsurface.rgb;
            subsurf_color += texture(u_subsurface_lut, dist / depth.z_far).r * subsurface.rgb;
        }

        subsurf_color /= float(i_samples);
        subsurf_color *= attenuation;
        subsurf_color *= subsurface.a;

        lighting_color *= (1.0 - subsurface.a);

        out_color = vec4(strength * ((1.0 - shadow) * lighting_color + subsurf_color), albedo.a);
    } else {
        out_color = vec4(strength * (1.0 - shadow) * lighting_color, albedo.a);
    }
}