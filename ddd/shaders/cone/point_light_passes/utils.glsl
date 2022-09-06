#include "../utils.glsl"

struct PointLightData {
    float falloff;
    
    float position_x;
    float position_y;
    float position_z;
    
    float color_r;
    float color_g;
    float color_b;

    float radius;
};

vec3 point_light_calc(
    PointLightData light, 
    vec3 view_pos,
    vec3 world_pos,
    vec3 normal,
    vec3 albedo,
    float roughness,
    float metallic
) {
    vec3 light_pos = vec3(light.position_x, light.position_y, light.position_z);

    vec3 view = normalize(view_pos - world_pos);
    vec3 to_light = light_pos - world_pos;
    vec3 to_light_unit = normalize(to_light);
    vec3 half_way = normalize(view + to_light);

    float distance2 = dot(to_light, to_light);
    float attenuation = 1.0 / (0.001 + light.falloff * distance2);
    vec3 radiance = vec3(light.color_r, light.color_g, light.color_b) * attenuation;

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo, metallic);
    vec3 f = fresnelSchlick(max(dot(half_way, view), 0.0), f0);

    float ndf = distributionGGX(normal, half_way, roughness);
    float g = geometry_smith(normal, view, to_light_unit, roughness);

    vec3 numerator = ndf * g * f;
    float denominator = 4.0 * max(dot(normal, view), 0.0) * max(dot(normal, to_light_unit), 0.0);
    vec3 specular = numerator / max(denominator, 0.001);
    
    // specular component weight
    vec3 ks = f;
    // diffuse component weight
    vec3 kd = vec3(1.0) - ks;
    kd *= 1.0 - metallic;

    float n_dot_l = max(dot(normal, to_light_unit), 0.0);

    return (kd * albedo / PI + specular) * radiance * n_dot_l;
}

struct PointDepthData {
    mat4 views[6];
    mat4 projection;
    float pos_x;
    float pos_y;
    float pos_z;
    float z_far;
    float strength;
    float bias;
};

vec3 sampleOffsetDirections[64] = vec3[]
(
    vec3( 0,  0,  0),
    vec3(   0,    0,  0.5),
    vec3(   0,    0, -0.5),
    vec3(   0,  0.5,    0),
    vec3(   0,  0.5,  0.5),
    vec3(   0,  0.5, -0.5),
    vec3(   0, -0.5,    0),
    vec3(   0, -0.5,  0.5),
    vec3(   0, -0.5, -0.5),
    vec3( 0.5,    0,    0),
    vec3( 0.5,    0,  0.5),
    vec3( 0.5,    0, -0.5),
    vec3( 0.5,  0.5,    0),
    vec3( 0.5,  0.5,  0.5),
    vec3( 0.5,  0.5, -0.5),
    vec3( 0.5, -0.5,    0),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5, -0.5, -0.5),
    vec3(-0.5,    0,    0),
    vec3(-0.5,    0,  0.5),
    vec3(-0.5,    0, -0.5),
    vec3(-0.5,  0.5,    0),
    vec3(-0.5,  0.5,  0.5),
    vec3(-0.5,  0.5, -0.5),
    vec3(-0.5, -0.5,    0),
    vec3(-0.5, -0.5,  0.5),
    vec3(-0.5, -0.5, -0.5),
    vec3( 0,  0,  1),
    vec3( 0,  0, -1),
    vec3( 0,  1,  0),
    vec3( 0,  1,  1),
    vec3( 0,  1, -1),
    vec3( 0, -1,  0),
    vec3( 0, -1,  1),
    vec3( 0, -1, -1),
    vec3( 1,  0,  0),
    vec3( 1,  0,  1),
    vec3( 1,  0, -1),
    vec3( 1,  1,  0),
    vec3( 1,  1,  1),
    vec3( 1,  1, -1),
    vec3( 1, -1,  0),
    vec3( 1, -1,  1),
    vec3( 1, -1, -1),
    vec3(-1,  0,  0),
    vec3(-1,  0,  1),
    vec3(-1,  0, -1),
    vec3(-1,  1,  0),
    vec3(-1,  1,  1),
    vec3(-1,  1, -1),
    vec3(-1, -1,  0),
    vec3(-1, -1,  1),
    vec3(-1, -1, -1),
    vec3(0, 1, -0.5),
    vec3(0, 1, 0.5),
    vec3(0, -1, -0.5),
    vec3(0, 1, 0.5),
    vec3(0, -0.5, 1),
    vec3(0, 0.5, 1),
    vec3(0, -0.5, -1),
    vec3(0, 0.5, 1),
    vec3(1, 0.5, 0),
    vec3(1, -0.5, 0),
    vec3(-1, 0.5, 0)
);

float point_shadow_calc(
    PointDepthData depth,
    vec3 world_pos,
    vec3 normal,
    samplerCube shadow_map,
    int samples
) {
    vec3 shadow_pos = vec3(depth.pos_x, depth.pos_y, depth.pos_z);
    vec3 to_shadow = world_pos - shadow_pos;
    float current_depth2 = dot(to_shadow, to_shadow);
    vec3 shadow_sample = to_shadow;
    shadow_sample.y *= -1.0;

    float z_far2 = depth.z_far * depth.z_far;

    if (current_depth2 >= z_far2) {
        return 0.0;
    }

    float shadow = 0.0;
    float bias = max(depth.bias * (1.0 - dot(normal, to_shadow)), depth.bias);
    float disk_radius = depth.strength * (1.0 + (current_depth2 / (z_far2)));
    for (int i = 0; i < samples; i++) {
        float tmp_depth = texture(shadow_map, shadow_sample + sampleOffsetDirections[i] * disk_radius).r;
        tmp_depth *= depth.z_far;
        tmp_depth *= tmp_depth;
        if (current_depth2 - bias >= tmp_depth)
            shadow += 1.0;
    }

    shadow /= float(samples);

    return shadow;
}