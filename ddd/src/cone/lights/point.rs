use std::borrow::Cow;
use std::collections::HashMap;

use crate::cone::*;
use crate::prelude::*;
use crate::utils::*;

pub type PointLight = gfx::Uniform<PointLightData>;
pub type PointLights = gfx::Storage<PointLightData>;

/// Describes parameters sent to the gpu for point lights
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLightData {
    /// Constant factor in light intensity falloff
    pub constant: f32,
    /// Linear factor in light intensity falloff
    pub linear: f32,
    /// Quadratic factor in light intensity falloff
    pub quadratic: f32,

    /// position of the light
    pub position: glam::Vec3,

    /// color of the light
    pub color: glam::Vec3,

    /// effective radius of the light
    pub radius: f32,
}

impl PointLightData {
    pub fn new(
        constant: f32,
        linear: f32,
        quadratic: f32,
        position: glam::Vec3,
        color: glam::Vec3,
        cutoff: f32,
    ) -> Self {
        let radius = if cutoff > 0.0 {
            // solve for when attenuation is less than cutoff
            let m = color.x.max(color.y.max(color.z));
            let c = constant - m * (1.0 / cutoff);
            let b = linear;
            let a = quadratic;
            (-b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
        } else {
            // avoid div 0 error and just use max radius
            std::f32::MAX
        };

        Self {
            constant,
            linear,
            quadratic,
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
        const BASE_CLIPPED        = 0b0000000001;
        const SHADOW_CLIPPED      = 0b0000000010;
        const SUBSURFACE_CLIPPED  = 0b0000000100;
        const BASE_FULL           = 0b0000001000;
        const SHADOW_FULL         = 0b0000010000;
        const SUBSURFACE_FULL     = 0b0000100000;

        const BASE                = Self::BASE_CLIPPED.bits | Self::BASE_FULL.bits;
        const SHADOW              = Self::SHADOW_CLIPPED.bits | Self::SHADOW_FULL.bits;
        const SUBSURFACE          = Self::SUBSURFACE_CLIPPED.bits | Self::SUBSURFACE_FULL.bits;

        const CLIPPED             = Self::BASE_CLIPPED.bits | Self::SHADOW_CLIPPED.bits | Self::SUBSURFACE_CLIPPED.bits;
        const FULL                = Self::BASE_FULL.bits | Self::SHADOW_FULL.bits | Self::SUBSURFACE_FULL.bits;
    }
);

/// Renders [`PointLight`] to the output of [`GeometryBuffer`] with optional
/// shadow and subsurface rendering via [`PointDepthMap`] and [`PointSubsurfaceMap`]
///
/// This will probably be re-written whenever shadow volumes and ray-traced shadows are implemented
///
/// ## Full pass vs Clipped pass
/// - A full pass draws a full screen quad and performs lighting caclulations for the whole screen based on what is stored in the [`GeometryBuffer`]
/// - A clipped pass draws a sphere at the position of the light with the effective radius  of the light so only performs lighting calculations for the pixels that are within that radius
/// - If your lights illuminate the whole screen the all pass is more efficient however if you have many light that each illuminate a small area the clipped pass can be more efficient
///
/// ## Types of passes
/// - Base pass just performs lighting calculations for the geometry so no shadows will be no shadows
/// - Shadow pass performs lighting caclulations with pcf shadow mapping
/// - Subsurface pass performs lighting calculations with pcf shadow mapping and pcf subsurface approximation
///
/// TODO cache sets not bundles to avoid creating duplicates
pub struct PointLightRenderer {
    pub sphere: gfx::Mesh<BasicVertex>,
    /// Pure point light calculation clipped to only work on pixels < light.radius from light
    pub base_clipped: Option<gfx::ReflectedGraphics>,
    pub base_clipped_bundles: HashMap<(u64, u64, u64), gfx::Bundle>,
    /// point light calculation with shadows clipped to only work on pixels < light.radius from light
    pub shadow_clipped: Option<gfx::ReflectedGraphics>,
    pub shadow_clipped_bundles: HashMap<(u64, u64, u64, u64), gfx::Bundle>,
    /// point light subsurface (must be used with base or shadow) clipped to only work on pixels < light.radius from light
    pub subsurface_clipped: Option<gfx::ReflectedGraphics>,
    pub subsurface_clipped_bundles: HashMap<(u64, u64, u64, u64), gfx::Bundle>,
    /// Pure point light calculation, acts on all pixels
    pub base_full: Option<gfx::ReflectedGraphics>,
    pub base_full_bundles: HashMap<(u64, u64, u64), gfx::Bundle>,
    /// point light calculation with shadows, acts on all pixels
    pub shadow_full: Option<gfx::ReflectedGraphics>,
    pub shadow_full_bundles: HashMap<(u64, u64, u64, u64), gfx::Bundle>,
    /// point light subsurface (must be used with base or shadow), acts on all pixels
    pub subsurface_full: Option<gfx::ReflectedGraphics>,
    pub subsurface_full_bundles: HashMap<(u64, u64, u64, u64), gfx::Bundle>,
}

impl PointLightRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        flags: PointLightRendererFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {

        let sn = name.as_ref().map(|n| format!("{}_sphere", n));
        let bcn = name.as_ref().map(|n| format!("{}_base_clipped_pipeline", n));
        let bfn = name.as_ref().map(|n| format!("{}_base_full_pipeline", n));
        let scn = name.as_ref().map(|n| format!("{}_shadow_clipped_pipeline", n));
        let sfn = name.as_ref().map(|n| format!("{}_shadow_full_pipeline", n));
        let sbcn = name.as_ref().map(|n| format!("{}_subsurface_clipped_pipeline", n));
        let sbfn = name.as_ref().map(|n| format!("{}_subsurface_full_pipeline", n));

        Ok(Self {
            sphere: mesh::ico_sphere(
                encoder,
                device,
                3,
                sn.as_ref().map(|n| &**n),
            )?,
            base_clipped: if flags.contains(PointLightRendererFlags::BASE_CLIPPED) {
                Some(Self::create_base_clipped(
                    device,
                    bcn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            base_clipped_bundles: HashMap::new(),
            base_full: if flags.contains(PointLightRendererFlags::BASE_FULL) {
                Some(Self::create_base_full(
                    device,
                    bfn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            base_full_bundles: HashMap::new(),
            shadow_clipped: if flags.contains(PointLightRendererFlags::SHADOW_CLIPPED) {
                Some(Self::create_shadow_clipped(
                    device,
                    scn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            shadow_clipped_bundles: HashMap::new(),
            shadow_full: if flags.contains(PointLightRendererFlags::SHADOW_FULL) {
                Some(Self::create_shadow_full(
                    device,
                    sfn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            shadow_full_bundles: HashMap::new(),
            subsurface_clipped: if flags.contains(PointLightRendererFlags::SUBSURFACE_CLIPPED) {
                Some(Self::create_subsurface_clipped(
                    device,
                    sbcn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            subsurface_clipped_bundles: HashMap::new(),
            subsurface_full: if flags.contains(PointLightRendererFlags::SUBSURFACE_FULL) {
                Some(Self::create_subsurface_full(
                    device,
                    sbfn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            subsurface_full_bundles: HashMap::new(),
        })
    }

    pub const BLEND_STATE: gpu::BlendState = gpu::BlendState::ADD;

    pub const RASTERIZER: gpu::Rasterizer = gpu::Rasterizer {
        cull_face: gpu::CullFace::Back,
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
        depth: gpu::DepthState,
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
                depth: Some(depth),
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

    pub fn create_clipped_pipeline(
        device: &gpu::Device,
        vert: &[u32],
        frag: &[u32],
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        Self::create_pipeline(
            device,
            vert,
            frag,
            gpu::DepthState {
                write_enable: false,
                test_enable: true,
                compare_op: gpu::CompareOp::LessEqual,
            },
            name,
        )
    }

    pub fn create_full_pipeline(
        device: &gpu::Device,
        vert: &[u32],
        frag: &[u32],
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        Self::create_pipeline(
            device,
            vert,
            frag,
            gpu::DepthState {
                write_enable: false,
                test_enable: true,
                compare_op: gpu::CompareOp::Greater,
            },
            name,
        )
    }

    pub fn create_base_clipped(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_clipped.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base.frag.spv");
        Self::create_clipped_pipeline(device, &vert, &frag, name)
    }

    pub fn create_base_full(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_full.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base.frag.spv");
        Self::create_full_pipeline(device, &vert, &frag, name)
    }

    pub fn create_shadow_clipped(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_clipped.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/shadow.frag.spv");
        Self::create_clipped_pipeline(device, &vert, &frag, name)
    }

    pub fn create_shadow_full(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_full.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/point_light_passes/shadow.frag.spv");
        Self::create_full_pipeline(device, &vert, &frag, name)
    }

    pub fn create_subsurface_clipped(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_clipped.vert.spv");
        let frag =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/subsurface.frag.spv");
        Self::create_clipped_pipeline(device, &vert, &frag, name)
    }

    pub fn create_subsurface_full(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/cone/point_light_passes/base_full.vert.spv");
        let frag =
            gpu::include_spirv!("../../../shaders/cone/point_light_passes/subsurface.frag.spv");
        Self::create_full_pipeline(device, &vert, &frag, name)
    }
}

impl PointLightRenderer {
    pub fn base_clipped_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), light.buffer.id());
        if let Some(b) = self.base_clipped_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .base_clipped
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };
            self.base_clipped_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    pub fn base_full_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), light.buffer.id());
        if let Some(b) = self.base_full_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .base_full
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };
            self.base_full_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    pub fn shadow_clipped_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), light.buffer.id(), shadow.id);
        if let Some(b) = self.shadow_clipped_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .shadow_clipped
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.shadow_clipped_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    pub fn shadow_full_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), light.buffer.id(), shadow.id);
        if let Some(b) = self.shadow_full_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .shadow_full
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.shadow_full_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    pub fn subsurface_clipped_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
        subsurface: &PointSubsurfaceMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (
            buffer.id,
            camera.buffer.id(),
            light.buffer.id(),
            subsurface.id,
        );
        if let Some(b) = self.subsurface_clipped_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .subsurface_clipped
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .set_resource("u_shadow_map", &(&shadow.texture, &buffer.sampler))
                .unwrap()
                .set_resource("u_subsurface_data", &subsurface.uniform)
                .unwrap()
                .set_combined_texture_sampler_ref(
                    "u_subsurface_map",
                    (&subsurface.texture.view, &buffer.sampler),
                )
                .unwrap()
                .set_resource("u_subsurface_lut", &subsurface.lut)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.subsurface_clipped_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    pub fn subsurface_full_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        light: &PointLight,
        shadow: &PointDepthMap,
        subsurface: &PointSubsurfaceMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (
            buffer.id,
            camera.buffer.id(),
            light.buffer.id(),
            subsurface.id,
        );
        if let Some(b) = self.subsurface_full_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .subsurface_full
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
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
                .set_resource("u_shadow_map", &(&shadow.texture, &buffer.sampler))
                .unwrap()
                .set_resource("u_subsurface_data", &subsurface.uniform)
                .unwrap()
                .set_combined_texture_sampler_ref(
                    "u_subsurface_map",
                    (&subsurface.texture.view, &buffer.sampler),
                )
                .unwrap()
                .set_resource("u_subsurface_lut", &subsurface.lut)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.subsurface_full_bundles.insert(key, b.clone());
            Ok(b)
        }
    }
}

// base passes
impl PointLightRenderer {
    pub fn base_clipped_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        lights: impl IntoIterator<Item = &'a PointLight>,
        strength: f32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.base_clipped
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for light in lights {
            let bundle = self.base_clipped_bundle(device, buffer, camera, light)?;
            pass.set_bundle_into(bundle);
            pass.draw_mesh_owned(&self.sphere);
        }

        Ok(())
    }

    pub fn base_full_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        lights: impl IntoIterator<Item = &'a PointLight>,
        strength: f32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.base_full
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for light in lights {
            let bundle = self.base_full_bundle(device, buffer, camera, light)?;
            pass.set_bundle_into(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }
}

// shadow passes
impl PointLightRenderer {
    pub fn shadow_clipped_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        lights: impl IntoIterator<Item = (&'a PointLight, &'a PointDepthMap)>,
        strength: f32,
        samples: u32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.shadow_clipped
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_u32("samples", samples);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for (light, shadow) in lights {
            let bundle = self.shadow_clipped_bundle(device, buffer, camera, light, shadow)?;
            pass.set_bundle_into(bundle);
            pass.draw_mesh_owned(&self.sphere);
        }

        Ok(())
    }

    pub fn shadow_full_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        lights: impl IntoIterator<Item = (&'a PointLight, &'a PointDepthMap)>,
        strength: f32,
        samples: u32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.shadow_full
                .as_ref()
                .expect("ERROR: PointLightRenderer missing flags"),
        )?;

        pass.push_f32("strength", strength);
        pass.push_u32("samples", samples);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);

        for (light, shadow) in lights {
            let bundle = self.shadow_full_bundle(device, buffer, camera, light, shadow)?;
            pass.set_bundle_into(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }
}

// subsurface passes
impl PointLightRenderer {
    pub fn subsurface_clipped_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        lights: impl IntoIterator<Item = (&'a PointLight, &'a PointDepthMap, &'a PointSubsurfaceMap)>,
        strength: f32,
        subsurface_samples: u32,
        shadow_samples: u32,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.subsurface_clipped
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
                self.subsurface_clipped_bundle(device, buffer, camera, light, shadow, subsurface)?;
            pass.set_bundle_into(bundle);
            pass.draw_mesh_owned(&self.sphere);
        }

        Ok(())
    }

    pub fn subsurface_full_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
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
                    Cow::Owned(buffer.get("output").unwrap().view.clone()),
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
                    Cow::Owned(buffer.depth.view.clone()),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.subsurface_full
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
                self.subsurface_full_bundle(device, buffer, camera, light, shadow, subsurface)?;
            pass.set_bundle_into(bundle);
            pass.draw(0, 3, 0, 1);
        }

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.base_clipped_bundles.clear();
        self.shadow_clipped_bundles.clear();
        self.subsurface_clipped_bundles.clear();
        self.base_full_bundles.clear();
        self.shadow_full_bundles.clear();
        self.subsurface_full_bundles.clear();
    }
}
