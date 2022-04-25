#version 450

layout(location = 0) out vec4 out_color;

layout(push_constant) uniform constants {
    vec4 color;
} u_constants;

void main() {
    out_color = u_constants.color;
}