use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cone::*;
use crate::prelude::*;
use crate::utils::*;

pub type PointLight = gfx::Uniform<PointLightData>;
pub type PointLights = gfx::Storage<PointLightData>;

/// Describes parameters sent to the gpu for point lights
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLightData {
    /// Affects the strength of light fall off, higher numbers mean gets dark faster
    pub falloff: f32,

    /// position of the light
    pub position: glam::Vec3,

    /// color of the light
    pub color: glam::Vec3,

    /// effective radius of the light
    pub radius: f32,
}

impl PointLightData {
    pub fn new(falloff: f32, position: glam::Vec3, color: glam::Vec3, cutoff: f32) -> Self {
        let radius = if cutoff > 0.0 {
            // solve for when attenuation is less than cutoff
            let m = color.x.max(color.y.max(color.z));
            let c = 0.0 - m * (1.0 / cutoff);
            let b = 0.0;
            let a = falloff;
            (-b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
        } else {
            // avoid div 0 error and just use max radius
            std::f32::MAX
        };

        Self {
            falloff,
            position,
            color,
            radius,
        }
    }
}

unsafe impl bytemuck::Pod for PointLightData {}
unsafe impl bytemuck::Zeroable for PointLightData {}

bitflags::bitflags!(
    pub struct PointLightRendererFlags: u32 {
        const BASE                = 0b0000001;
        const SHADOW              = 0b0000010;
        const SUBSURFACE          = 0b0000100;
    }
);

/// Renders [`PointLight`] to the output of [`GeometryBuffer`] with optional
/// shadow and subsurface rendering via [`PointDepthMap`] and [`PointSubsurfaceMap`]
///
/// If you scene has few lights that all effect most pixels then this will probably be more efficient, but if your
/// scene has many lights that each only effect a small number of pixels then it will probably be more efficient to
/// pack all of your lights into one [`PointLights`] and render then with [`PointLightsRenderer`]
/// 
/// ## Types of passes
/// - Base pass just performs lighting calculations for the geometry so no shadows
/// - Shadow pass performs lighting caclulations with pcf shadow mapping
/// - Subsurface pass performs lighting calculations with pcf shadow mapping and pcf subsurface approximation
///
/// TODO cache sets not bundles to avoid creating duplicates
#[derive(Clone)]
pub struct PointLightRenderer {
    /// Pure point light calculation, acts on all pixels
    pub base: Option<gfx::ReflectedGraphics>,
    pub base_bundles: Arc<Mutex<HashMap<(u64, u64, u64), gfx::Bundle>>>,
    /// point light calculation with shadows, acts on all pixels
    pub shadow: Option<gfx::ReflectedGraphics>,
    pub shadow_bundles: Arc<Mutex<HashMap<(u64, u64, u64, u64), gfx::Bundle>>>,
    /// point light subsurface (must be used with base or shadow), acts on all pixels
    pub subsurface: Option<gfx::ReflectedGraphics>,
    pub subsurface_bundles: Arc<Mutex<HashMap<(u64, u64, u64, u64), gfx::Bundle>>>,
}

impl PointLightRenderer {
    pub fn new(
        device: &gpu::Device,
        flags: PointLightRendererFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let bfn = name.as_ref().map(|n| format!("{}_base_pipeline", n));
        let sfn = name.as_ref().map(|n| format!("{}_shadow_pipeline", n));
        let sbfn = name.as_ref().map(|n| format!("{}_subsurface_pipeline", n));

        Ok(Self {
            base: if flags.contains(PointLightRendererFlags::BASE) {
                Some(Self::create_base(device, bfn.as_ref().map(|n| &**n))?)
            } else {
                None
            },
            base_bundles: Arc::default(),
            shadow: if flags.contains(PointLightRendererFlags::SHADOW) {
                Some(Self::create_shadow(device, sfn.as_ref().map(|n| &**n))?)
            } else {
                None
            },
            shadow_bundles: Arc::default(),
            subsurface: if flags.contains(PointLightRendererFlags::SUBSURFACE) {
                Some(Self::create_subsurface(
                    device,
                    sbfn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            subsurface_bundles: Arc::default(),
        })
    }

    pub const BLEND_STATE: gpu::BlendState = gpu::BlendState::ADD;

    pub const RASTERIZER: gpu::Rasterizer = gpu::Rasterizer {
        cull_face: gpu::CullFace::None,
        front_face: gpu::FrontFace::Clockwise,
        polygon_mode: gpu::PolygonMode::Fill,
        primitive_topology: gpu::PrimitiveTopology::TriangleList,
        depth_bias_constant: 0.0,
        depth_bias_slope: 0.0,
        depth_bias: false,
        depth_clamp: false,
        line_width: 1.0,
    };

    pub fn create_pipeline(
        device: &gpu::Device,
        vert: &[u32],
        frag: &[u32],
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        match gfx::ReflectedGraphics::from_spv(
            device,
            &vert,
            None,
            Some(&frag),
            Self::RASTERIZER,
            &[Self::BLEND_STATE],
            Some(gpu::DepthStencilState {
                depth: Some(gpu::DepthState {
                    test_enable: true,
                    write_enable: false,
                    compare_op: gpu::CompareOp::Greater,
                }),
                stencil_front: None,
                stencil_back: None,
            }),
            name,
        ) {
            Ok(g) => Ok(g),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        }
    }

    pub fn create_base(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/single_base.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }

    pub fn create_shadow(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/single_shadow.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }

    pub fn create_subsurface(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/single_subsurface.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }
}

// base passes
impl PointLightRenderer {
    pub fn base_bundle(
        &self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let mut bundles = self.base_bundles.lock().unwrap();
        let key = (buffer.id, camera.buffer.id(), light.buffer.id());
        if bundles.get(&key).is_none() {
            let b = match self
                .base
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("world_pos").unwrap())
                .unwrap()
                .set_resource("u_normal", buffer.get("normal").unwrap())
                .unwrap()
                .set_resource("u_albedo", buffer.get("albedo").unwrap())
                .unwrap()
                .set_resource("u_roughness", buffer.get("roughness").unwrap())
                .unwrap()
                .set_resource("u_metallic", buffer.get("metallic").unwrap())
                .unwrap()
                .set_resource("u_subsurface", buffer.get("subsurface").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_light_data", light)
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(key, b);
        }

        Ok(bundles.get(&key).unwrap().clone())
    }

    /// Add the lights contributions to the output map of the geometry buffer including shadow and subsurface
    /// 
    /// Each light in the iterator will be drawn as a fullscreen pass under a separate draw call
    /// 
    /// strength multiplies the lights contibution per pixel
    /// clear specifies if to clear the geometry buffers output map or not
    pub fn base_pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &'a Camera,
        lights: impl IntoIterator<Item = &'a PointLight>,
        strength: f32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("output").unwrap().view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: if clear {
                    gpu::LoadOp::Clear
                } else {
                    gpu::LoadOp::Load
                },
                store: gpu::StoreOp::Store,
            }],
            &[],
            Some(gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.depth.view),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.base
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for light in lights {
            let bundle = self.base_bundle(device, buffer, camera, light)?;
            pass.set_bundle_owned(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }
}

// shadow passes
impl PointLightRenderer {
    pub fn shadow_bundle(
        &self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let mut bundles = self.shadow_bundles.lock().unwrap();
        let key = (buffer.id, camera.buffer.id(), light.buffer.id(), shadow.id);
        if bundles.get(&key).is_none() {
            let b = match self
                .shadow
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("world_pos").unwrap())
                .unwrap()
                .set_resource("u_normal", buffer.get("normal").unwrap())
                .unwrap()
                .set_resource("u_albedo", buffer.get("albedo").unwrap())
                .unwrap()
                .set_resource("u_roughness", buffer.get("roughness").unwrap())
                .unwrap()
                .set_resource("u_metallic", buffer.get("metallic").unwrap())
                .unwrap()
                .set_resource("u_subsurface", buffer.get("subsurface").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_light_data", light)
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .set_resource("u_shadow_data", &shadow.uniform)
                .unwrap()
                .set_combined_texture_sampler_ref(
                    "u_shadow_map",
                    (&shadow.texture.view, &buffer.sampler),
                )
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(key, b);
        }

        Ok(bundles.get(&key).unwrap().clone())
    }

    /// Add the lights contributions to the output map of the geometry buffer including shadow
    /// 
    /// Each light in the iterator will be drawn as a fullscreen pass under a separate draw call
    /// 
    /// strength multiplies the lights contibution per pixel
    /// shadow samples is the number of shadow map reads for calculating shadow contribution (max 64)
    /// clear specifies if to clear the geometry buffers output map or not
    pub fn shadow_pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &'a Camera,
        lights: impl IntoIterator<Item = (&'a PointLight, &'a PointDepthMap)>,
        strength: f32,
        samples: u32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("output").unwrap().view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: if clear {
                    gpu::LoadOp::Clear
                } else {
                    gpu::LoadOp::Load
                },
                store: gpu::StoreOp::Store,
            }],
            &[],
            Some(gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.depth.view),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.shadow
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_u32("samples", samples.min(64));
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for (light, shadow) in lights {
            let bundle = self.shadow_bundle(device, buffer, camera, light, shadow)?;
            pass.set_bundle_owned(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }
}

// subsurface passes
impl PointLightRenderer {
    pub fn subsurface_bundle(
        &self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
        subsurface: &PointSubsurfaceMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let mut bundles = self.subsurface_bundles.lock().unwrap();
        let key = (
            buffer.id,
            camera.buffer.id(),
            light.buffer.id(),
            subsurface.id,
        );
        if bundles.get(&key).is_none() {
            let b = match self
                .subsurface
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("world_pos").unwrap())
                .unwrap()
                .set_resource("u_normal", buffer.get("normal").unwrap())
                .unwrap()
                .set_resource("u_albedo", buffer.get("albedo").unwrap())
                .unwrap()
                .set_resource("u_roughness", buffer.get("roughness").unwrap())
                .unwrap()
                .set_resource("u_metallic", buffer.get("metallic").unwrap())
                .unwrap()
                .set_resource("u_subsurface", buffer.get("subsurface").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_light_data", light)
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .set_resource("u_shadow_data", &shadow.uniform)
                .unwrap()
                .set_resource("u_shadow_map", &(&shadow.texture, &shadow.sampler))
                .unwrap()
                .set_resource("u_subsurface_data", &subsurface.uniform)
                .unwrap()
                .set_resource("u_subsurface_map", &(&subsurface.texture, &shadow.sampler))
                .unwrap()
                .set_resource("u_subsurface_lut", &(&subsurface.lut, &shadow.sampler))
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(key, b.clone());
        }

        Ok(bundles.get(&key).unwrap().clone())
    }

