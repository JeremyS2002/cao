#version 450

layout(location = 0) in vec3 in_col;

layout(location = 0) out vec3 out_col;

void main() {
    out_col = in_col;
}