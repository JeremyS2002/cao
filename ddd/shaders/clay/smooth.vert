#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 out_normal;
layout(location = 1) out vec3 out_view;

layout(set = 0, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec3 position;
} u_camera;

layout(set = 1, binding = 0) uniform Instance {
    mat4 model;
} u_instance;

void main() {
    vec4 world_pos = u_instance.model * vec4(in_pos, 1.0);
    gl_Position = u_camera.projection * u_camera.view * world_pos;
    out_normal = normalize(mat3(u_instance.model) * in_normal);
    out_view = normalize(world_pos.xyz - u_camera.position);
}