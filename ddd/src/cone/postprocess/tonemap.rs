use gfx::prelude::*;

use std::collections::HashMap;

/// Describes the curve in which linear colors are transformed by
/// 
/// source <https://www.slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting> slide 142
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlobalToneMapParams {
    pub shoulder: f32,
    pub linear_strength: f32,
    pub linear_angle: f32,
    pub toe_strength: f32,
    pub toe_numerator: f32,
    pub toe_denominator: f32,
    pub linear_white: f32,
}

impl GlobalToneMapParams {
    /// Parameters to match how film exposes
    /// source <https://www.slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting> slide 143
    pub const FILMIC: Self = Self { 
        shoulder: 0.22f32, 
        linear_strength: 0.3f32, 
        linear_angle: 0.1f32, 
        toe_strength: 0.2f32, 
        toe_numerator: 0.01f32, 
        toe_denominator: 0.3f32,
        linear_white: 11.2f32,
    };
}

impl std::default::Default for GlobalToneMapParams {
    fn default() -> Self {
        Self { 
            shoulder: 0.22f32.powf(1.0f32 / 2.2f32), 
            linear_strength: 0.3f32.powf(1.0f32 / 2.2f32), 
            linear_angle: 0.1f32.powf(1.0f32 / 2.2f32), 
            toe_strength: 0.2f32.powf(1.0f32 / 2.2f32), 
            toe_numerator: 0.01f32.powf(1.0f32 / 2.2f32), 
            toe_denominator: 0.3f32.powf(1.0f32 / 2.2f32),
            linear_white: 11.2f32,
        }
    }
}

unsafe impl bytemuck::Pod for GlobalToneMapParams { }
unsafe impl bytemuck::Zeroable for GlobalToneMapParams { }

pub struct GlobalToneMapRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub bundles: HashMap<u64, gfx::Bundle>,
    pub params: gfx::Uniform<GlobalToneMapParams>,
    pub sampler: gpu::Sampler,
}

impl GlobalToneMapRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        params: GlobalToneMapParams,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.map(|n| format!("{}_sampler", n)),
            ..gpu::SamplerDesc::LINEAR
        })?;

        let params = gfx::Uniform::new(
            encoder, 
            device, 
            params, 
            name.as_ref().map(|n| format!("{}_params", n)),
        )?;

        let pipeline = Self::create_pipeline(
            device, 
            name.as_ref().map(|n| format!("{}_pipeline", n)),
        )?;

        Ok(Self {
            pipeline,
            bundles: HashMap::new(),
            params,
            sampler,
        })
    }

    pub fn create_pipeline(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/postprocess/tonemap_global.frag.spv");
        match gfx::ReflectedGraphics::from_spv(
            device,
            &vert,
            None,
            Some(&frag),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
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

impl GlobalToneMapRenderer {
    pub fn pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            &self.pipeline,
        )?;

        if self.bundles.get(&src.id()).is_none() {
            let b = match self.pipeline
                .bundle()
                .unwrap()
                .set_resource("u_texture", src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .set_resource("u", &self.params)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.bundles.insert(src.id(), b);
        }
        let bundle = self.bundles.get(&src.id()).unwrap();
        pass.set_bundle_owned(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.bundles.clear();
    }
}

/// Describes the curve in which linear colors are transformed by
/// 
/// source <https://www.slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting> slide 142
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LocalToneMapParams {
    pub shoulder: f32,
    pub linear_strength: f32,
    pub linear_angle: f32,
    pub toe_strength: f32,
    pub toe_numerator: f32,
    pub toe_denominator: f32,
    pub width: f32,
    pub height: f32,
}