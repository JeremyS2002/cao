# spirv
Create spir-v shader modules at runtime from rust code, including pre-compiled shaders is better if possible but quickly becomes a pain when trying to build shaders dynamically.

Note: Requires nightly compiler until TypeId::of::<T>() is const

# examples
run from insed `cao/spirv` (to avoid path errors in loading files)
|Name          |Command                             |Description                                                                               |
|--------------|------------------------------------|------------------------------------------------------------------------------------------|
|basic         |`cargo run --example basic`         |Pass through vertex and fragment shaders                                                  |
|uniform       |`cargo run --example uniform`       |Using a uniform buffer to draw solid color                                                |
|storage       |`cargo run --example storage `      |Using a storage buffer to draw solid color                                                |
|push_constants|`cargo run --example push_constants`|Using push constants to draw solid color                                                  |
|condition     |`cargo run --example condition`     |Control flow in fragment shader, (click on the window to change the color)                |
|texture       |`cargo run --example texture`       |Using a texture to draw an image                                                          |
|discard       |`cargo run --example discard`       |Same as the texture example but fragments with 0.0 in alpha channel are discarded         |
|maths         |`cargo run --example maths`         |Using vectors and maths operations                                                        |
