#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec3 in_normal;
layout(location = 1) in vec3 in_view;

layout(push_constant) uniform PushColor {
    vec4 u_color;
};

void main() {
    out_color = vec4(u_color.rgb * abs(dot(in_normal, in_view)), u_color.a);
}