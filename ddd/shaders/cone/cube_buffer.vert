#version 450
// intended for drawing a cube with a camera as a uniform buffer

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec3 out_pos;

layout(set = 0, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
    vec3 position;
} u_camera;

void main() {
    // remove translation from model
    vec4 pos = u_camera.projection * mat4(mat3(u_camera.view)) * vec4(in_pos, 0.0);
    out_pos = in_pos;
    gl_Position = pos.xyww;
}
