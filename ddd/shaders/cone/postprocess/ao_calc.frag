#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out float out_ao;

layout(set = 0, binding = 0) uniform texture2D u_position;
layout(set = 0, binding = 1) uniform texture2D u_normal;
layout(set = 0, binding = 2) uniform sampler u_buf_sampler;

layout(set = 1, binding = 0) uniform texture2D u_noise;
layout(set = 1, binding = 1) uniform sampler u_noise_sampler;

layout(set = 1, binding = 2) uniform Data {
    uint kernel_size;
    float radius;
    float bias;
    float power;
    vec3 samples[64];
} u_data;

layout(set = 2, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec3 position;
} u_camera;

layout(push_constant) uniform PushData {
    vec2 noise_scale;
};

void main() {
    vec3 position = texture(sampler2D(u_position, u_buf_sampler), in_uv).xyz;
    vec3 normal = texture(sampler2D(u_normal, u_buf_sampler), in_uv).xyz;
    // tile the noise texture over the screen by scaling the uv and mirroring the texture
    vec3 random = texture(sampler2D(u_noise, u_noise_sampler), in_uv * noise_scale).xyz;

    vec3 tangent = normalize(random - normal * dot(random, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 tbn = mat3(tangent, bitangent, normal);

    float occlusion = 0.0;
    for (uint i = 0; i < u_data.kernel_size; i++) {
        vec3 sample_pos = position + (tbn * u_data.samples[i]) * u_data.radius;

        vec4 offset = u_camera.projection * u_camera.view * vec4(sample_pos, 1.0);
        offset.xyz /= offset.w;
        offset.xyz = offset.xyz * 0.5 + 0.5;

        float sample_depth = texture(sampler2D(u_position, u_buf_sampler), offset.xy).z;

        // if the difference in the depth is much bigger than the radius then it's probably sampling
        // from a surface far behind or infront of the actual object
        float range_check = smoothstep(0.0, 1.0, u_data.radius / abs(position.z - sample_depth));

        occlusion += (sample_depth >= sample_pos.z + u_data.bias ? 1.0 : 0.0) * range_check;
    }
    occlusion = 1.0 - (occlusion / float(u_data.kernel_size));

    out_ao = pow(occlusion, u_data.power);
}