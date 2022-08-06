//! SMAA: (Enhanced) Subpixel Morphological Antialiasing
//! 
//! See the reference implementation for more info <https://github.com/iryoku/smaa>

use gfx::prelude::*;

use super::smaa_area;
use super::smaa_search;
use super::DisplayFlags;

use std::borrow::Cow;

/// Describes how SMAA (Subpixel morphological antialiasing) should be performed
pub struct SMAAState {
    /// How edges are detected defaults to Luma
    pub edge: SMAAEdgeMethod,
    /// How good the antialiasing is defaults to Medium
    pub quality: SMAAQuality,
}

impl SMAAState {
    pub const LOW: Self = Self {
        edge: SMAAEdgeMethod::Luma,
        quality: SMAAQuality::Low,
    };

    pub const MEDIUM: Self = Self {
        edge: SMAAEdgeMethod::Luma,
        quality: SMAAQuality::Medium,
    };

    pub const HIGH: Self = Self {
        edge: SMAAEdgeMethod::Luma,
        quality: SMAAQuality::High,
    };

    pub const ULTRA: Self = Self {
        edge: SMAAEdgeMethod::Luma,
        quality: SMAAQuality::Ultra,
    };

    pub fn edge_detect_vert(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!(
                "../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/edge_detect.vert.spv"
            ),
            SMAAQuality::Medium => gpu::include_spirv!(
                "../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/edge_detect.vert.spv"
            ),
            SMAAQuality::High => gpu::include_spirv!(
                "../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/edge_detect.vert.spv"
            ),
            SMAAQuality::Ultra => gpu::include_spirv!(
                "../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/edge_detect.vert.spv"
            ),
        }
    }

    pub fn edge_detect_frag(&self) -> Cow<'static, [u32]> {
        match self.edge {
            SMAAEdgeMethod::Depth(_) => self.edge_detect_depth_frag(),
            SMAAEdgeMethod::Luma => self.edge_detect_luma_frag(),
            SMAAEdgeMethod::Color => self.edge_detect_color_frag(),
        }
    }

    pub fn edge_detect_depth_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/depth_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/depth_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/depth_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/depth_edge_detect.frag.spv"),
        }
    }

    pub fn edge_detect_luma_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/luma_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/luma_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/luma_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/luma_edge_detect.frag.spv"),
        }
    }

    pub fn edge_detect_color_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/color_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/color_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/color_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/color_edge_detect.frag.spv"),
        }
    }

    pub fn blend_weight_vert(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/blending_weight.vert.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/blending_weight.vert.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/blending_weight.vert.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/blending_weight.vert.spv"),
        }
    }

    pub fn blend_weight_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/blending_weight.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/blending_weight.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/blending_weight.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/blending_weight.frag.spv"),
        }
    }

    pub fn neighborhood_blend_vert(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/neighborhood_blending.vert.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending.vert.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/neighborhood_blending.vert.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/neighborhood_blending.vert.spv"),
        }
    }

    pub fn neighborhood_blend_clip_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/neighborhood_blending_clip.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending_clip.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/neighborhood_blending_clip.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/neighborhood_blending_clip.frag.spv"),
        }
    }

    pub fn neighborhood_blend_reinhard_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/neighborhood_blending_reinhard.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending_reinhard.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/neighborhood_blending_reinhard.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/neighborhood_blending_reinhard.frag.spv"),
        }
    }

    pub fn neighborhood_blend_aces_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_LOW/neighborhood_blending_aces.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending_aces.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_HIGH/neighborhood_blending_aces.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/cone/postprocess/smaa/SMAA_PRESET_ULTRA/neighborhood_blending_aces.frag.spv"),
        }
    }
}

impl Default for SMAAState {
    fn default() -> Self {
        Self {
            edge: SMAAEdgeMethod::Luma,
            quality: SMAAQuality::Medium,
        }
    }
}

/// Methods for how edges are detected
pub enum SMAAEdgeMethod {
    /// Use depth infomation to detect edges
    /// This is the least accurate but fastest and will miss chroma only aliasing
    Depth(gpu::TextureView),
    /// Use color intensity to detect edges
    /// This is a good default choice and can remove more aliasing than depth only
    Luma,
    /// Use color to detect edges
    /// This is the slowest method but should also remove the most aliasing
    Color,
}

