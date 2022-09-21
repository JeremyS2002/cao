# spirv
Create spir-v shader modules at runtime from rust code, including pre-compiled shaders is better if possible but quickly becomes a pain when trying to build shaders dynamically.

# Examples

Build a vertex module that takes as input position, normal, uv as well as uniform view projection and model matrices and transforms them to pass to the fragment shader.
```rust
let vertex_spv: Vec<u32> = {
    let b = spv::Builder::new();

    let in_pos = b.in_vec3(0, "in_pos");
    let in_normal = b.in_vec3(1, "in_normal");
    let in_uv = b.in_vec2(2, "in_uv");

    let vk_pos = b.vk_position();
    
    let out_pos = b.out_vec3(0, "out_pos");
    let out_normal = b.out_vec3(1, "out_normal");
    let out_uv = b.out_vec2(2, "out_uv");

    let u_model = b.uniform::<spv::Mat4>(0, 0, Some("u_model"));
    let u_view = b.uniform::<spv::Mat4>(0, 1, Some("u_view"));
    let u_projection = b.uniform::<spv::Mat4>(0, 2, Some("u_projection"));

    b.entry(spv::Stage::Vertex, "main", || {
        let pos = in_pos.load();
        let normal = in_normal.load();
        let uv = in_uv.load();

        let projection = u_projection.load();
        let view = u_view.load();
        let model = u_model.load();

        let world_pos = model * b.vec4(pos.x(), pos.y(), pos.z(), 1.0);
        out_pos.store(world_pos.xyz());
        vk_pos.store(projection * view * world_pos);
        
        let model3: spv::Mat3 = model.into();
        out_normal.store(model3 * normal);

        out_uv.store(uv);
    });

    b.compile();
};
```

Build a fragment module that takes as input world_pos, normal, uv as well as texture and light data and outputs color

```rust
let fragment_spv: Vec<u32> = {
    let b = spv::Builder::new();

    let in_pos = b.in_vec3(0, "in_pos");
    let in_normal = b.in_vec3(1, "in_normal");
    let in_uv = b.in_vec2(2, "in_uv");

    let out_color = b.out_vec4(0, "out_color");

    let u_light = b.uniform::<spv::Vec3>(0, 0, Some("u_light"))
    
    let u_texture = b.texture2d(0, 1, Some("u_texture"));
    let u_sampler = b.sampler(0, 2, Some("u_sampler"));

    b.entry(spv::Stage::Fragment, "main", || {
        let pos = in_pos.load();
        let normal = in_normal.load();
        let uv = in_uv.load();

        let light_pos = u_light.load();

        let mut to_light = light_pos - pos;
        let dist = to_light.length();
        to_light.normalize();

        let attenuation = 1.0 / (dist * dist);
        
        let diffuse = normal.dot(to_light); 

        let combined = spv::combine(&u_texture, u_sampler);
        let mut color = spv::sample(&combined, uv);
        color *= attenuation;
        color *= diffuse;
        
        out_color.store(color);
    })
};
```

The resulting `Vec<u32>` from `Builder::compile` can then be passed into vulkans VkShaderModuleCreateInfo.pCode to run the shaders in a pipeline.

More example can be found in example folder (run from insed `cao/spirv` to avoid path errors in loading files)
|Name          |Command                             |Description                                                                               |
|--------------|------------------------------------|------------------------------------------------------------------------------------------|
|basic         |`cargo run --example basic`         |Pass through vertex and fragment shaders                                                  |
|uniform       |`cargo run --example uniform`       |Using a uniform buffer to draw solid color                                                |
|push_constants|`cargo run --example push_constants`|Using push constants to draw solid color                                                  |
|condition     |`cargo run --example condition`     |Control flow in fragment shader, (click on the window to change the color)                |
|texture       |`cargo run --example texture`       |Using a texture to draw an image                                                          |
|camera        |`cargo run --example camera`        |Using model, view and projection matrices to transform vertices into screen space         |