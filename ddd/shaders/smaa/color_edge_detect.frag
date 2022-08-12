#version 450

// layout(set = 0, binding = 0) uniform PushData {
//     vec4 rt;
// } u_data;

layout(push_constant) uniform PushData {
    vec4 rt;
};

#define SMAA_RT_METRICS rt
#define SMAA_GLSL_4
#define SMAA_INCLUDE_VS 0
#include "SMAA.hlsl"

layout(location = 0) out vec2 out_color;

layout(location = 0) in vec4 offset0;
layout(location = 1) in vec4 offset1;
layout(location = 2) in vec4 offset2;
layout(location = 3) in vec2 texcoord;

layout(set = 0, binding = 0) uniform sampler2D u_tex;

void main() {
    vec4 offset[3] = vec4[](offset0, offset1, offset2);
    out_color = SMAAColorEdgeDetectionPS(texcoord, offset, u_tex);
}