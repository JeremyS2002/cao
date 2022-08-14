#version 450

layout(location = 0) out vec2 out_uv;

void main() {
    if (gl_VertexIndex == 0) gl_Position = vec4(-1.0, -1.0, 1.0, 1.0);
    if (gl_VertexIndex == 1) gl_Position = vec4(3.0, -1.0, 1.0, 1.0);
    if (gl_VertexIndex == 2) gl_Position = vec4(-1.0, 3.0, 1.0, 1.0);
    out_uv = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
}