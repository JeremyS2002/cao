#version 450

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_albedo;
layout(set = 0, binding = 1) uniform texture2D u_ao;
layout(set = 0, binding = 2) uniform sampler u_sampler;

layout(push_constant) uniform Strength {
    float strength;
    float width;
    float height;
};

void main() {
    vec2 uv = vec2(gl_FragCoord.xy) / vec2(width, height);
    vec4 albedo = texture(sampler2D(u_albedo, u_sampler), uv);
    float ao = texture(sampler2D(u_ao, u_sampler), uv).x;
    out_color = vec4(albedo.rgb * strength * ao, albedo.a);
}