    /// Add the lights contributions to the output map of the geometry buffer including shadow and subsurface
    /// 
    /// Each light in the iterator will be drawn as a fullscreen pass under a separate draw call
    /// 
    /// strength multiplies the lights contibution per pixel
    /// subsurface samples is the number of shadow map reads for calculating subsurface contribution (max 64)
    /// shadow samples is the number of shadow map reads for calculating shadow contribution (max 64)
    /// clear specifies if to clear the geometry buffers output map or not
    pub fn subsurface_pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &'a Camera,
        lights: impl IntoIterator<Item = (&'a PointLight, &'a PointDepthMap, &'a PointSubsurfaceMap)>,
        strength: f32,
        subsurface_samples: u32,
        shadow_samples: u32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("output").unwrap().view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: if clear {
                    gpu::LoadOp::Clear
                } else {
                    gpu::LoadOp::Load
                },
                store: gpu::StoreOp::Store,
            }],
            &[],
            Some(gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.depth.view),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.subsurface
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_u32("subsurface_samples", subsurface_samples.min(64));
        pass.push_u32("shadow_samples", shadow_samples.min(64));
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for (light, shadow, subsurface) in lights {
            let bundle =
                self.subsurface_bundle(device, buffer, camera, light, shadow, subsurface)?;
            pass.set_bundle_owned(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.base_bundles.lock().unwrap().clear();
        self.shadow_bundles.lock().unwrap().clear();
        self.subsurface_bundles.lock().unwrap().clear();
    }
}

/// Renders [`PointLights`] to the output of a [`GeometryBuffer`] with optional
/// shadow and subsurface rendering via [`PointDepthMaps`] and [`PointSubsurfaceMaps`]
///
/// This implements a tiled rendering approach, aach [`PointLights`] draw will be preprocessed
/// and lights will be assigned to tiles over the screen so that multiple lights can be drawn more efficiently than otherwise
/// 
/// If your scene has many lights but each pixel will only have meaningful contributions from a few lights then
/// this will be more efficient, but if your scene has few lights that all effect most pixels then it's probably more 
/// efficient to use multiple [`PointLight`]s and render them with [`PointLightRenderer`]
///
/// ## Types of passes
/// - Base pass just performs lighting calculations for the geometry so no shadows
/// - Shadow pass performs lighting caclulations with pcf shadow mapping
/// - Subsurface pass performs lighting calculations with pcf shadow mapping and pcf subsurface approximation
///
/// TODO cache sets not bundles to avoid creating duplicates
pub struct PointLightsRenderer {
    /// map from (tile_width, tile_height) to (depth_texture, length_texture)
    pub tile_map: Arc<Mutex<HashMap<(u32, u32), (gfx::GTexture2D, gfx::GTexture2D)>>>,
    /// map from (tile_width, tile_height, num_lights) to storage
    pub indices_map: Arc<Mutex<HashMap<(u32, u32, usize), gfx::Storage<u32>>>>,

