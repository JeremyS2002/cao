
use gfx::prelude::*;

use std::collections::HashMap;

use crate::utils::DisplayFlags;

pub struct DisplayRenderer {
    clip: Option<gfx::ReflectedGraphics>,
    clip_bundles: HashMap<u64, gfx::Bundle>,

    reinhard: Option<gfx::ReflectedGraphics>,
    reinhard_bundles: HashMap<u64, gfx::Bundle>,

    aces: Option<gfx::ReflectedGraphics>,
    aces_bundles: HashMap<u64, gfx::Bundle>,

    sampler: gpu::Sampler,
}

impl DisplayRenderer {
    pub fn new(
        device: &gpu::Device,
        flags: DisplayFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.map(|n| format!("{}_sampler", n)),
            ..gpu::SamplerDesc::LINEAR
        })?;
        let clip = if flags.contains(DisplayFlags::CLIP) {
            let c = Self::create_clip(device, None)?;
            Some(c)
        } else {
            None
        };
        let reinhard = if flags.contains(DisplayFlags::REINHARD) {
            let c = Self::create_reinhard(device, None)?;
            Some(c)
        } else {
            None
        };
        let aces = if flags.contains(DisplayFlags::ACES) {
            let c = Self::create_aces(device, None)?;
            Some(c)
        } else {
            None
        };
        Ok(Self {
            clip,
            clip_bundles: HashMap::new(),

            reinhard,
            reinhard_bundles: HashMap::new(),

            aces,
            aces_bundles: HashMap::new(),

            sampler,
        })
    }

    pub fn create_pipeline(
        device: &gpu::Device,
        frag: &[u32],
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../shaders/cone/postprocess/display.vert.spv");
        match gfx::ReflectedGraphics::from_spv(
            device,
            &vert,
            None,
            Some(frag),
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

    pub fn create_clip(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let frag = gpu::include_spirv!("../../shaders/cone/postprocess/clip.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }

    pub fn create_reinhard(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let frag = gpu::include_spirv!("../../shaders/cone/postprocess/reinhard.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }

    pub fn create_aces(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let frag = gpu::include_spirv!("../../shaders/cone/postprocess/aces.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }

    /// Get a reference to the display renderer's sampler.
    pub fn sampler(&self) -> &gpu::Sampler {
        &self.sampler
    }

    /// Get a reference to the display renderer's clip.
    pub fn clip_renderer(&self) -> &Option<gfx::ReflectedGraphics> {
        &self.clip
    }
}

impl DisplayRenderer {
    pub fn clip<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let c = self.clip
            .as_ref()
            .expect("ERROR: DisplayRenderer missing flags");
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            c,
        )?;

        if self.clip_bundles.get(&src.id()).is_none() {
            let b = match c
                .bundle()
                .unwrap()
                .set_resource("u_texture", src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.clip_bundles.insert(src.id(), b);
        }
        let bundle = self.clip_bundles.get(&src.id()).unwrap();
        pass.set_bundle_owned(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    pub fn reinhard<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let c = self.reinhard
            .as_ref()
            .expect("ERROR: DisplayRenderer missing flags");
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            c,
        )?;

        if self.reinhard_bundles.get(&src.id()).is_none() {
            let b = match c
                .bundle()
                .unwrap()
                .set_resource("u_texture", src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.reinhard_bundles.insert(src.id(), b);
        }
        let bundle = self.reinhard_bundles.get(&src.id()).unwrap();
        pass.set_bundle_owned(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    pub fn aces<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let c = self.aces
            .as_ref()
            .expect("ERROR: DisplayRenderer missing flags");
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            c,
        )?;

        if self.aces_bundles.get(&src.id()).is_none() {
            let b = match c
                .bundle()
                .unwrap()
                .set_resource("u_texture", src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.aces_bundles.insert(src.id(), b);
        }
        let bundle = self.aces_bundles.get(&src.id()).unwrap();
        pass.set_bundle_owned(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }
}
