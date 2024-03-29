#version 450

// layout(set = 0, binding = 0) uniform PushData {
//     vec4 rt;
// } u_data;

layout(push_constant) uniform PushData {
    vec4 rt;
};

#define SMAA_RT_METRICS rt
#define SMAA_GLSL_4
#define SMAA_INCLUDE_PS 0
#include "SMAA.hlsl"

layout(location = 0) out vec4 offset0;
layout(location = 1) out vec4 offset1;
layout(location = 2) out vec4 offset2;
layout(location = 3) out vec2 texcoord;

void main() {
    if (gl_VertexIndex == 0) gl_Position = vec4(-1.0, -1.0, 1.0, 1.0);
    if (gl_VertexIndex == 1) gl_Position = vec4(-1.0, 3.0, 1.0, 1.0);
    if (gl_VertexIndex == 2) gl_Position = vec4(3.0, -1.0, 1.0, 1.0);
    texcoord = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
    vec4 offset[3];
    SMAAEdgeDetectionVS(texcoord, offset);
    offset0=offset[0];
    offset1=offset[1];
    offset2=offset[2];
}