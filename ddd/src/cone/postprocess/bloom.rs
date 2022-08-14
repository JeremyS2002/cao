
use gfx::prelude::*;

use std::collections::HashMap;
use std::borrow::Cow;

use crate::cone::GeometryBuffer;

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
pub struct BloomRenderer {
    pub prefilter_pipeline: gfx::ReflectedGraphics,
    pub prefilter_bundles: HashMap<u64, gfx::Bundle>,
    pub blur_pipeline: gfx::ReflectedGraphics,
    pub uniform: gfx::Uniform<BloomParams>,
    pub iterations: usize,
    pub blur_targets: HashMap<u64, Vec<gfx::GTexture2D>>,
    pub blur_bundles: HashMap<u64, Vec<gfx::Bundle>>,
    pub intensity: f32,
    pub name: Option<String>,
}

impl BloomRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        mut iterations: usize,
        intensity: f32,
        threshold: f32,
        // buffer: &GeometryBuffer,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        iterations = iterations.max(2);

        let params = BloomParams::new(intensity, threshold, 0.7);

        let uniform = gfx::Uniform::new(
            encoder,
            device,
            params,
            name.as_ref().map(|n| format!("{}_uniform", n)),
        )?;

        let [prefilter_pipeline, blur_pipeline] = Self::pipelines(device, name.as_ref())?;

        Ok(Self {
            prefilter_pipeline,
            prefilter_bundles: HashMap::new(),
            blur_pipeline,
            blur_targets: HashMap::new(),
            blur_bundles: HashMap::new(),
            uniform,
            iterations,
            // targets,
            name,
            intensity,
        })
    }

    pub fn pipelines(device: &gpu::Device, name: Option<&String>) -> Result<[gfx::ReflectedGraphics; 2], gpu::Error> {
        let vert_spv = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let prefilter_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/bloom_prefilter.frag.spv");
        let blur_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/bloom_blur.frag.spv");

        let prefilter_pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&prefilter_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            name.as_ref().map(|n| format!("{}_prefilter_renderer", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            }
        };

        let blur_pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&blur_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::ADD],
            None,
            name.as_ref().map(|n| format!("{}_blur_renderer", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            }
        };

        Ok([prefilter_pipeline, blur_pipeline])
    }

    pub fn bloom_pass<'a>(
        &'a mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
    ) -> Result<(), gpu::Error> {
        if self.blur_targets.get(&buffer.id).is_none() {
            let mut blur_targets = Vec::new();
            
            for i in 0..self.iterations {
                let w = buffer.width >> (i + 1);
                let h = buffer.height >> (i + 1);
                if w < 2 || h < 2 { break }

                let t = gfx::GTexture2D::from_formats(
                    device, 
                    w, 
                    h, 
                    gpu::Samples::S1, 
                    gpu::TextureUsage::SAMPLED
                        | gpu::TextureUsage::COLOR_OUTPUT, 
                    1, 
                    gfx::alt_formats(gpu::Format::Rgba16Float), 
                    self.name.as_ref().map(|n| format!("{}_bloom_target_{}", n, i))
                )?.unwrap();

                blur_targets.push(t);
            }

            self.blur_targets.insert(buffer.id, blur_targets);
        }

        let blur_targets = self.blur_targets.get(&buffer.id).unwrap();

        let first = &blur_targets.get(0).unwrap().view;
        
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device, 
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(first),
                    gpu::ClearValue::ColorFloat([1.0; 4]),
                ),
                load: gpu::LoadOp::DontCare,
                store: gpu::StoreOp::Store,
            }], 
            &[],
            None,
            &self.prefilter_pipeline,
        )?;

        let prefilter_bundle = if let Some(b) = self.prefilter_bundles.get(&buffer.id) {
            b.clone()
        } else {
            let b = match self.prefilter_pipeline.bundle().unwrap()
                .set_resource("u_color", buffer.get("output").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_data", &self.uniform)
                .unwrap()
                .build(device) {
                    Ok(g) => g,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };
            self.prefilter_bundles.insert(buffer.id, b.clone());
            b
        };
            
        pass.set_bundle_into(prefilter_bundle);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        let blur_bundles = {
            if self.blur_bundles.get(&buffer.id).is_none() {
                let b = blur_targets.iter()
                    .map(|t| {
                        match self.blur_pipeline.bundle().unwrap()
                            .set_resource("u_color", t)
                            .unwrap()
                            .set_resource("u_sampler", &buffer.sampler)
                            .unwrap()
                            .build(device) {
                                Ok(g) => Ok(g),
                                Err(e) => match e {
                                    gfx::BundleBuildError::Gpu(e) => Err(e),
                                    e => unreachable!("{}", e)
                                }
                            }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.blur_bundles.insert(buffer.id, b);
            }
            self.blur_bundles.get(&buffer.id).unwrap()
        };

        let len = self.iterations.min(blur_targets.len());
        for i in 1..len {
            let target = blur_targets.get(i).unwrap();
            let mut pass = encoder.graphics_pass_reflected::<()>(
                device, 
                &[gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&target.view),
                        gpu::ClearValue::ColorFloat([0.0; 4]),
                    ),
                    load: gpu::LoadOp::Clear,
                    store: gpu::StoreOp::Store,
                }], 
                &[], 
                None, 
                &self.blur_pipeline
            )?;
            let (width, height) = (target.dimension.0, target.dimension.1);
            let texel_size = 1.0 / glam::vec2(width as f32, height as f32);
            pass.push_vec2("texel_size", texel_size.into());
            pass.push_f32("intensity", 1.0);

            let bundle = blur_bundles.get(i - 1).unwrap();
            pass.set_bundle_ref(bundle);
            pass.draw(0, 3, 0, 1);
            pass.finish();
        }

        for i in (0..(len-1)).rev() {
            let target = blur_targets.get(i).unwrap();
            let mut pass = encoder.graphics_pass_reflected::<()>(
                device, 
                &[gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&target.view), 
                        gpu::ClearValue::ColorFloat([0.0; 4])
                    ),
                    load: gpu::LoadOp::Load,
                    store: gpu::StoreOp::Store,
                }], 
                &[], 
                None, 
                &self.blur_pipeline
            )?;
            let (width, height) = (target.dimension.0, target.dimension.1);
            let texel_size = 1.0 / glam::vec2(width as f32, height as f32);
            pass.push_vec2("texel_size", texel_size.into());
            pass.push_f32("intensity", 1.0);

            let bundle = blur_bundles.get(i + 1).unwrap();
            pass.set_bundle_ref(bundle);
            pass.draw(0, 3, 0, 1);
            pass.finish();
        }
        
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device, 
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("output").unwrap().view), 
                    gpu::ClearValue::ColorFloat([0.0; 4])
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }], 
            &[], 
            None, 
            &self.blur_pipeline,
        )?;
        let texel_size = 1.0 / glam::vec2(buffer.width as f32, buffer.height as f32);
        pass.push_vec2("texel_size", texel_size.into());
        pass.push_f32("intensity", self.intensity);
        let bundle = blur_bundles.get(0).unwrap();
        pass.set_bundle_ref(bundle);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.blur_bundles.clear();
        self.blur_targets.clear();
        self.prefilter_bundles.clear();
    }
}