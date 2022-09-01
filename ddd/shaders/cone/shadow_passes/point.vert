#version 450

layout(location = 0) in vec3 in_pos;

layout(location = 0) out float out_depth;

layout(set = 0, binding = 0) uniform Light {
    mat4 views[6];
    mat4 projection;
    float pos_x;
    float pos_y;
    float pos_z;
    float z_far;
    float strength;
} u_shadow;

layout(set = 1, binding = 0) buffer Instances {
    mat4 models[];
} u_instance;

layout(push_constant) uniform Face {
    uint face;
};

void main() {
    // vec4 world_pos = u_instance.models[gl_InstanceIndex] * vec4(in_pos, 1.0);
    // gl_Position = u_shadow.projection * u_shadow.views[face] * world_pos;
    
    vec4 world_pos = u_instance.models[gl_InstanceIndex] * vec4(in_pos, 1.0);
    vec3 light_pos = vec3(u_shadow.pos_x, u_shadow.pos_y, u_shadow.pos_z);
    float dist = length(world_pos.xyz - light_pos);
    dist /= u_shadow.z_far;
    out_depth = dist;
    gl_Position = u_shadow.projection * u_shadow.views[face] * world_pos;
}