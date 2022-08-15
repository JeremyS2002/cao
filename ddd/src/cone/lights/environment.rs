//! Environment lighting effects
//! 
//! This module provides utilities for image based lighting (IBL).
//! IBL provides much more realistic effects especiallly for smooth metallic materials
//!  
//! The basic steps to set up image based lighting are:
//!  - load hdri image
//!  - convert equirectangular image to cubemap texture (see [`new_skybox`])
//!  - convert cubemap texture to environment map (see [`new_env_map`])
//!  - use the environment map to render lighting (see [`EnvironmentRenderer::environment_pass`])

use crate::cone::*;
use crate::utils::*;
use crate::prelude::*;

use gfx::image;
use image::ImageBuffer;
use image::Rgb;
use image::Rgba;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

pub type SkyBox = gfx::GTextureCube;

/// Create a new [`SkyBox`]
///
/// Note: If more than one skybox will be created then the SkyBoxGenerator
/// should be used. This is just a wrapper for
/// ```no_compile
/// let g = SkyBoxGenerator::new(..)?;
/// let skybox = g.generate_from_image(..)?;
/// ```
pub fn new_skybox<'a>(
    encoder: &mut gfx::CommandEncoder<'a>,
    device: &gpu::Device,
    hdri: &'a ImageBuffer<Rgb<f32>, Vec<f32>>,
    resolution: u32,
) -> Result<SkyBox, gpu::Error> {
    let mut generator = SkyBoxGenerator::new(encoder, device)?;

    generator.generate_from_hdri(encoder, device, &hdri, resolution, resolution)
}

/// Create a new [`EnvironmentMap`]
///
/// Note: If more than one environment map will be created then the EnvironmentMapGenerator
/// should be used. This is just a wrapper for
/// ```no_compile
/// let g = EnvironmentmapGenerator::new(..)?
/// let env_map = g.generate(..)?
/// ```
pub fn new_env_map(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    sky: &SkyBox,
    diffuse_resolution: u32,
    specular_resolution: u32,
    brdf_resolution: u32,
    sample_count: u32,
) -> Result<EnvironmentMap, gpu::Error> {
    let generator = EnvironmentMapGenerator::new(encoder, device, None)?;
    let mip_levels = gfx::max_mip_levels(gfx::texture::D1(specular_resolution));
    generator.generate(
        encoder,
        device,
        sky,
        diffuse_resolution,
        diffuse_resolution,
        specular_resolution,
        specular_resolution,
        mip_levels,
        brdf_resolution,
        brdf_resolution,
        sample_count,
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SpecularData {
    sample_count: u32,
    width: u32,
    height: u32,
}

unsafe impl bytemuck::Pod for SpecularData {}
unsafe impl bytemuck::Zeroable for SpecularData {}

/// Builds skyboxes from hdri textures
pub struct SkyBoxGenerator<'a> {
    pub pipeline: gfx::ReflectedGraphics,
    /// TODO: Multiple HashMap from format to compute? to allow for other that Rgba32Float
    pub rgb_to_rgba: Option<gfx::ReflectedCompute>,
    pub sampler: Cow<'a, gpu::Sampler>,
    pub cube: Cow<'a, gfx::BasicMesh<BasicVertex>>,
}

impl SkyBoxGenerator<'static> {
    /// Create a new skybox generator owning its data
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
    ) -> Result<Self, gpu::Error> {
        let pipeline = Self::pipeline(device)?;
        let sampler = device.create_sampler(&gpu::SamplerDesc::default())?;
        let cube = mesh::cube(encoder, device, None)?;
        Ok(Self {
            pipeline,
            rgb_to_rgba: None,
            sampler: Cow::Owned(sampler),
            cube: Cow::Owned(cube),
        })
    }
}

impl<'a> SkyBoxGenerator<'a> {
    pub fn pipeline(device: &gpu::Device) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vertex_spv = gpu::include_spirv!("../../../shaders/cube_push.vert.spv");
        let fragment_spv = gpu::include_spirv!("../../../shaders/cone/creation/skybox.frag.spv");

