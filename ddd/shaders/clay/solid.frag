#version 450

layout(location = 0) out vec4 out_color;

layout(push_constant) uniform PushData {
    vec4 u_color;
};

void main() {
    out_color = u_color;
}