#version 450

layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec3 out_pos;

layout(set = 0, binding = 0) uniform Light {
    mat4 views[6];
    mat4 projection;
    float pos_x;
    float pos_y;
    float pos_z;
    float z_far;
    float strength;
} u_shadow;

layout(set = 1, binding = 0) uniform Instance {
    mat4 model;
} u_instance;

layout(push_constant) uniform Face {
    uint face;
};

void main() {
    out_pos = (u_instance.model * vec4(in_pos, 1.0)).xyz;
    gl_Position = u_shadow.projection * u_shadow.views[face] * u_instance.model * vec4(in_pos, 1.0);
}