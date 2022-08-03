#version 450

#include "utils.glsl"

layout(location = 0) in vec3 in_pos;

layout(set = 1, binding = 0) uniform CameraData {
    mat4 projection;
    mat4 view;
    mat4 position;
} u_camera;

layout(set = 2, binding = 0) uniform Data {
    PointLightData light;
} u_light_data;

void main() {
    PointLightData light = u_light_data.light;
    vec3 light_pos = vec3(light.position_x, light.position_y, light.position_z);
    gl_Position = u_camera.projection * u_camera.view * vec4(in_pos * light.radius + light_pos, 1.0);
}