/// Preset quality level for smaa
///
/// ## SMAA_THRESHOLD
/// specifies the threshold or sensitivity to edges.
/// Lowering this value you will be able to detect more edges at the expense of
/// performance.
///
/// Range: [0, 0.5]
///   0.1 is a reasonable value, and allows to catch most visible edges.
///   0.05 is a rather overkill value, that allows to catch 'em all.
///
///   If temporal supersampling is used, 0.2 could be a reasonable value, as low
///   contrast edges are properly filtered by just 2x.
///
///
/// ## SMAA_MAX_SEARCH_STEPS
/// specifies the maximum steps performed in the
/// horizontal/vertical pattern searches, at each side of the pixel.
///
/// In number of pixels, it's actually the double. So the maximum line length
/// perfectly handled by, for example 16, is 64 (by perfectly, we meant that
/// longer lines won't look as good, but still antialiased).
///
/// Range: [0, 112]
///
/// ## SMAA_MAX_SEARCH_STEPS_DIAG
/// specifies the maximum steps performed in the
/// diagonal pattern searches, at each side of the pixel. In this case we jump
/// one pixel at time, instead of two.
///
/// Range: [0, 20]
///
/// On high-end machines it is cheap (between a 0.8x and 0.9x slower for 16
/// steps), but it can have a significant impact on older machines.
///
/// ## SMAA_CORNER_ROUNDING
/// specifies how much sharp corners will be rounded.
///
/// Range: [0, 100]
///
/// Define SMAA_DISABLE_CORNER_DETECTION to disable corner processing.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SMAAQuality {
    /// SMAA_THRESHOLD = 0.15
    /// SMAA_MAX_SEARCH_STEPS 4
    /// SMAA_MAX_SEARCH_STEPS_DIAG (disabled)
    /// SMAA_CORNER_ROUNDING (disabled)
    Low,
    /// SMAA_THRESHOLD 0.1
    /// SMAA_MAX_SEARCH_STEPS 8
    /// SMAA_MAX_SEARCH_STEPS_DIAG (disabled)
    /// SMAA_CORNER_ROUNDING (disabled)
    Medium,
    /// SMAA_THRESHOLD 0.1
    /// SMAA_MAX_SEARCH_STEPS 16
    /// SMAA_MAX_SEARCH_STEPS_DIAG 8
    /// SMAA_CORNER_ROUNDING 25
    High,
    /// SMAA_THRESHOLD 0.05
    /// SMAA_MAX_SEARCH_STEPS 32
    /// SMAA_MAX_SEARCH_STEPS_DIAG 16
    /// SMAA_CORNER_ROUNDING 25
    Ultra,
}

/// Implementation of SMAA (Subpixel morphological antialiasing) using the [reference](https://github.com/iryoku/smaa)
///
/// Usage overview: Render your scene into this by using the Deref to [`gpu::TextureView`]
/// and the use the resolve function to resolve the target in place
///
/// There are four modes of quality to chose between when creating this
/// - Low
/// - Medium
/// - High
/// - Ultra
/// see [`SMAAQuality`] for more infomation on what these do
pub struct SMAARenderer {
    uniform: gfx::Uniform<glam::Vec4>,

    color_target: gpu::TextureView,
    edges_target: gfx::GTexture2D,
    blend_target: gfx::GTexture2D,

    area: gfx::GTexture2D,
    search: gfx::GTexture2D,
    sampler: gpu::Sampler,

    edge_detect: gfx::ReflectedGraphics,
    edge_detect_bundle: gfx::Bundle,

    blend_weight: gfx::ReflectedGraphics,
    blend_weight_bundle: gfx::Bundle,

    neighborhood_blend_clip: Option<gfx::ReflectedGraphics>,
    neighborhood_blend_clip_bundle: Option<gfx::Bundle>,

    neighborhood_blend_reinhard: Option<gfx::ReflectedGraphics>,
    neighborhood_blend_reinhard_bundle: Option<gfx::Bundle>,

    neighborhood_blend_aces: Option<gfx::ReflectedGraphics>,
    neighborhood_blend_aces_bundle: Option<gfx::Bundle>,
}

