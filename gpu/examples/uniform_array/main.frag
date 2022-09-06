#version 450

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform Data {
    vec4 color;
} u_data[2];

layout(push_constant) uniform Index {
    uint index;
};

void main() {
    out_color = u_data[index].color;
}