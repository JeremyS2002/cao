
use gfx::prelude::*;

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
    pub prefilter_bundle: gfx::Bundle,
    pub blur_pipeline: gfx::ReflectedGraphics,
    pub uniform: gfx::Uniform<BloomParams>,
    pub targets: Vec<gfx::GTexture2D>,
    pub blur_bundles: Vec<gfx::Bundle>,
    pub intensity: f32,
}

impl BloomRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        mut iterations: usize,
        intensity: f32,
        threshold: f32,
        buffer: &GeometryBuffer,
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

        let targets = Self::targets(
            device, 
            buffer.width,
            buffer.height,
            iterations, 
            name.as_ref()
        )?;

        let [prefilter_pipeline, blur_pipeline] = Self::pipelines(device, name)?;

        let prefilter_bundle = match prefilter_pipeline.bundle().unwrap()
            .set_resource("u_color", buffer.get("output").unwrap())
            .unwrap()
            .set_resource("u_sampler", &buffer.sampler)
            .unwrap()
            .set_resource("u_data", &uniform)
            .unwrap()
            .build(device) {
                Ok(g) => g,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };

        let blur_bundles = targets.iter()
            .map(|t| {
                match blur_pipeline.bundle().unwrap()
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

        Ok(Self {
            prefilter_pipeline,
            prefilter_bundle,
            blur_pipeline,
            blur_bundles,
            uniform,
            targets,
            intensity,
        })
    }

    pub fn targets(device: &gpu::Device, width: u32, height: u32, iterations: usize, name: Option<&String>) -> Result<Vec<gfx::GTexture2D>, gpu::Error> {
        let mut targets = Vec::new();

        for i in 0..iterations {
            let w = width >> (i + 1);
            let h = height >> (i + 1);
            if width < 2 || height < 2 { break }

            let t = gfx::GTexture2D::from_formats(
                device, 
                w, h, 
                gpu::Samples::S1, 
                gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COLOR_OUTPUT, 
                1, 
                gfx::alt_formats(gpu::Format::Rgba32Float), 
                name.map(|n| format!("{}_target_{}", n, i))
            )?.unwrap();

            targets.push(t);
        }

        Ok(targets)
    }

    pub fn pipelines(device: &gpu::Device, name: Option<String>) -> Result<[gfx::ReflectedGraphics; 2], gpu::Error> {
        let vert_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/display.vert.spv");
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
        let first = &self.targets.get(0).unwrap().view;
        
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

        pass.set_bundle_ref(&self.prefilter_bundle);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        for i in 1..self.targets.len() {
            let target = self.targets.get(i).unwrap();
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

            let bundle = self.blur_bundles.get(i - 1).unwrap();
            pass.set_bundle_ref(bundle);
            pass.draw(0, 3, 0, 1);
            pass.finish();
        }

        for i in (0..(self.targets.len()-1)).rev() {
            let target = self.targets.get(i).unwrap();
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

            let bundle = self.blur_bundles.get(i + 1).unwrap();
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
        let bundle = self.blur_bundles.get(0).unwrap();
        pass.set_bundle_ref(bundle);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        Ok(())
    }
}