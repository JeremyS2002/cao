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

layout(set = 2, binding = 0) uniform textureCube u_diffuse;
layout(set = 2, binding = 1) uniform textureCube u_specular;
layout(set = 2, binding = 2) uniform texture2D u_brdf_lut;

layout(push_constant) uniform Data {
    float max_reflection_lod;
    float strength;
    float width;
    float height;
};

void main() {
    vec2 uv = vec2(gl_FragCoord.xy) / vec2(width, height);
    vec3 position = texture(sampler2D(u_position, u_sampler), uv).xyz;
    vec3 normal = texture(sampler2D(u_normal, u_sampler), uv).xyz;
    vec4 albedo = texture(sampler2D(u_albedo, u_sampler), uv);
    float metallic = texture(sampler2D(u_metallic, u_sampler), uv).x;
    float roughness = texture(sampler2D(u_roughness, u_sampler), uv).x;

    vec3 view = normalize(u_camera.position - position);
    vec3 ref = reflect(-view, normal);
    ref.y *= -1.0; // correct for vulkan coordinates

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo.rgb, metallic);

    vec3 f = fresnelSchlickRoughness(max(dot(normal, view), 0.0), f0, roughness);

    vec3 kS = fresnelSchlickRoughness(max(dot(normal, view), 0.0), f0, roughness);
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metallic;
    vec3 irradiance_sample = normal;
    irradiance_sample.y *= -1.0;
    vec3 irradiance = texture(samplerCube(u_diffuse, u_sampler), irradiance_sample).rgb;
    vec3 diffuse = irradiance * albedo.rgb;

    vec3 prefilteredColor = textureLod(samplerCube(u_specular, u_sampler), ref, roughness * max_reflection_lod).rgb;
    vec2 envBRDF = texture(sampler2D(u_brdf_lut, u_sampler), vec2(max(dot(normal, view), 0.0), roughness)).rg;
    vec3 specular = prefilteredColor * (f * envBRDF.x + envBRDF.y);

    out_color = vec4(strength * (kD * diffuse + specular), albedo.a);
}
