#version 450
/// Used for manually resolving depth texture as atm this is not possible otherwise

layout(set = 0, binding = 0) uniform sampler2DMS u_texture;

void main() {
    ivec2 pos = ivec2(gl_FragCoord.x, gl_FragCoord.y);
    int samples = textureSamples(u_texture);
    float tmp = 0.0;
    bool edge = false;
    for (int i = 0; i < samples; i++) {
        float depth = texelFetch(u_texture, pos, i).x;
        tmp += depth;
    }
    gl_FragDepth = tmp / float(samples);
}
