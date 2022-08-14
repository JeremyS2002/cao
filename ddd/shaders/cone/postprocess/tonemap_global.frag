#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(set = 1, binding = 0) uniform Params {
    float a;
    float b;
    float c;
    float d;
    float e;
    float f;
    float w;
} u;

vec4 f(vec4 x) {
    return ((x * (u.a * x + u.c * u.b) + u.d * u.e) / (x * (u.a * x + u.b) + u.d * u.f)) - u.e / u.f;
}

void main() {
    vec4 t = texture(sampler2D(u_texture, u_sampler), in_uv);
    vec4 w = vec4(u.w);
    out_color = f(t) / f(w);
}