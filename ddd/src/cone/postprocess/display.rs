
use gfx::prelude::*;

bitflags::bitflags!(
    pub struct DisplayFlags: u32 {
        /// Clip HDR images to 0-1 range any value outside will be set to 1
        const CLIP         = 0b001;
        /// Apply reinhard tonemapping
        const REINHARD     = 0b010;
        /// Apply aces tonemapping
        const ACES         = 0b100;
    }
);

pub struct DisplayRenderer {
    target: gpu::TextureView,

    clip: Option<gfx::ReflectedGraphics>,
    clip_bundle: Option<gfx::Bundle>,

    reinhard: Option<gfx::ReflectedGraphics>,
    reinhard_bundle: Option<gfx::Bundle>,

    aces: Option<gfx::ReflectedGraphics>,
    aces_bundle: Option<gfx::Bundle>,

    sampler: gpu::Sampler,
}

impl std::ops::Deref for DisplayRenderer {
    type Target = gpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl DisplayRenderer {
    pub fn new(
        device: &gpu::Device,
        target: &gpu::TextureView,
        flags: DisplayFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.map(|n| format!("{}_sampler", n)),
            ..gpu::SamplerDesc::LINEAR
        })?;
        let (clip, clip_bundle) = if flags.contains(DisplayFlags::CLIP) {
            let c = Self::create_clip(device, None)?;
            let b = c
                .bundle()
                .unwrap()
                .set_resource("u_texture", target)
                .unwrap()
                .set_resource("u_sampler", &sampler)
                .unwrap()
                .build(device)?;
            (Some(c), Some(b))
        } else {
            (None, None)
        };
        let (reinhard, reinhard_bundle) = if flags.contains(DisplayFlags::REINHARD) {
            let c = Self::create_reinhard(device, None)?;
            let b = c
                .bundle()
                .unwrap()
                .set_resource("u_texture", target)
                .unwrap()
                .set_resource("u_sampler", &sampler)
                .unwrap()
                .build(device)?;
            (Some(c), Some(b))
        } else {
            (None, None)
        };
        let (aces, aces_bundle) = if flags.contains(DisplayFlags::ACES) {
            let c = Self::create_aces(device, None)?;
            let b = c
                .bundle()
                .unwrap()
                .set_resource("u_texture", target)
                .unwrap()
                .set_resource("u_sampler", &sampler)
                .unwrap()
                .build(device)?;
            (Some(c), Some(b))
        } else {
            (None, None)
        };
        Ok(Self {
            target: target.clone(),

            clip,
            clip_bundle,

            reinhard,
            reinhard_bundle,

            aces,
            aces_bundle,

            sampler,
        })
    }

    pub fn create_pipeline(
        device: &gpu::Device,
        frag: &[u32],
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/cone/postprocess/display.vert.spv");
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
        let frag = gpu::include_spirv!("../../../shaders/cone/postprocess/clip.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }

    pub fn create_reinhard(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let frag = gpu::include_spirv!("../../../shaders/cone/postprocess/reinhard.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }

    pub fn create_aces(
        device: &gpu::Device,
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let frag = gpu::include_spirv!("../../../shaders/cone/postprocess/aces.frag.spv");
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

    /// Get a reference to the display renderer's target.
    pub fn target(&self) -> &gpu::TextureView {
        &self.target
    }

    /// Get a reference to the display renderer's clip bundle.
    pub fn clip_bundle(&self) -> &Option<gfx::Bundle> {
        &self.clip_bundle
    }
}

impl DisplayRenderer {
    pub fn clip<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            self.clip
                .as_ref()
                .expect("ERROR: DisplayRenderer missing flags"),
        )?;

        pass.set_bundle_owned(self.clip_bundle.as_ref().unwrap());
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    pub fn reinhard<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            self.reinhard
                .as_ref()
                .expect("ERROR: DisplayRenderer missing flags"),
        )?;

        pass.set_bundle_owned(self.reinhard_bundle.as_ref().unwrap());
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    pub fn aces<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[target],
            &[],
            None,
            self.aces
                .as_ref()
                .expect("ERROR: DisplayRenderer missing flags"),
        )?;

        pass.set_bundle_owned(self.aces_bundle.as_ref().unwrap());
        pass.draw(0, 3, 0, 1);

        Ok(())
    }
}
