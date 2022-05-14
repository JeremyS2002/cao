
#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 1, binding = 0) uniform AdvectionParams {
    vec2 sim_texel_size;
    float dissipation;
    float dt;
} u_params;

layout(set = 1, binding = 1) uniform texture2D u_velocity;
layout(set = 2, binding = 0) uniform texture2D u_source;
layout(set = 2, binding = 1) uniform sampler u_sampler;

void main() {
    vec2 coord = in_uv - u_params.dt * texture(sampler2D(u_velocity, u_sampler), in_uv).xy * u_params.sim_texel_size;    
    vec4 result = texture(sampler2D(u_source, u_sampler), coord);
    float decay = 1.0 + u_params.dt * u_params.dissipation;
    out_color = result / decay;
}