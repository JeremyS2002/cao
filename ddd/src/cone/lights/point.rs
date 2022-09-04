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
/// This will probably be re-written whenever shadow volumes and ray-traced shadows are implemented
///
/// ## Types of passes
/// - Base pass just performs lighting calculations for the geometry so no shadows
/// - Shadow pass performs lighting caclulations with pcf shadow mapping
/// - Subsurface pass performs lighting calculations with pcf shadow mapping and pcf subsurface approximation
///
/// TODO cache sets not bundles to avoid creating duplicates
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
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }

    pub fn create_shadow(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/shadow.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }

    pub fn create_subsurface(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/subsurface.frag.spv");
        Self::create_pipeline(device, &vert, &frag, name)
    }
}

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
}

// base passes
impl PointLightRenderer {
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
        pass.push_u32("samples", samples);
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
        pass.push_u32("subsurface_samples", subsurface_samples);
        pass.push_u32("shadow_samples", shadow_samples);
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