        match gfx::ReflectedGraphics::from_spv(
            device,
            &vertex_spv,
            None,
            Some(&fragment_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        ) {
            Ok(g) => Ok(g),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }

    pub fn rgb_to_rgba(device: &gpu::Device) -> Result<gfx::ReflectedCompute, gpu::Error> {
        let compute_spv =
            gpu::include_spirv!("../../../shaders/cone/workarounds/rgb32f_to_rgba32f.comp.spv");

        match gfx::ReflectedCompute::new(device, &compute_spv, None) {
            Ok(c) => Ok(c),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }

    /// Create new skybox from an image
    pub fn generate_from_hdri(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        hdri: &'a ImageBuffer<Rgb<f32>, Vec<f32>>,
        width: u32,
        height: u32,
    ) -> Result<SkyBox, gpu::Error> {
        // It is quite common that Rgb32Float is unsupported so if that is the case
        // a work around needs to be done
        // This will be made unnecissary with more robust GTexture::from_image methods and dynamic images
        let mut ok = true;
        if let Ok(p) = device.texture_properties(
            gpu::Format::Rgb32Float,
            gpu::TextureKind::D2,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COPY_DST,
        ) {
            let dimension = gpu::TextureDimension::D2(width, height, gpu::Samples::S1);
            let extent: gpu::Extent3D = dimension.into();
            if extent.width > p.max_extent.width
                || extent.height > p.max_extent.height
                || extent.depth > p.max_extent.depth
            {
                ok = false;
            }
            if 1 > p.max_mip_levels {
                ok = false;
            }
            if dimension.layers() > p.max_array_layers {
                ok = false;
            }
            if !p.sample_counts.contains(gpu::Samples::S1.flags()) {
                ok = false;
            }
        } else {
            ok = false;
        }
        if ok {
            let texture = gfx::Texture2D::from_image(
                encoder,
                device,
                hdri,
                gpu::TextureUsage::SAMPLED,
                1,
                None,
            )?;

            self.generate_from_texture(encoder, device, &texture, width, height)
        } else {
            let texture = gfx::GTexture2D::new(
                device,
                hdri.width(),
                hdri.height(),
                gpu::Samples::S1,
                gpu::TextureUsage::SAMPLED | gpu::TextureUsage::STORAGE,
                1,
                gpu::Format::Rgba32Float,
                None,
            )?;

            let storage = gfx::Storage::new(encoder, device, &hdri, None)?;

            if self.rgb_to_rgba.is_none() {
                self.rgb_to_rgba = Some(Self::rgb_to_rgba(device)?);
            }

            let bundle = match self
                .rgb_to_rgba
                .as_ref()
                .unwrap()
                .bundle()
                .unwrap()
                .set_resource("u_rgb", &storage)
                .unwrap()
                .set_resource("u_output", &texture)
                .unwrap()
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };

            let mut comp_pass =
                encoder.compute_pass_reflected_owned(self.rgb_to_rgba.as_ref().unwrap())?;

            comp_pass.set_bundle_into(bundle);
            comp_pass.push_i32("cols", hdri.width() as _);
            comp_pass.dispatch(hdri.width(), hdri.height(), 1);
            comp_pass.finish();

            self.generate_from_texture(encoder, device, &texture, width, height)
        }
    }

    /// Same as generate from image but using an Rgba image
    /// as Rgb images arn't supported everywhere so reading in
    /// an Rgba image and using that can be easier than running
    /// a compute shader to convert the formats
    pub fn generate_from_image_rgba(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        hdri: &ImageBuffer<Rgba<f32>, Vec<f32>>,
        width: u32,
        height: u32,
    ) -> Result<SkyBox, gpu::Error> {
        // Texture format is infered from the image type
        // very nice!
        let texture = gfx::Texture2D::from_image(
            encoder,
            device,
            hdri,
            gpu::TextureUsage::COPY_SRC,
            1,
            None,
        )?;

        self.generate_from_texture(encoder, device, &texture, width, height)
    }

    /// Create a new skybox from a flat texture
    pub fn generate_from_texture(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        texture: &gfx::GTexture2D,
        width: u32,
        height: u32,
    ) -> Result<SkyBox, gpu::Error> {
        let cube_texture = gfx::GTextureCube::new(
            device,
            width,
            height,
            gpu::TextureUsage::COLOR_OUTPUT | gpu::TextureUsage::SAMPLED,
            1,
            gpu::Format::Rgba32Float,
            None,
        )?;

        let bundle = match self
            .pipeline
            .bundle()
            .unwrap()
            .set_resource("u_texture", texture)
            .unwrap()
            .set_resource("u_sampler", self.sampler.as_ref())
            .unwrap()
            .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };

        let projection = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 10.0);

