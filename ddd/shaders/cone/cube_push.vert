#version 450
// intended for drawing a cube with a camera set once via push constants

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec3 out_pos;

layout(push_constant) uniform Camera {
    mat4 projection;
    mat4 view;
};

void main() {
    vec4 pos = projection * view * vec4(in_pos, 1.0);
    out_pos = in_pos;
    gl_Position = pos;
}
