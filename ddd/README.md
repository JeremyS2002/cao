
# ddd

# examples
run form inside `cao/ddd` (to avoid path errors in loading files)
|Name          |Command                             |Description                                                                               |
|--------------|------------------------------------|------------------------------------------------------------------------------------------|
|cone          |`cargo run --example cone`          |Demonstrates cone - the deferred physically based renderer                                |
|clay          |`cargo run --example clay`          |Demonstrates clay - the forward debugging renderer                                        |

## TODO

- cone
    - [x] deferred rendering
    - [x] image based lighting
    - [ ] physically based camera
    - [ ] point lights
        - [x] basic lighting
        - [x] shadow maps
        - [x] subsurface scattering
        - [ ] shadow volumes
        - [ ] raytraced shadows
    - [ ] directional lights
        - [ ] basic lighting
        - [ ] shadow maps
        - [ ] subsurface scattering
        - [ ] shadow volumes
        - [ ] raytraced shadows
    - [ ] volume lights
        - [ ] basic lighting
        - [ ] shadows
    - [ ] hdr
        - [ ] gamma correction
        - [x] reinhard
        - [x] aces
        - [ ] exposure
    - [ ] antialiasing
        - [x] msaa
        - [ ] smaa (1x ✔️ T2x ❌ S2x ❌ 4x ❌)
    - [x] normal mapping
    - [ ] parallax mapping
    - [ ] postprocessing
        - [x] bloom
        - [x] ambient occlusion
        - [ ] screen space reflections
        - [ ] screen space refractions
        - [ ] outlining
        - [ ] fog
        - [ ] motion blur
        - [ ] depth of field
        - [ ] screen space global illumination
    - [ ] skeletal animation
    - [ ] reflection probes
- clay
    - [ ] basic template
    - [ ] wireframing
    - [ ] transparency
    - [ ] highliting objects