        let views = [
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::Y, glam::Vec3::Z),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Y, -glam::Vec3::Z),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Z, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::Z, glam::Vec3::Y),
        ];

        for face in gfx::CubeFace::iter() {
            let view = cube_texture.face_view(face)?;
            let mut pass = encoder.graphics_pass_reflected(
                device,
                &[gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(view),
                        gpu::ClearValue::ColorFloat([0.0; 4]),
                    ),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                }],
                &[],
                None,
                &self.pipeline,
            )?;

            pass.set_bundle_owned(&bundle);
            pass.push_mat4("projection", projection.to_cols_array());
            pass.push_mat4("view", views[face as usize].to_cols_array());
            match &self.cube {
                Cow::Borrowed(c) => {
                    pass.draw_mesh_ref(*c);
                }
                Cow::Owned(c) => {
                    pass.draw_mesh_owned(c);
                }
            }
        }

        cube_texture.gen_mipmaps_owned(encoder);

        Ok(cube_texture)
    }
}

/// Builds environment maps from skyboxes
pub struct EnvironmentMapGenerator<'a> {
    pub diffuse_pipeline: gfx::ReflectedGraphics,
    pub specular_pipeline: gfx::ReflectedGraphics,
    pub brdf_pipeline: gfx::ReflectedGraphics,
    pub sampler: Cow<'a, gpu::Sampler>,
    pub cube: Cow<'a, gfx::BasicMesh<BasicVertex>>,
}

impl EnvironmentMapGenerator<'static> {
    /// Create a new skybox generator owning its data
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.as_ref().map(|n| format!("{}_sampler", n)),
            ..Default::default()
        })?;
        let n = name.map(|n| format!("{}_cube", n));
        let cube = mesh::cube(
            encoder, 
            device, 
            n.as_ref().map(|n| &**n),
        )?;
        let [diffuse_pipeline, specular_pipeline, brdf_pipeline] = Self::pipelines(device, name)?;
        
        Ok(Self {
            diffuse_pipeline,
            specular_pipeline,
            brdf_pipeline,
            sampler: Cow::Owned(sampler),
            cube: Cow::Owned(cube),
        })
    }
}

