use gfx::prelude::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// Copies the src texture view provided into the target
pub struct CopyRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub bundles: Arc<Mutex<HashMap<u64, gfx::Bundle>>>,
    pub sampler: gpu::Sampler,
}

impl CopyRenderer {
    pub fn new(device: &gpu::Device, cache: Option<gpu::PipelineCache>, name: Option<&str>) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.map(|n| format!("{}_sampler", n)),
            ..gpu::SamplerDesc::NEAREST
        })?;

        let n = name.as_ref().map(|n| format!("{}_pipeline", n));
        let pipeline = Self::create_pipeline(device, cache, n.as_ref().map(|n| &**n))?;

        Ok(Self {
            pipeline,
            bundles: Arc::default(),
            sampler,
        })
    }

    pub fn create_pipeline(
        device: &gpu::Device,
        cache: Option<gpu::PipelineCache>,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../shaders/copy.frag.spv");
        match gfx::ReflectedGraphics::from_spirv(
            device,
            &vert,
            None,
            Some(&frag),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            cache,
            name,
        ) {
            Ok(g) => Ok(g),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }
}

impl CopyRenderer {
    pub fn pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let mut pass =
            encoder.graphics_pass_reflected::<()>(device, &[target], &[], None, &self.pipeline)?;

        let mut bundles = self.bundles.lock().unwrap();
        if bundles.get(&src.id()).is_none() {
            let b = match self
                .pipeline
                .bundle()
                .unwrap()
                .set_resource("u_texture", src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(src.id(), b);
        }
        let bundle = bundles.get(&src.id()).unwrap().clone();
        pass.set_bundle_owned(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.bundles.lock().unwrap().clear();
    }
}
