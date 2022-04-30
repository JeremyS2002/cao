#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform Buffer {
    vec2 center;
    float width;
    float height;
    vec2 start_val;
    int julia;
    uint iterations;
    float offset;
};

float map(float value, float min1, float max1, float min2, float max2) {
    float perc = (value - min1) / (max1 - min1);
    return perc * (max2 - min2) + min2;
}

vec2 comp_mul(vec2 a, vec2 b) {
    vec2 c = vec2(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
    return c;
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 get_color(int i, vec2 z) {
    float mu = float(i) + 1 - log(log(length(z))) / log(2.0);
    return hsv2rgb(vec3((mu / iterations) + offset, 1.0, 1.0));
}

void main() {
    vec2 screen_coord = (in_uv - vec2(0.5)) * 2.0;
    // vulkan has y down
    screen_coord.y = -screen_coord.y;

    vec2 c;
    c.x = map(screen_coord.x, -1.0, 1.0, center.x - (width / 2.0), center.x + (width / 2.0));
    c.y = map(screen_coord.y, -1.0, 1.0, center.y - (height / 2.0), center.y + (height / 2.0));

    vec2 z = start_val;

    if (julia == 1) {
        vec2 tmp = c;
        c = z;
        z = tmp;
    }

    bool in_set = true;
    int i;
    for (i = 0; i < iterations; i++) {
        z = comp_mul(z, z) + c;
        if (length(z) > 2.0) {
            in_set = false;
            break;
        }
    }

    if (in_set) {
        out_color = vec4(vec3(0.0), 1.0);
    } else {
        out_color = vec4(get_color(i, z), 1.0);
    }
}