#version 450

layout(set = 0, binding = 0) uniform Light {
    mat4 views[6];
    mat4 projection;
    float pos_x;
    float pos_y;
    float pos_z;
    float z_far;
    float strength;
} u_shadow;

layout(location = 0) in vec3 in_pos;

void main() {
    vec3 light_pos = vec3(u_shadow.pos_x, u_shadow.pos_y, u_shadow.pos_z);
    float dist = length(in_pos - light_pos);
    dist /= u_shadow.z_far;
    gl_FragDepth = dist;
}