#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out float out_ao;

layout(set = 0, binding = 0) uniform texture2D u_ao_input;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform Data {
    vec2 texel_size;
};

void main() {
    float result = 0.0;
    for (int x = -2; x < 2; x++) {
        for (int y = -2; y < 2; y++) {
            vec2 offset = vec2(float(x), float(y)) * texel_size;
            result += texture(sampler2D(u_ao_input, u_sampler), in_uv + offset).r;
        }
    }
    out_ao = result / 16.0;
}
