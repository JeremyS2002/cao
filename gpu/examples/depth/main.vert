#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_col;

layout(location = 0) out vec3 out_col;

layout(set = 0, binding = 0) uniform Buf {
    mat4 model;
    mat4 view;
    mat4 projection;
};

void main() {
    gl_Position = projection * view * model * vec4(in_pos, 1.0);

    out_col = in_col;
}