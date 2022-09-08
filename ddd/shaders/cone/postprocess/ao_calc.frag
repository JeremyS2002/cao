#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out float out_ao;

layout(set = 0, binding = 0) uniform texture2D u_position;
layout(set = 0, binding = 1) uniform texture2D u_normal;
layout(set = 0, binding = 2) uniform sampler u_buf_sampler;

layout(set = 1, binding = 0) uniform texture2D u_noise;
layout(set = 1, binding = 1) uniform sampler u_noise_sampler;

struct Sample {
    float x;
    float y;
    float z;
};

// I fucking hate glsl memory alignment requirements
// Jesus christ I always forget about them and have to spend hours
// debugging wtf is going on with shader code
// two days of debugging why ao looked kinda right but had artifacts
// fixed by: vec3 samples[64] -> vec4 samples[64], urrrrrghhhh
layout(set = 1, binding = 2) uniform Data {
    vec4 samples[64];
    int kernel_size;
    float radius;
    float bias;
    float power;
} u_data;

layout(set = 2, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec4 position;
    float z_far;
} u_camera;

layout(push_constant) uniform PushData {
    vec2 noise_scale;
};

float calc(int i, vec3 position, mat3 tbn) {
    // Sample tangent_sample_s = u_data.samples[i];
    // vec3 tangent_sample = vec3(tangent_sample_s.x, tangent_sample_s.y, tangent_sample_s.z);
    vec3 tangent_sample = u_data.samples[i].xyz;
    vec3 sample_pos = position + (tbn * tangent_sample) * u_data.radius;

    vec4 offset = u_camera.projection * vec4(sample_pos, 1.0);
    offset.xyz /= offset.w;
    offset.xyz = offset.xyz * 0.5 + 0.5;

    float sample_depth = texture(sampler2D(u_position, u_buf_sampler), offset.xy).z;

    // if the difference in the depth is much bigger than the radius then it's probably sampling
    // from a surface far behind or infront of the actual object which wouldn't be occluding this fragment
    float range_check = smoothstep(0.0, 1.0, u_data.radius / abs(sample_pos.z - sample_depth));

    return (sample_depth >= sample_pos.z + u_data.bias ? 1.0: 0.0) * range_check;
}

void main() {
    vec3 position = texture(sampler2D(u_position, u_buf_sampler), in_uv).xyz;
    vec3 normal = texture(sampler2D(u_normal, u_buf_sampler), in_uv).xyz;
    mat3 normal_matrix = transpose(inverse(mat3(u_camera.view)));
    // mat3 normal_matrix = mat3(u_camera.view);
    normal = normalize(normal_matrix * normal);
    // tile the noise texture over the screen by scaling the uv and mirroring the texture
    vec3 random = vec3(texture(sampler2D(u_noise, u_noise_sampler), in_uv * noise_scale).xy, 0.0);

    vec3 tangent = normalize(random - normal * dot(random, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 tbn = mat3(tangent, bitangent, normal);

    float occlusion = 0.0;
    for (int i = 0; i < u_data.kernel_size; i++) {
        occlusion += calc(i, position, tbn);
    }

    occlusion = 1.0 - (occlusion / float(u_data.kernel_size));

    out_ao = pow(occlusion, u_data.power);
}