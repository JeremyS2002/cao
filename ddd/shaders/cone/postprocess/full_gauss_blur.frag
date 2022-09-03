#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_color;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform Data {
    vec2 texel_size;
    float radius;
};

void main() {
    vec4 result = vec4(0.0);
    float div = 0.0;
    float w0 = 0.3780 / pow(radius, 1.975);
    float radius2 = radius * radius;
    for (float x = -radius; x <= radius; x++) {
        for (float y = -radius; y <= radius; y++) {
            vec2 offset = vec2(x, y);
            float n = dot(offset, offset);
            if (n <= radius2) {
                offset *= texel_size;
                vec2 coord = in_uv + offset;
                if (coord.x > 1.0 || coord.x < -1.0) {
                    continue;
                }
                if (coord.y > 1.0 || coord.y < -1.0) {
                    continue;
                }
                float w = w0 * exp(-n/(2.0 * radius2));
                result += w * texture(sampler2D(u_color, u_sampler), coord);
                div += w;
                // result += texture(sampler2D(u_color, u_sampler), coord);
                // div += 1.0;
            }
        }
    }
    out_color = result / div;
}