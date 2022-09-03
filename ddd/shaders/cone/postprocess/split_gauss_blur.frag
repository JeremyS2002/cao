#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D u_color;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(push_constant) uniform Data {
    vec2 texel_size;
    float radius;
    int axis;
};

void main() {
    vec4 result = vec4(0.0);
    float div = 0.0;
    float w0 = 0.5135 / pow(radius, 0.96);
    float radius2 = radius * radius;

    vec2 offset_mul;
    if (axis == 0) {
        offset_mul = vec2(0.0, 1.0);
    } else {
        offset_mul = vec2(1.0, 0.0);
    }

    for (float c = -radius; c <= radius; c++) {
        vec2 offset = offset_mul * c;
        float n = dot(offset, offset);
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
    out_color = result / div;
}