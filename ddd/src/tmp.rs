use embers_gfx as gfx;
use embers_gpu as gpu;
// use embers_mesh as mesh;

use parking_lot::RwLock;
use std::collections::HashMap;

use crate::*;

bitflags::bitflags!(
    pub struct DisplayRendererFlags: u32 {
        const CLIP         = 0b001;
        const REINHARD     = 0b010;
    }
);

pub struct DisplayRenderer {
    /// Clip the color into 0-1 range
    pub clip: Option<gfx::ReflectedGraphics>,
    pub clip_bundles: RwLock<HashMap<u64, gfx::Bundle>>,
    // Apply reinhard tonemapping to color
    // pub reinhard: Option<gfx::ReflectedGraphics>,
    // pub reinhard_bundles: RwLock<HashMap<u64, gfx::Bundle>>,
}

impl DisplayRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        flags: DisplayRendererFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Ok(Self {
            clip: if flags.contains(DisplayRendererFlags::CLIP) {
                Some(Self::create_clip(
                    &encoder.device,
                    name.map(|n| format!("{}_clip", n)),
                )?)
            } else {
                None
            },
            clip_bundles: RwLock::default(),
        })
    }

    pub fn create_pipeline(
        device: &gpu::Device,
        frag: &[u32],
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../shaders/postprocess/display.vert.spv");
        match gfx::ReflectedGraphics::new(
            device,
            &vert,
            Some(frag),
            None,
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
        let frag = gpu::include_spirv!("../shaders/postprocess/clip.frag.spv");
        Self::create_pipeline(device, &frag, name)
    }
}

impl DisplayRenderer {
    pub fn clip_bundle(
        &self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let c = self.clip_bundles.read();
        if let Some(b) = c.get(&buffer.id) {
            Ok(b.clone())
        } else {
            drop(c);
            let b = self
                .clip
                .as_ref()
                .expect("ERROR: DisplayRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_texture", buffer.get("output").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .build(device)?;
            self.clip_bundles.write().insert(buffer.id, b.clone());
            Ok(b)
        }
    }
}

impl DisplayRenderer {
    pub fn clip_ref<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        buffer: &'a GeometryBuffer,
        output: &'a gpu::TextureView,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            &[gfx::Attachment::color_ref(
                output,
                gpu::LoadOp::Clear,
                gpu::StoreOp::Store,
            )],
            &[],
            None,
            self.clip
                .as_ref()
                .expect("ERROR: DisplayRenderer missing flags"),
        )?;

        let bundle = self.clip_bundle(&pass.encoder.device, buffer)?;

        pass.set_bundle_into(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }
}

pub struct AORenderer {}

pub struct BloomRenderer {}

pub struct FogRenderer {}

pub struct MotionBlurRenderer {}