impl<'a> EnvironmentMapGenerator<'a> {
    pub fn pipelines(device: &gpu::Device, name: Option<&str>) -> Result<[gfx::ReflectedGraphics; 3], gpu::Error> {
        let cube_push_vertex_spv = gpu::include_spirv!("../../../shaders/cube_push.vert.spv");
        let cube_buffer_vertex_spv = gpu::include_spirv!("../../../shaders/cube_buffer.vert.spv");
        let diffuse_spv =
            gpu::include_spirv!("../../../shaders/cone/creation/ibl_diffuse.frag.spv");
        let specular_spv =
            gpu::include_spirv!("../../../shaders/cone/creation/ibl_specular.frag.spv");
        let screen_spv = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let brdf_spv = gpu::include_spirv!("../../../shaders/cone/creation/ibl_brdf.frag.spv");

        let n = name.map(|n| format!("{}_diffuse_renderer", n));
        let diffuse = match gfx::ReflectedGraphics::from_spv(
            device,
            &cube_push_vertex_spv,
            None,
            Some(&diffuse_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            n.as_ref().map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let specular = match gfx::ReflectedGraphics::from_spv(
            device,
            &cube_buffer_vertex_spv,
            None,
            Some(&specular_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let brdf = match gfx::ReflectedGraphics::from_spv(
            device,
            &screen_spv,
            None,
            Some(&brdf_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        Ok([diffuse, specular, brdf])
    }

    /// Generate an environment map from
    pub fn generate(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        skybox: &SkyBox,
        diffuse_width: u32,
        diffuse_height: u32,
        specular_width: u32,
        specular_height: u32,
        specular_mip_levels: u32,
        brdf_width: u32,
        brdf_height: u32,
        sample_count: u32,
    ) -> Result<EnvironmentMap, gpu::Error> {
        let diffuse = gfx::GTextureCube::new(
            device,
            diffuse_width,
            diffuse_height,
            gpu::TextureUsage::COLOR_OUTPUT | gpu::TextureUsage::SAMPLED,
            1,
            gpu::Format::Rgba32Float,
            None,
        )?;

        let diffuse_bundle = match self
            .diffuse_pipeline
            .bundle()
            .unwrap()
            .set_resource("u_texture", skybox)
            .unwrap()
            .set_resource("u_sampler", self.sampler.as_ref())
            .unwrap()
            .build(device)  {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };

        let projection = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 10.0);

        let views = [
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::Y, glam::Vec3::Z),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Y, -glam::Vec3::Z),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Z, glam::Vec3::Y),
            glam::Mat4::look_at_rh(glam::Vec3::ZERO, -glam::Vec3::Z, glam::Vec3::Y),
        ];

        for face in gfx::CubeFace::iter() {
            let view = diffuse.face_view(face)?;
            let mut pass = encoder.graphics_pass_reflected(
                device,
                &[gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(view),
                        gpu::ClearValue::ColorFloat([0.0; 4]),
                    ),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                }],
                &[],
                None,
                &self.diffuse_pipeline,
            )?;
            pass.set_bundle_owned(&diffuse_bundle);
            pass.push_mat4("projection", projection.to_cols_array());
            pass.push_mat4("view", views[face as usize].to_cols_array());
            match &self.cube {
                Cow::Borrowed(c) => {
                    pass.draw_mesh_ref(*c);
                }
                Cow::Owned(c) => {
                    pass.draw_mesh_owned(c);
                }
            }
        }

        let specular = gfx::GTextureCube::new(
            device,
            specular_width,
            specular_height,
            gpu::TextureUsage::COLOR_OUTPUT | gpu::TextureUsage::SAMPLED,
            specular_mip_levels,
            gpu::Format::Rgba32Float,
            None,
        )?;

        let specular_data = gfx::Uniform::new(
            encoder,
            device,
            SpecularData {
                sample_count,
                width: specular_width,
                height: specular_height,
            },
            None,
        )?;

        let mut camera = gfx::Uniform::new(
            encoder,
            device,
            CameraData {
                projection,
                view: views[0],
                position: glam::Vec3::ZERO,
            },
            None,
        )?;

        let specular_bundle = match self
            .specular_pipeline
            .bundle()
            .unwrap()
            .set_resource("u_texture", skybox)
            .unwrap()
            .set_resource("u_sampler", self.sampler.as_ref())
            .unwrap()
            .set_resource("u_data", &specular_data)
            .unwrap()
            .set_resource("u_camera", &camera)
            .unwrap()
            .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };

        for mip in 0..specular_mip_levels {
            for face in gfx::CubeFace::iter() {
                let w = (specular_width as f32 * 0.5f32.powi(mip as _)) as u32;
                let h = (specular_height as f32 * 0.5f32.powi(mip as _)) as u32;
                let roughness = mip as f32 / (specular_mip_levels as f32 - 1.0);
                let view = specular.create_view(&gpu::TextureViewDesc {
                    dimension: gpu::TextureDimension::D2(w, h, gpu::Samples::S1),
                    base_mip_level: mip,
                    mip_levels: 1,
                    base_array_layer: face as _,
                    name: None,
                    format_change: None,
                })?;
                camera.data.view = views[face as usize];
                camera.update_gpu_owned(encoder);
                let mut pass = encoder.graphics_pass_reflected(
                    device,
                    &[gfx::Attachment {
                        raw: gpu::Attachment::View(
                            Cow::Owned(view),
                            gpu::ClearValue::ColorFloat([0.0; 4]),
                        ),
                        load: gpu::LoadOp::Clear,
                        store: gpu::StoreOp::Store,
                    }],
                    &[],
                    None,
                    &self.specular_pipeline,
                )?;
                pass.set_bundle_owned(&specular_bundle);
                pass.push_f32("roughness", roughness);
                // pass.push_mat4("projection", projection.to_cols_array());
                // pass.push_mat4("view", views[face as usize].to_cols_array());
                match &self.cube {
                    Cow::Borrowed(c) => {
                        pass.draw_mesh_ref(*c);
                    }
                    Cow::Owned(c) => {
                        pass.draw_mesh_owned(c);
                    }
                }
            }
        }

        let brdf_lut = gfx::GTexture2D::new(
            device,
            brdf_width,
            brdf_height,
            gpu::Samples::S1,
            gpu::TextureUsage::COLOR_OUTPUT | gpu::TextureUsage::SAMPLED,
            1,
            gpu::Format::Rg32Float,
            None,
        )?;

        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(brdf_lut.view.clone()),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.brdf_pipeline,
        )?;
        pass.push_u32("sample_count", sample_count);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        Ok(EnvironmentMap::new(diffuse, specular, brdf_lut))
    }
}

