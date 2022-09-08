use gfx::prelude::*;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cone::GeometryBuffer;

use super::ChainBlurRenderer;

/// Parameters to tweak how bloom is applied
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BloomParams {
    pub curve: glam::Vec3,
    pub threshold: f32,
    pub intensity: f32,
}

unsafe impl bytemuck::Pod for BloomParams {}
unsafe impl bytemuck::Zeroable for BloomParams {}

impl BloomParams {
    pub fn new(intensity: f32, threshold: f32, soft_knee: f32) -> Self {
        // let intensity: f32 = 0.8;
        // let threshold: f32 = 0.6;
        // let soft_knee: f32 = 0.7;

        let knee = threshold * soft_knee + 0.0001;
        let c0 = threshold - knee;
        let c1 = knee * 2.0;
        let c2 = 0.25 / knee;

        Self {
            curve: glam::vec3(c0, c1, c2),
            threshold,
            intensity,
        }
    }
}

/// Renders bloom into the output of
#[derive(Debug, Clone)]
pub struct BloomRenderer {
    pub prefilter_pipeline: gfx::ReflectedGraphics,
    pub prefiltered: Arc<Mutex<HashMap<u64, (gfx::GTexture2D, gfx::Bundle)>>>,
    pub blur_renderer: ChainBlurRenderer,
    pub uniform: gfx::Uniform<BloomParams>,
    pub intensity: f32,
    pub name: Option<String>,
}

impl BloomRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        intensity: f32,
        threshold: f32,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let blur_renderer = ChainBlurRenderer::new(
            device,
            name.map(|n| format!("{}_blur_renderer", n))
                .as_ref()
                .map(|n| &**n),
        )?;
        Self::from_blur(encoder, device, intensity, threshold, blur_renderer, name)
    }

    pub fn from_blur(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        intensity: f32,
        threshold: f32,
        blur_renderer: ChainBlurRenderer,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let params = BloomParams::new(intensity, threshold, 0.7);

        let n = name.as_ref().map(|n| format!("{}_uniform", n));
        let uniform = gfx::Uniform::new(encoder, device, params, n.as_ref().map(|n| &**n))?;

        let vert_spv = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let prefilter_spv =
            gpu::include_spirv!("../../../shaders/cone/postprocess/bloom_prefilter.frag.spv");

        let n = name.as_ref().map(|n| format!("{}_prefilter_renderer", n));
        let prefilter_pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&prefilter_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            n.as_ref().map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        Ok(Self {
            prefilter_pipeline,
            prefiltered: Arc::default(),
            blur_renderer,
            uniform,
            name: name.map(|n| n.to_string()),
            intensity,
        })
    }

    pub fn pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        iterations: usize,
    ) -> Result<(), gpu::Error> {
        let mut prefiltered_map = self.prefiltered.lock().unwrap();
        if prefiltered_map.get(&buffer.id).is_none() {
            let filtered_texture = gfx::GTexture2D::new(
                device,
                buffer.width,
                buffer.height,
                gpu::Samples::S1,
                gpu::TextureUsage::COLOR_OUTPUT | gpu::TextureUsage::SAMPLED,
                1,
                buffer.get("output").unwrap().format(),
                self.name
                    .as_ref()
                    .map(|n| format!("{}_filtered_texture_{:?}", n, buffer.id()))
                    .as_ref()
                    .map(|n| &**n),
            )?;
            let b = match self
                .prefilter_pipeline
                .bundle()
                .unwrap()
                .set_resource("u_color", buffer.get("output").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_data", &self.uniform)
                .unwrap()
                .build(device)
            {
                Ok(g) => g,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            prefiltered_map.insert(buffer.id, (filtered_texture, b));
        }

        let (filtered_texture, prefilter_bundle) = prefiltered_map.get(&buffer.id).unwrap().clone();

        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(filtered_texture.view.clone()),
                    gpu::ClearValue::ColorFloat([1.0; 4]),
                ),
                load: gpu::LoadOp::DontCare,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.prefilter_pipeline,
        )?;

        pass.set_bundle_owned(prefilter_bundle);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        self.blur_renderer.pass(
            encoder,
            device,
            &filtered_texture.view,
            &buffer.get("output").unwrap().view,
            iterations,
            1.0,
            false,
        )?;

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&self) {
        // self.blur_bundles.clear();
        // self.blur_targets.clear();
        // self.prefilter_bundles.clear();
        self.blur_renderer.clean();
        self.prefiltered.lock().unwrap().clear();
    }
}
