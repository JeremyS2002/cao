# cao
Experimenting with Vulkan and Rust

## gpu
Low level wrapper of vulkan. Manages creation and destruction of vulkan objects as well as hiding some of the more unpleasant parts of the vulkan api.

## gfx
Utilities built on top of gpu. Specifically: simpler command buffer recording, reflected pipelines, image loading, mesh creation.

## spirv
Create spir-v shader modules at runtime from rust code, including pre-compiled shaders is better if possible but quickly becomes a pain when trying to build shaders dynamically.
