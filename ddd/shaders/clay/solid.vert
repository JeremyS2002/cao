#version 450

layout(location = 0) in vec3 in_pos;

layout(set = 0, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec3 position;
} u_camera;

layout(set = 1, binding = 0) buffer Instance {
    mat4 models[];
} u_instances;

void main() {
    mat4 model = u_instances.models[gl_InstanceIndex];
    gl_Position = u_camera.projection * u_camera.view * model * vec4(in_pos, 1.0);
}