/// A cube texture intended to be used for image based lighting
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EnvironmentMap {
    pub(crate) id: u64,
    pub diffuse: gfx::GTextureCube,
    pub specular: gfx::GTextureCube,
    pub brdf_lut: gfx::GTexture2D,
}

impl EnvironmentMap {
    pub fn new(
        diffuse: gfx::GTextureCube,
        specular: gfx::GTextureCube,
        brdf_lut: gfx::GTexture2D,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        diffuse.hash(&mut hasher);
        specular.hash(&mut hasher);
        brdf_lut.hash(&mut hasher);

        Self {
            id: hasher.finish(),
            diffuse,
            specular,
            brdf_lut,
        }
    }
}

bitflags::bitflags!(
    pub struct EnvironmentRendererFlags: u8 {
        const AMBIENT          = 0b0001;
        const SKYBOX           = 0b0010;
        const ENVIRONMENT      = 0b0100;
    }
);

/// Renders [`Skybox`] and [`Environment`] to the output of a GeometryBuffer
pub struct EnvironmentRenderer {
    pub cube: gfx::BasicMesh<BasicVertex>,
    /// Ambient lighting calculation
    pub ambient: Option<gfx::ReflectedGraphics>,
    pub ambient_bundles: HashMap<u64, gfx::Bundle>,
    /// Skybox effect behind all geometry
    pub skybox: Option<gfx::ReflectedGraphics>,
    pub skybox_bundles: HashMap<(u64, u64, u64), gfx::Bundle>,
    /// Environment map lighting
    pub environment: Option<gfx::ReflectedGraphics>,
    pub environment_bundles: HashMap<(u64, u64, u64), gfx::Bundle>,
    pub sampler: gpu::Sampler,
}

