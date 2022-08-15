//! SMAA: (Enhanced) Subpixel Morphological Antialiasing
//! 
//! See the reference implementation for more info <https://github.com/iryoku/smaa>

pub(crate) mod smaa_area;
pub(crate) mod smaa_search;

use gfx::prelude::*;

use std::collections::HashMap;
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
                "../../../shaders/smaa/SMAA_PRESET_LOW/edge_detect.vert.spv"
            ),
            SMAAQuality::Medium => gpu::include_spirv!(
                "../../../shaders/smaa/SMAA_PRESET_MEDIUM/edge_detect.vert.spv"
            ),
            SMAAQuality::High => gpu::include_spirv!(
                "../../../shaders/smaa/SMAA_PRESET_HIGH/edge_detect.vert.spv"
            ),
            SMAAQuality::Ultra => gpu::include_spirv!(
                "../../../shaders/smaa/SMAA_PRESET_ULTRA/edge_detect.vert.spv"
            ),
        }
    }

    pub fn edge_detect_frag(&self) -> Cow<'static, [u32]> {
        match self.edge {
            SMAAEdgeMethod::Depth => self.edge_detect_depth_frag(),
            SMAAEdgeMethod::Luma => self.edge_detect_luma_frag(),
            SMAAEdgeMethod::Color => self.edge_detect_color_frag(),
        }
    }

    pub fn edge_detect_depth_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/depth_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/depth_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/depth_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/depth_edge_detect.frag.spv"),
        }
    }

    pub fn edge_detect_luma_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/luma_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/luma_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/luma_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/luma_edge_detect.frag.spv"),
        }
    }

    pub fn edge_detect_color_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/color_edge_detect.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/color_edge_detect.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/color_edge_detect.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/color_edge_detect.frag.spv"),
        }
    }

    pub fn blend_weight_vert(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/blending_weight.vert.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/blending_weight.vert.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/blending_weight.vert.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/blending_weight.vert.spv"),
        }
    }

    pub fn blend_weight_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/blending_weight.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/blending_weight.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/blending_weight.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/blending_weight.frag.spv"),
        }
    }

    pub fn neighborhood_blend_vert(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/neighborhood_blending.vert.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending.vert.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/neighborhood_blending.vert.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/neighborhood_blending.vert.spv"),
        }
    }

    pub fn neighborhood_blend_frag(&self) -> Cow<'static, [u32]> {
        match self.quality {
            SMAAQuality::Low => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_LOW/neighborhood_blending.frag.spv"),
            SMAAQuality::Medium => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_MEDIUM/neighborhood_blending.frag.spv"),
            SMAAQuality::High => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_HIGH/neighborhood_blending.frag.spv"),
            SMAAQuality::Ultra => gpu::include_spirv!("../../../shaders/smaa/SMAA_PRESET_ULTRA/neighborhood_blending.frag.spv"),
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
    Depth,
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
    state: SMAAState,
    pub edges_targets: HashMap<(u32, u32), gfx::GTexture2D>,
    pub blend_targets: HashMap<(u32, u32), gfx::GTexture2D>,

    pub area: gfx::GTexture2D,
    pub search: gfx::GTexture2D,
    pub sampler: gpu::Sampler,
    
    pub edge_detect: gfx::ReflectedGraphics,
    pub edge_detect_bundles: HashMap<u64, gfx::Bundle>,

    pub blend_weight: gfx::ReflectedGraphics,
    pub blend_weight_bundles: HashMap<(u32, u32), gfx::Bundle>,

    pub neighborhood_blend: gfx::ReflectedGraphics,
    pub neighborhood_blend_bundles: HashMap<u64, gfx::Bundle>,
}

impl SMAARenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        state: SMAAState,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let n = name.as_ref().map(|n| format!("{}_area_texture", n));
        let area = gfx::GTexture2D::from_raw_image(
            encoder,
            device,
            bytemuck::cast_slice::<u8, [u8; 2]>(smaa_area::BYTES),
            smaa_area::WIDTH as u32,
            smaa_area::HEIGHT as u32,
            gpu::TextureUsage::SAMPLED,
            1,
            n.as_ref().map(|n| &**n),
        )?;

        let n = name.as_ref().map(|n| format!("{}_search_texture", n));
        let search = gfx::GTexture2D::from_raw_image(
            encoder,
            device,
            smaa_search::BYTES,
            smaa_search::WIDTH as u32,
            smaa_search::HEIGHT as u32,
            gpu::TextureUsage::SAMPLED,
            1,
            n.as_ref().map(|n| &**n),
        )?;

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
            wrap_z: gpu::WrapMode::ClampToEdge,
            min_filter: gpu::FilterMode::Linear,
            mag_filter: gpu::FilterMode::Linear,
            name: name.as_ref().map(|n| format!("{}_sampler", n)),
            ..Default::default()
        })?;

        let n = name.as_ref().map(|n| format!("{}_edge_detect", n));
        let edge_detect = match gfx::ReflectedGraphics::from_spv(
            device,
            &state.edge_detect_vert(),
            None,
            Some(&state.edge_detect_luma_frag()),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            n.as_ref().map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{:?}", e),
            },
        };

        let n = name.as_ref().map(|n| format!("{}_blend_weight", n));
        let blend_weight = match gfx::ReflectedGraphics::from_spv(
            device,
            &state.blend_weight_vert(),
            None,
            Some(&state.blend_weight_frag()),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            n.as_ref().map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{:?}", e),
            },
        };

        let n = name.as_ref().map(|n| format!("{}_neighborhood_blend", n));
        let neighborhood_blend = Self::create_neighborhood(
            device, 
            &state, 
            n.as_ref().map(|n| &**n),
        )?;

        Ok(Self {
            state, 

            edges_targets: HashMap::new(),
            blend_targets: HashMap::new(),

            area,
            search,
            sampler,

            edge_detect,
            edge_detect_bundles: HashMap::new(),

            blend_weight,
            blend_weight_bundles: HashMap::new(),

            neighborhood_blend,
            neighborhood_blend_bundles: HashMap::new(),
        })
    }

    pub fn create_neighborhood(
        device: &gpu::Device,
        state: &SMAAState,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = state.neighborhood_blend_vert();
        let frag = state.neighborhood_blend_frag();
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
                e => unreachable!("{}", e),
            },
        }
    }
}