impl SMAARenderer {
    fn create_targets(
        device: &gpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(gfx::GTexture2D, gfx::GTexture2D), gpu::Error> {
        let edges_target = gfx::GTexture2D::from_formats(
            device,
            width,
            height,
            gpu::Samples::S1,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            [
                gpu::Format::Rg8Unorm,
                gpu::Format::Rg16Unorm,
                gpu::Format::Rg16Float,
                gpu::Format::Rg32Float,
            ]
            .into_iter(),
            None,
        )?
        .unwrap();
        let blend_target = gfx::GTexture2D::from_formats(
            device,
            width,
            height,
            gpu::Samples::S1,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            [
                gpu::Format::Rgba8Unorm,
                gpu::Format::Rgba16Unorm,
                gpu::Format::Rgba16Float,
                gpu::Format::Rgba32Float,
            ]
            .into_iter(),
            None,
        )?
        .unwrap();
        Ok((edges_target, blend_target))
    }

    fn create_bundles(
        device: &gpu::Device,
        edge_detect: &gfx::ReflectedGraphics,
        uniform: &gfx::Uniform<glam::Vec4>,
        target: &gpu::TextureView,
        sampler: &gpu::Sampler,
        blend_weight: &gfx::ReflectedGraphics,
        edges_target: &gfx::GTexture2D,
        area: &gfx::GTexture2D,
        search: &gfx::GTexture2D,
    ) -> Result<(gfx::Bundle, gfx::Bundle), gpu::Error> {
        let edge_detect_bundle = edge_detect
            .bundle()
            .unwrap()
            .set_resource("u_data", uniform)
            .unwrap()
            .set_combined_texture_sampler_ref("u_tex", (target, sampler))
            .unwrap()
            .build(device)?;
        let blend_weight_bundle = blend_weight
            .bundle()
            .unwrap()
            .set_resource("u_data", uniform)
            .unwrap()
            .set_resource("u_edges", &(edges_target, sampler))
            .unwrap()
            .set_resource("u_area", &(area, sampler))
            .unwrap()
            .set_resource("u_search", &(search, sampler))
            .unwrap()
            .build(device)?;
        Ok((edge_detect_bundle, blend_weight_bundle))
    }

    pub fn new_target(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        width: u32,
        height: u32,
        format: gpu::Format,
        state: SMAAState,
        flags: DisplayFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let target = gfx::GTexture2D::new(
            device,
            width,
            height,
            gpu::Samples::S1,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            format,
            None,
        )?
        .view;
        Self::new(encoder, device, &target, state, flags, name)
    }

    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        target: &gpu::TextureView,
        state: SMAAState,
        flags: DisplayFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let width = target.extent().width;
        let height = target.extent().height;

        let uniform = gfx::Uniform::new(
            encoder,
            device,
            glam::vec4(
                1.0 / width as f32,
                1.0 / height as f32,
                width as f32,
                height as f32,
            ),
            None,
        )?;

        let (edges_target, blend_target) = Self::create_targets(device, width, height)?;

        let area = gfx::GTexture2D::from_raw_image(
            encoder,
            device,
            bytemuck::cast_slice::<u8, [u8; 2]>(smaa_area::BYTES),
            smaa_area::WIDTH as u32,
            smaa_area::HEIGHT as u32,
            gpu::TextureUsage::SAMPLED,
            1,
            name.map(|n| format!("{}_area_texture", n)),
        )?;

        let search = gfx::GTexture2D::from_raw_image(
            encoder,
            device,
            smaa_search::BYTES,
            smaa_search::WIDTH as u32,
            smaa_search::HEIGHT as u32,
            gpu::TextureUsage::SAMPLED,
            1,
            name.map(|n| format!("{}_search_texture", n)),
        )?;

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
            wrap_z: gpu::WrapMode::ClampToEdge,
            min_filter: gpu::FilterMode::Linear,
            mag_filter: gpu::FilterMode::Linear,
            name: name.map(|n| format!("{}_sampler", n)),
            ..Default::default()
        })?;

        let edge_detect = match gfx::ReflectedGraphics::from_spv(
            device,
            &state.edge_detect_vert(),
            None,
            Some(&state.edge_detect_luma_frag()),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            name.map(|n| format!("{}_edge_detect", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{:?}", e),
            },
        };

        let blend_weight = match gfx::ReflectedGraphics::from_spv(
            device,
            &state.blend_weight_vert(),
            None,
            Some(&state.blend_weight_frag()),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            name.map(|n| format!("{}_blend_weight", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{:?}", e),
            },
        };

        let (neighborhood_blend_clip, neighborhood_blend_clip_bundle) =
            if flags.contains(DisplayFlags::CLIP) {
                let g = Self::crate_clip(device, &state, name)?;
                let b = g
                    .bundle()
                    .unwrap()
                    .set_resource("u_data", &uniform)
                    .unwrap()
                    .set_combined_texture_sampler_ref("u_color", (&target, &sampler))
                    .unwrap()
                    .set_resource("u_blend", &(&blend_target, &sampler))
                    .unwrap()
                    .build(device)?;

                (Some(g), Some(b))
            } else {
                (None, None)
            };

        let (neighborhood_blend_reinhard, neighborhood_blend_reinhard_bundle) =
            if flags.contains(DisplayFlags::REINHARD) {
                let g = Self::crate_reinhard(device, &state, name)?;
                let b = g
                    .bundle()
                    .unwrap()
                    .set_resource("u_data", &uniform)
                    .unwrap()
                    .set_combined_texture_sampler_ref("u_color", (&target, &sampler))
                    .unwrap()
                    .set_resource("u_blend", &(&blend_target, &sampler))
                    .unwrap()
                    .build(device)?;

                (Some(g), Some(b))
            } else {
                (None, None)
            };

        let (neighborhood_blend_aces, neighborhood_blend_aces_bundle) =
            if flags.contains(DisplayFlags::ACES) {
                let g = Self::crate_clip(device, &state, name)?;
                let b = g
                    .bundle()
                    .unwrap()
                    .set_resource("u_data", &uniform)
                    .unwrap()
                    .set_combined_texture_sampler_ref("u_color", (&target, &sampler))
                    .unwrap()
                    .set_resource("u_blend", &(&blend_target, &sampler))
                    .unwrap()
                    .build(device)?;

                (Some(g), Some(b))
            } else {
                (None, None)
            };

        let (edge_detect_bundle, blend_weight_bundle) = Self::create_bundles(
            device,
            &edge_detect,
            &uniform,
            if let SMAAEdgeMethod::Depth(d) = &state.edge {
                d
            } else {
                target
            },
            &sampler,
            &blend_weight,
            &edges_target,
            &area,
            &search,
        )?;

        Ok(Self {
            uniform,

            color_target: target.clone(),
            edges_target,
            blend_target,

            area,
            search,
            sampler,

            edge_detect,
            edge_detect_bundle,

            blend_weight,
            blend_weight_bundle,

            neighborhood_blend_clip,
            neighborhood_blend_clip_bundle,

            neighborhood_blend_reinhard,
            neighborhood_blend_reinhard_bundle,

            neighborhood_blend_aces,
            neighborhood_blend_aces_bundle,
        })
    }

    pub fn create_neighborhood(
        device: &gpu::Device,
        vert: &[u32],
        frag: &[u32],
        name: Option<String>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        match gfx::ReflectedGraphics::from_spv(
            device,
            vert,
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
                e => unreachable!("{}", e),
            },
        }
    }

    pub fn crate_clip(
        device: &gpu::Device,
        state: &SMAAState,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = state.neighborhood_blend_vert();
        let frag = state.neighborhood_blend_clip_frag();
        Self::create_neighborhood(
            device,
            &vert,
            &frag,
            name.map(|n| format!("{}_neighborhood_blend_clip", n)),
        )
    }

    pub fn crate_reinhard(
        device: &gpu::Device,
        state: &SMAAState,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = state.neighborhood_blend_vert();
        let frag = state.neighborhood_blend_reinhard_frag();
        Self::create_neighborhood(
            device,
            &vert,
            &frag,
            name.map(|n| format!("{}_neighborhood_blend_reinhard", n)),
        )
    }

    pub fn crate_aces(
        device: &gpu::Device,
        state: &SMAAState,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = state.neighborhood_blend_vert();
        let frag = state.neighborhood_blend_aces_frag();
        Self::create_neighborhood(
            device,
            &vert,
            &frag,
            name.map(|n| format!("{}_neighborhood_blend_aces", n)),
        )
    }
}