impl EnvironmentRenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        flags: EnvironmentRendererFlags,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let sampler = device.create_sampler(&gpu::SamplerDesc {
            name: name.as_ref().map(|n| format!("{}_sampler", n)),
            ..gpu::SamplerDesc::LINEAR
        })?;

        let cn = name.as_ref().map(|n| format!("{}_cube", n));
        let an = name.as_ref().map(|n| format!("{}_ambient", n));
        let sn = name.as_ref().map(|n| format!("{}_skybox", n));
        let en = name.as_ref().map(|n| format!("{}_environment", n));

        Ok(Self {
            cube: mesh::cube(
                encoder,
                device,
                cn.as_ref().map(|n| &**n),
            )?,
            ambient: if flags.contains(EnvironmentRendererFlags::AMBIENT) {
                Some(Self::create_ambient(
                    device,
                    an.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            ambient_bundles: HashMap::new(),
            skybox: if flags.contains(EnvironmentRendererFlags::SKYBOX) {
                Some(Self::create_skybox(
                    device,
                    sn.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            skybox_bundles: HashMap::new(),
            environment: if flags.contains(EnvironmentRendererFlags::ENVIRONMENT) {
                Some(Self::create_environment(
                    device,
                    en.as_ref().map(|n| &**n),
                )?)
            } else {
                None
            },
            environment_bundles: HashMap::new(),
            sampler,
        })
    }

    pub const LIGHT_BLEND_STATE: gpu::BlendState = gpu::BlendState {
        alpha_blend_op: gpu::BlendOp::Add,
        src_alpha_blend: gpu::BlendFactor::One,
        dst_alpha_blend: gpu::BlendFactor::Zero,
        ..gpu::BlendState::ADD
    };

    pub const LIGHT_RASTERIZER: gpu::Rasterizer = gpu::Rasterizer {
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

    pub const SKYBOX_BLEND_STATE: gpu::BlendState = gpu::BlendState {
        alpha_blend_op: gpu::BlendOp::Add,
        src_alpha_blend: gpu::BlendFactor::One,
        dst_alpha_blend: gpu::BlendFactor::Zero,
        ..gpu::BlendState::ADD
    };

    pub const SKYBOX_RASTERIZER: gpu::Rasterizer = gpu::Rasterizer {
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

    pub fn create_light_pipeline(
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
            Self::LIGHT_RASTERIZER,
            &[Self::LIGHT_BLEND_STATE],
            Some(gpu::DepthStencilState::depth(
                true,
                false,
                gpu::CompareOp::Greater,
            )),
            name,
        ) {
            Ok(g) => Ok(g),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }

    pub fn create_ambient(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/environment/ambient.frag.spv");
        Self::create_light_pipeline(device, &vert, &frag, name)
    }

    pub fn create_environment(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag = gpu::include_spirv!("../../../shaders/cone/environment/environment.frag.spv");
        Self::create_light_pipeline(device, &vert, &frag, name)
    }

    pub fn create_skybox(
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        match gfx::ReflectedGraphics::from_spv(
            device,
            &gpu::include_spirv!("../../../shaders/cube_buffer.vert.spv"),
            None,
            Some(&gpu::include_spirv!(
                "../../../shaders/cone/postprocess/skybox.frag.spv"
            )),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            Some(gpu::DepthStencilState {
                depth: Some(gpu::DepthState {
                    test_enable: true,
                    write_enable: false,
                    compare_op: gpu::CompareOp::LessEqual,
                }),
                stencil_back: None,
                stencil_front: None,
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
}

impl EnvironmentRenderer {
    /// Create and insert or get a bundle referencing the geometry buffer and return it
    pub fn ambient_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
    ) -> Result<gfx::Bundle, gpu::Error> {
        if let Some(b) = self.ambient_bundles.get(&buffer.id) {
            Ok(b.clone())
        } else {
            let b = match self
                .ambient
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_albedo", buffer.get("albedo").unwrap())
                .unwrap()
                .set_resource("u_ao", buffer.get("ao").unwrap())
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
            self.ambient_bundles.insert(buffer.id, b.clone());
            Ok(b)
        }
    }

    /// Create and insert or get a bundle referencing the geometry buffer camera and skybox and return it
    pub fn skybox_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        skybox: &SkyBox,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), skybox.id());
        if let Some(b) = self.skybox_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .skybox
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags")
                .bundle()
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .set_resource("u_skybox", skybox)
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
            self.skybox_bundles.insert(key, b.clone());
            Ok(b)
        }
    }

    /// Create and insert or get a bundle referencing the geometry buffer camera and environment map and return it
    pub fn environment_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        environment: &EnvironmentMap,
    ) -> Result<gfx::Bundle, gpu::Error> {
        let key = (buffer.id, camera.buffer.id(), environment.id);
        if let Some(b) = self.environment_bundles.get(&key) {
            Ok(b.clone())
        } else {
            let b = match self
                .environment
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags")
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
                .set_resource("u_ao", buffer.get("ao").unwrap())
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .set_resource("u_diffuse", &environment.diffuse)
                .unwrap()
                .set_resource("u_specular", &environment.specular)
                .unwrap()
                .set_resource("u_brdf_lut", &environment.brdf_lut)
                .unwrap()
                .build(device) {
                    Ok(b) => b,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    }
                };

            self.environment_bundles.insert(key, b.clone());
            Ok(b)
        }
    }
}

impl EnvironmentRenderer {
    pub fn ambient_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
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
            self.ambient
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags"),
        )?;

        let bundle = self.ambient_bundle(&device, buffer)?;
        pass.push_f32("strength", strength);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);
        pass.set_bundle_into(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }
}

impl EnvironmentRenderer {
    pub fn skybox_pass<'a>(
        &'a mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &'a Camera,
        skybox: &'a SkyBox,
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
                    Cow::Borrowed(&buffer.depth.view),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            }),
            self.skybox
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags"),
        )?;

        let bundle = self.skybox_bundle(&device, buffer, camera, skybox)?;

        pass.push_f32("strength", strength);
        pass.set_bundle_into(bundle);
        pass.draw_mesh_ref(&self.cube);

        Ok(())
    }
}

impl EnvironmentRenderer {
    pub fn environment_pass(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
        environment: &EnvironmentMap,
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
            self.environment
                .as_ref()
                .expect("ERROR: EnvironmentRenderer missing flags"),
        )?;

        let bundle = self.environment_bundle(&device, buffer, camera, environment)?;

        pass.push_f32("max_reflection_lod", environment.specular.texture.mip_levels() as f32);
        pass.push_f32("strength", strength);
        pass.push_f32("width", buffer.width as _);
        pass.push_f32("height", buffer.height as _);
        pass.set_bundle_into(bundle);
        pass.draw(0, 3, 0, 1);

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.ambient_bundles.clear();
        self.environment_bundles.clear();
        self.skybox_bundles.clear();
    }
}