impl SMAARenderer {
    pub fn edges_target(&mut self, device: &gpu::Device, width: u32, height: u32) -> Result<gpu::TextureView, gpu::Error> {
        if self.edges_targets.get(&(width, height)).is_none() {
            let t = gfx::GTexture2D::from_formats(
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
            self.edges_targets.insert((width, height), t);
        }
        Ok(self.edges_targets.get(&(width, height)).unwrap().view.clone())
    }

    fn blend_weight_target(&mut self, device: &gpu::Device, width: u32, height: u32) -> Result<gpu::TextureView, gpu::Error> {
        if self.blend_targets.get(&(width, height)).is_none() {
            let t = gfx::GTexture2D::from_formats(
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
            self.blend_targets.insert((width, height), t);
        }
        Ok(self.blend_targets.get(&(width, height)).unwrap().view.clone())
    }

    #[inline]
    pub fn edge_detect_pass(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        src: &gpu::TextureView,
    ) -> Result<(), gpu::Error> {
        let width = src.extent().width;
        let height = src.extent().height;
        let target = self.edges_target(device, width, height)?;
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(target),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.edge_detect,
        )?;
        if self.edge_detect_bundles.get(&src.id()).is_none() {
            let b = match self.edge_detect
                .bundle()
                .unwrap()
                .set_combined_texture_sampler_ref("u_tex", (src, &self.sampler))
                .unwrap()
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };
            self.edge_detect_bundles.insert(src.id(), b);
        }
        let bundle = self.edge_detect_bundles.get(&src.id()).unwrap().clone();
        pass.set_bundle_into(bundle);
        let rt = glam::vec4(1.0 / width as f32, 1.0 / height as f32, width as f32, height as f32);
        pass.push_vec4("rt", rt.into());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn blend_weight_pass(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), gpu::Error> {
        let target = self.blend_weight_target(device, width, height)?;
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(target),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.blend_weight,
        )?;
        if self.blend_weight_bundles.get(&(width, height)).is_none() {
            let edges_target = self.edges_target(device, width, height)?;
            let b = match self.blend_weight
                .bundle()
                .unwrap()
                .set_combined_texture_sampler_ref("u_edges", (&edges_target, &self.sampler))
                .unwrap()
                .set_resource("u_area", &(&self.area, &self.sampler))
                .unwrap()
                .set_resource("u_search", &(&self.search, &self.sampler))
                .unwrap()
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };
            self.blend_weight_bundles.insert((width, height), b);
        }
        let bundle = self.blend_weight_bundles.get(&(width, height)).unwrap().clone();
        pass.set_bundle_into(bundle);
        let rt = glam::vec4(1.0 / width as f32, 1.0 / height as f32, width as f32, height as f32);
        pass.push_vec4("rt", rt.into());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    #[inline]
    pub fn neighborhood_blend_pass<'a>(
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
            &self.neighborhood_blend
        )?;

        if self.neighborhood_blend_bundles.get(&src.id()).is_none() {
            let width = src.extent().width;
            let height = src.extent().height;
            let blend_target = {
                if self.blend_targets.get(&(width, height)).is_none() {
                    let t = gfx::GTexture2D::from_formats(
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
                    self.blend_targets.insert((width, height), t);
                }
                self.blend_targets.get(&(width, height)).unwrap().view.clone()
            };

            let b = match self.neighborhood_blend
                .bundle()
                .unwrap()
                .set_combined_texture_sampler_ref("u_color", (&src, &self.sampler))
                .unwrap()
                .set_combined_texture_sampler_ref("u_blend", (&blend_target, &self.sampler))
                .unwrap()      
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.neighborhood_blend_bundles.insert(src.id(), b);
        }
        let bundle = self.neighborhood_blend_bundles.get(&src.id()).unwrap();
        pass.set_bundle_owned(bundle);

        let width = src.extent().width;
        let height = src.extent().height;
        let rt = glam::vec4(1.0 / width as f32, 1.0 / height as f32, width as f32, height as f32);
        pass.push_vec4("rt", rt.into());
        pass.draw(0, 3, 0, 1);
        Ok(())
    }

    /// Peform antialiasing from the src into the target
    /// 
    /// Color values will be clipped into the range of the target textures format
    /// If self was created with [`SMAAEdgeMethod::Depth`] then depth must not be [`None`]
    pub fn pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src: &gpu::TextureView,
        depth: Option<&gpu::TextureView>,
        target: gfx::Attachment<'a>,
    ) -> Result<(), gpu::Error> {
        let width = src.extent().width;
        let height = src.extent().height;
        match self.state.edge {
            SMAAEdgeMethod::Depth => self.edge_detect_pass(
                encoder, 
                device, 
                depth.expect("Depth must not be None for SMAARenderer::pass if the renderer was created with EdgeMethod::Depth")
            )?,
            _ => self.edge_detect_pass(encoder, device, src)?,
        }
        self.blend_weight_pass(encoder, device, width, height)?;
        self.neighborhood_blend_pass(encoder, device, src, target)?;
        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.edge_detect_bundles.clear();
        self.blend_weight_bundles.clear();
        self.neighborhood_blend_bundles.clear();
    }
}