impl SMAARenderer {
    #[inline]
    pub fn edge_detect_pass(
        &self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(self.edges_target.view.clone()),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.edge_detect,
        )?;
        pass.set_bundle_owned(&self.edge_detect_bundle);
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn blend_weight_pass(
        &self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(self.blend_target.view.clone()),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.blend_weight,
        )?;
        pass.set_bundle_owned(&self.blend_weight_bundle);
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn neighborhood_blend_clip_pass<'a>(
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
            self.neighborhood_blend_clip
                .as_ref()
                .expect("SMAARenderer missing flags"),
        )?;
        pass.set_bundle_owned(&self.neighborhood_blend_clip_bundle.clone().unwrap());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn neighborhood_blend_reinhard_pass<'a>(
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
            self.neighborhood_blend_reinhard
                .as_ref()
                .expect("SMAARenderer missing flags"),
        )?;
        pass.set_bundle_owned(&self.neighborhood_blend_reinhard_bundle.clone().unwrap());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn neighborhood_blend_aces_pass<'a>(
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
            self.neighborhood_blend_aces
                .as_ref()
                .expect("SMAARenderer missing flags"),
        )?;
        pass.set_bundle_owned(&self.neighborhood_blend_aces_bundle.clone().unwrap());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    pub fn clip<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        self.edge_detect_pass(encoder, device)?;
        self.blend_weight_pass(encoder, device)?;
        self.neighborhood_blend_clip_pass(encoder, device, target)?;
        Ok(())
    }

    pub fn reinhard<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        self.edge_detect_pass(encoder, device)?;
        self.blend_weight_pass(encoder, device)?;
        self.neighborhood_blend_reinhard_pass(encoder, device, target)?;
        Ok(())
    }

    pub fn aces<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        self.edge_detect_pass(encoder, device)?;
        self.blend_weight_pass(encoder, device)?;
        self.neighborhood_blend_aces_pass(encoder, device, target)?;
        Ok(())
    }

    /// Get a reference to the smaa's area.
    #[inline]
    pub fn area(&self) -> &gfx::GTexture2D {
        &self.area
    }

    /// Get a reference to the smaa's search.
    #[inline]
    pub fn search(&self) -> &gfx::GTexture2D {
        &self.search
    }

    /// Get a reference to the smaa's sampler.
    #[inline]
    pub fn sampler(&self) -> &gpu::Sampler {
        &self.sampler
    }

    /// Get a reference to the smaa's uniform.
    #[inline]
    pub fn uniform(&self) -> &gfx::Uniform<glam::Vec4> {
        &self.uniform
    }

    /// Get a reference to the smaa's output target.
    #[inline]
    pub fn color_target(&self) -> &gpu::TextureView {
        &self.color_target
    }

    /// Get a reference to the smaa's edges target.
    #[inline]
    pub fn edges_target(&self) -> &gfx::GTexture2D {
        &self.edges_target
    }

    /// Get a reference to the smaa's blend target.
    #[inline]
    pub fn blend_target(&self) -> &gfx::GTexture2D {
        &self.blend_target
    }

    /// Get a reference to the smaa's edge detect.
    #[inline]
    pub fn edge_detect(&self) -> &gfx::ReflectedGraphics {
        &self.edge_detect
    }

    /// Get a reference to the smaa's edge detect bundle.
    #[inline]
    pub fn edge_detect_bundle(&self) -> &gfx::Bundle {
        &self.edge_detect_bundle
    }

    /// Get a reference to the smaa's blend weight.
    #[inline]
    pub fn blend_weight(&self) -> &gfx::ReflectedGraphics {
        &self.blend_weight
    }

    /// Get a reference to the smaa's neighborhood blend.
    #[inline]
    pub fn neighborhood_blend_clip(&self) -> Option<&gfx::ReflectedGraphics> {
        self.neighborhood_blend_clip.as_ref()
    }

    /// Get a reference to the smaa's neighborhood blend bundle.
    #[inline]
    pub fn neighborhood_blend_clip_bundle(&self) -> Option<&gfx::Bundle> {
        self.neighborhood_blend_clip_bundle.as_ref()
    }

    /// Get a reference to the smaa's blend weight bundle.
    #[inline]
    pub fn blend_weight_bundle(&self) -> &gfx::Bundle {
        &self.blend_weight_bundle
    }

    /// Get a mutable reference to the s m a a's uniform.
    pub fn uniform_mut(&mut self) -> &mut gfx::Uniform<glam::Vec4> {
        &mut self.uniform
    }

    /// Get a reference to the s m a a renderer's neighborhood blend reinhard.
    #[inline]
    pub fn neighborhood_blend_reinhard(&self) -> &Option<gfx::ReflectedGraphics> {
        &self.neighborhood_blend_reinhard
    }

    /// Get a reference to the s m a a renderer's neighborhood blend reinhard bundle.
    #[inline]
    pub fn neighborhood_blend_reinhard_bundle(&self) -> &Option<gfx::Bundle> {
        &self.neighborhood_blend_reinhard_bundle
    }

    /// Get a reference to the s m a a renderer's neighborhood blend aces.
    #[inline]
    pub fn neighborhood_blend_aces(&self) -> &Option<gfx::ReflectedGraphics> {
        &self.neighborhood_blend_aces
    }

    /// Get a reference to the s m a a renderer's neighborhood blend aces bundle.
    #[inline]
    pub fn neighborhood_blend_aces_bundle(&self) -> &Option<gfx::Bundle> {
        &self.neighborhood_blend_aces_bundle
    }
}

impl std::ops::Deref for SMAARenderer {
    type Target = gpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.color_target
    }
}