    /// compute pipeline for calculating the min / max depth of each tile
    /// TODO profile combining this and tile assign into one pipeline
    pub depth_calc: gfx::ReflectedCompute,
    /// map from (depth_texture, geometry_buffer) to bundle
    pub depth_calc_bundles: Arc<Mutex<HashMap<(u64, u64), gfx::Bundle>>>,

    /// compute pipeline for assigning lights to tiles
    pub tile_assign: gfx::ReflectedCompute,
    /// map from (depth_texture, camera, light_indices, length_texture, lights) to bundle
    pub tile_assign_bundles: Arc<Mutex<HashMap<(u64, u64, u64, u64, u64), gfx::Bundle>>>,

    /// compute pipeline for adding basic lighting contributions
    pub base: gfx::ReflectedCompute,
    /// map from (geometry_buffer, camera, lights, light_indices, length_texture) to bundle
    pub base_bundles: Arc<Mutex<HashMap<(u64, u64, u64, u64, u64), gfx::Bundle>>>,

    pub name: Option<String>,
}

impl PointLightsRenderer {
    pub fn new(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let depth_calc_spv = gpu::include_spirv!("../../../shaders/cone/point_light_passes/depth_calc.comp.spv");

        let depth_calc = match gfx::ReflectedCompute::new(
            device, 
            &depth_calc_spv, 
            name.map(|n| format!("{}_depth_calc", n)).as_ref().map(|n| &**n)
        ) {
            Ok(p) => p,
            Err(e) => match e {
                gfx::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let tile_assign_spv = gpu::include_spirv!("../../../shaders/cone/point_light_passes/tile_assign.comp.spv");

        let tile_assign = match gfx::ReflectedCompute::new(
            device, 
            &tile_assign_spv, 
            name.map(|n| format!("{}_tile_assign", n)).as_ref().map(|n| &**n)
        ) {
            Ok(p) => p,
            Err(e) => match e {
                gfx::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let base_spv = gpu::include_spirv!("../../../shaders/cone/point_light_passes/tile_base.comp.spv");

        let base = match gfx::ReflectedCompute::new(
            device, 
            &base_spv, 
            name.map(|n| format!("{}_base_pass", n)).as_ref().map(|n| &**n)
        ) {
            Ok(p) => p,
            Err(e) => match e {
                gfx::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        Ok(Self {
            tile_map: Arc::default(),
            indices_map: Arc::default(),

            depth_calc,
            depth_calc_bundles: Arc::default(),

            tile_assign,
            tile_assign_bundles: Arc::default(),

            base,
            base_bundles: Arc::default(),

            name: name.map(|n| n.to_string()),
        })
    }
}

impl PointLightsRenderer {
    pub fn pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &'a Camera,
        lights: impl IntoIterator<Item = &'a PointLights>,
        strength: f32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        // let tile_size = 16u32;
        let tile_size = 16u32;

        let tile_tex_width = buffer.width.div_ceil(tile_size);
        let tile_tex_height = buffer.height.div_ceil(tile_size);
        // get / create tile map and projection matrices
        let mut tile_map = self.tile_map.lock().unwrap();
        if tile_map.get(&(tile_tex_width, tile_tex_height)).is_none() {
            let d = gfx::GTexture2D::from_formats(
                device, 
                tile_tex_width, 
                tile_tex_height, 
                gpu::Samples::S1, 
                gpu::TextureUsage::STORAGE
                    | gpu::TextureUsage::SAMPLED, 
                1, 
                gfx::alt_formats(gpu::Format::Rg32Float), 
                self.name.as_ref().map(|n| format!("{}_depth_texture_width_{}_height_{}", n, tile_tex_width, tile_tex_height)).as_ref().map(|n| &**n)
            )?.unwrap();

            let l = gfx::GTexture2D::from_formats(
                device,
                tile_tex_width,
                tile_tex_height,
                gpu::Samples::S1,
                gpu::TextureUsage::STORAGE,
                1,
                gfx::alt_formats(gpu::Format::R32Uint),
                self.name.as_ref().map(|n| format!("{}_length_texture_width_{}_height_{}", n, tile_tex_width, tile_tex_height)).as_ref().map(|n| &**n)
            )?.unwrap();
            tile_map.insert((tile_tex_width, tile_tex_height), (d, l));
        }
        let (depth_texture, length_texture) = tile_map.get(&(tile_tex_width, tile_tex_height)).unwrap();
        
        // get / create depth calc bundle
        let mut depth_calc_bundles = self.depth_calc_bundles.lock().unwrap();
        if depth_calc_bundles.get(&(depth_texture.id(), buffer.id())).is_none() {
            let b = match self.depth_calc
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("view_pos").unwrap())
                .unwrap()
                .set_texture_ref("out_depth", &depth_texture.view)
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    },
                };
            depth_calc_bundles.insert((depth_texture.id(), buffer.id()), b);
        }
        let bundle = depth_calc_bundles.get(&(depth_texture.id(), buffer.id())).unwrap();
        
        // compute pass to get min / max depth and store into texture
        let mut pass = encoder.compute_pass_reflected_ref(&self.depth_calc).unwrap();
        pass.set_bundle_owned(bundle.clone());
        pass.push_u32("width", buffer.width);
        pass.push_u32("height", buffer.height);
        pass.dispatch(tile_tex_width, tile_tex_height, 1);
        pass.finish();

        for light in lights {
            // get / create light indices storage
            let mut indices_map = self.indices_map.lock().unwrap();
            if indices_map.get(&(tile_tex_width, tile_tex_height, light.length)).is_none() {
                let data = vec![0u32; tile_tex_width as usize * tile_tex_height as usize * light.length];
                // TODO uninitialized Storage
                let storage = gfx::Storage::from_vec(
                    encoder, 
                    device, 
                    data, 
                    self.name.as_ref().map(|n| format!("{}_light_indices_{}_lights_tile_width_{}_tile_height_{}", n, light.length, tile_tex_width, tile_tex_height)).as_ref().map(|n| &**n)
                )?;
                indices_map.insert((tile_tex_width, tile_tex_height, light.length), storage);
            }

            let light_indices = indices_map.get(&(tile_tex_width, tile_tex_height, light.length)).unwrap();

            // get / create bundle for assign pipeline
            let mut assign_bundles = self.tile_assign_bundles.lock().unwrap();
            if assign_bundles.get(&(depth_texture.id(), camera.buffer.id(), light_indices.id(), length_texture.id(), light.id())).is_none() {
                let b = match self.tile_assign
                    .bundle()
                    .unwrap()
                    .set_resource("in_depth", depth_texture)
                    .unwrap()
                    .set_resource("u_camera", camera)
                    .unwrap()
                    .set_resource("u_tiles", light_indices)
                    .unwrap()
                    .set_resource("out_lengths", length_texture)
                    .unwrap()
                    .set_resource("u_lights", light)
                    .unwrap()
                    .build(device)  {
                        Ok(b) => b,
                        Err(e) => match e {
                            gfx::BundleBuildError::Gpu(e) => Err(e)?,
                            e => unreachable!("{}", e),
                        },
                    };
                assign_bundles.insert((depth_texture.id(), camera.buffer.id(), light_indices.id(), length_texture.id(), light.id()), b);
            }
            let bundle = assign_bundles.get(&(depth_texture.id(), camera.buffer.id(), light_indices.id(), length_texture.id(), light.id())).unwrap();

            // compute pass to assign lights to tiles
            let mut pass = encoder.compute_pass_reflected_ref(&self.tile_assign)?;
            pass.set_bundle_owned(bundle.clone());
            pass.dispatch(tile_tex_width, tile_tex_height, 1);
            pass.finish();

            // get / create bundle for base pipeline
            let mut base_bundles = self.base_bundles.lock().unwrap();
            if base_bundles.get(&(buffer.id, camera.buffer.id(), light.id(), light_indices.id(), length_texture.id())).is_none() {
                let b = match self.base 
                    .bundle()
                    .unwrap()
                    .set_resource("in_lengths", length_texture)
                    .unwrap()
                    .set_resource("u_tiles", light_indices)
                    .unwrap()
                    .set_resource("u_lights", light)
                    .unwrap()
                    .set_resource("u_position", buffer.get("world_pos").unwrap())
                    .unwrap()
                    .set_resource("u_normal", buffer.get("normal").unwrap())
                    .unwrap()
                    .set_resource("u_albedo", buffer.get("albedo").unwrap())
                    .unwrap()
                    .set_resource("u_roughness", buffer.get("roughness").unwrap())
                    .unwrap()
                    .set_resource("u_metallic", buffer.get("metallic").unwrap())
                    .unwrap()
                    .set_resource("u_subsurface", buffer.get("subsurface").unwrap())
                    .unwrap()
                    .set_resource("u_output", buffer.get("output").unwrap())
                    .unwrap()
                    .set_resource("u_camera", camera)
                    .unwrap()
                    .build(device)  {
                        Ok(b) => b,
                        Err(e) => match e {
                            gfx::BundleBuildError::Gpu(e) => Err(e)?,
                            e => unreachable!("{}", e),
                        }
                    };
                base_bundles.insert((buffer.id, camera.buffer.id(), light.id(), light_indices.id(), length_texture.id()), b);
            }
            let bundle = base_bundles.get(&(buffer.id, camera.buffer.id(), light.id(), light_indices.id(), length_texture.id())).unwrap();

            // compute pass to render light contributions
            let mut pass = encoder.compute_pass_reflected_ref(&self.base)?;
            pass.set_bundle_owned(bundle.clone());
            pass.push_f32("strength", strength);
            pass.push_u32("width", buffer.width);
            pass.push_u32("height", buffer.height);
            if clear {
                pass.push_i32("clear", 1);
            } else {
                pass.push_i32("clear", 0);
            }
            pass.dispatch(tile_tex_width, tile_tex_height, 1);
            pass.finish();
        }
        
        Ok(())
    }

    pub fn tmp(&self) -> gpu::TextureView {
        let map = self.tile_map.lock().unwrap();
        let t = map.iter().next().unwrap().1;
        t.0.view.clone()
    }
}
