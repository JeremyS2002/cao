#version 450

layout(set = 0, binding = 0) uniform PushData {
    vec4 rt;
} u_data;

#define SMAA_RT_METRICS u_data.rt
#define SMAA_GLSL_4
#define SMAA_INCLUDE_VS 0
#include "SMAA.hlsl"

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec4 offset;
layout(location = 1) in vec2 texcoord;

layout(set = 0, binding = 1) uniform sampler2D u_color;
layout(set = 0, binding = 2) uniform sampler2D u_blend;

void main() {
    out_color = SMAANeighborhoodBlendingPS(texcoord, offset, u_color, u_blend);
}