#version 450

layout(location = 0) out vec4 out_color;

layout(location = 0) in float in_brightness;

layout(push_constant) uniform PushColor {
    vec4 u_color;
};

void main() {
    out_color = vec4(u_color.rgb * in_brightness, u_color.a);
}

