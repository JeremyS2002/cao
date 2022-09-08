#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out float out_brightness;

layout(set = 0, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec4 position;
    float z_far;
} u_camera;

layout(set = 1, binding = 0) buffer Instances {
    mat4 models[];
} u_instances;

void main() {
    mat4 model = u_instances.models[gl_InstanceIndex];
    vec4 world_pos = model * vec4(in_pos, 1.0);
    gl_Position = u_camera.projection * u_camera.view * world_pos;
    vec3 normal = normalize(mat3(model) * in_normal);
    vec3 to_view = normalize(world_pos.xyz - u_camera.position.xyz);
    out_brightness = abs(dot(normal, to_view));
}