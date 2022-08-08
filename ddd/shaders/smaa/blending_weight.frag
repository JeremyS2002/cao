#version 450

layout(set = 0, binding = 0) uniform PushData {
    vec4 rt;
} u_data;

#define SMAA_RT_METRICS u_data.rt
#define SMAA_GLSL_4
#define SMAA_INCLUDE_VS 0
#include "SMAA.hlsl"

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 pixcoord;
layout(location = 1) in vec4 offset0;
layout(location = 2) in vec4 offset1;
layout(location = 3) in vec4 offset2;
layout(location = 4) in vec2 texcoord;

layout(set = 0, binding = 1) uniform sampler2D u_edges;
layout(set = 0, binding = 2) uniform sampler2D u_area;
layout(set = 0, binding = 3) uniform sampler2D u_search;

void main() {
    vec4 subsample_indices = vec4(0.0);
    vec4 offset[3] = vec4[](offset0, offset1, offset2);
    out_color = SMAABlendingWeightCalculationPS(
        texcoord, 
        pixcoord, 
        offset, 
        u_edges, 
        u_area, 
        u_search, 
        subsample_indices
    );
}
