#version 450
// this shader pre-computes ibl data from a skybox into and env map

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec3 in_pos;

layout(set = 0, binding = 0) uniform textureCube u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

const float PI = 3.14159265359;

void main() {
    vec3 normal = normalize(in_pos);
    vec3 irradiance = vec3(0.0);

    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = cross(up, normal);
    up = cross(normal, right);

    // numerical integration over hemisphere
    float sample_delta = 0.025;
    float n_samples = 0.0;
    for(float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            // basic trig to calculate sample vector in tangent space
            vec3 tangent = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            // convert tangent space into world space
            vec3 sample_vec = tangent.x * right + tangent.y * up + tangent.z * normal;
            // sample from environment and scale by angle as smaller area at top compared to bottom
            irradiance += texture(samplerCube(u_texture, u_sampler), sample_vec).rgb * cos(theta) * sin(theta);
            n_samples += 1;
        }
    }

    irradiance = PI * irradiance * (1.0 / float(n_samples));

    out_color = vec4(irradiance, 1.0);
}