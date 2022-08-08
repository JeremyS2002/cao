
use gfx::prelude::*;

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::borrow::Cow;

use rand::Rng;

use crate::cone::GeometryBuffer;
use crate::utils::Camera;

/// Parameters to tweak how ambient occlusion is calculated
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct AOParams {
    /// The number of samples to take for ambient occlusion calculation
    pub kernel_size: u32,
    /// The radius to take the samples in stick to something small
    pub radius: f32,
    /// The bias added to depth to prevent z fighting
    pub bias: f32,
    /// The power to raise the occlusion to, higher powers create more occlusion
    pub power: f32,
    /// random sample vectors to sample depth values from use ..Default::default() for default random values
    pub samples: [glam::Vec3; 64],
}

unsafe impl bytemuck::Pod for AOParams {}
unsafe impl bytemuck::Zeroable for AOParams {}

impl std::default::Default for AOParams {
    fn default() -> Self {
        let mut samples = MaybeUninit::uninit_array();        

        let mut rng = rand::thread_rng();
        for i in 0..64 {
            let u = rng.gen_range(-1.0f32..1.0);
            let v = rng.gen_range(-1.0f32..1.0);
            let w = rng.gen_range(0.0f32..1.0);
            let mut sample = glam::vec3(u, v, w);
            sample = sample.normalize();
            sample *= rng.gen_range(0.0f32..1.0);

            let mut scale = i as f32 / 64.0;
            // scale = lerp(0.1, 1.0, scale * scale);
            scale = 0.1 + ((1.0 - 0.1) * scale * scale);
            
            sample *= scale;

            samples[i].write(sample);
        }

        AOParams {
            kernel_size: 32,
            radius: 0.5,
            bias: 0.025,
            power: 1.0,
            samples: unsafe {
                MaybeUninit::array_assume_init(samples)
            },
        }
    }
}

/// Pipeline management for rendering to the ambient occlusion map of a [`crate::cone::GeometryBuffer`]
pub struct AORenderer {
    /// Calculating ambient occulsion from geometry
    pub calc_pipeline: gfx::ReflectedGraphics,
    pub calc_bundles: HashMap<(u64, u64), gfx::Bundle>,
    /// Blurring calculated occlusion
    pub blur_pipeline: gfx::ReflectedGraphics,
    pub blur_bundles: HashMap<u64, gfx::Bundle>,
    pub noise_sampler: gpu::Sampler,
    pub noise_texture: gfx::GTexture2D,
    pub uniform: gfx::Uniform<AOParams>,
}

impl AORenderer {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        params: AOParams,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        let noise_texture = Self::noise_texture(
            encoder, 
            device,
            4,
            name.as_ref().map(|n| format!("{}_noise_texture", n)),
        )?;
        
        let uniform = gfx::Uniform::new(
            encoder,
            device,
            params,
            name.as_ref().map(|n| format!("{}_uniform", n)),
        )?;
        
        let noise_sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::Repeat,
            wrap_y: gpu::WrapMode::Repeat,
            min_filter: gpu::FilterMode::Nearest,
            mag_filter: gpu::FilterMode::Nearest,
            name: name.as_ref().map(|n| format!("{}_noise_sampler", n)),
            ..Default::default()
        })?;

        let [calc_pipeline, blur_pipeline] = Self::pipelines(device, name)?;

        Ok(Self {
            calc_pipeline,
            calc_bundles: HashMap::new(),
            blur_pipeline,
            blur_bundles: HashMap::new(),
            noise_texture,
            noise_sampler,
            uniform,
        })
    }

    pub fn pipelines(device: &gpu::Device, name: Option<String>) -> Result<[gfx::ReflectedGraphics; 2], gpu::Error> {
        let screen_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/display.vert.spv");
        let calc_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/ao_calc.frag.spv");
        let blur_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/ao_blur.frag.spv");

        let calc_pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &screen_spv,
            None,
            Some(&calc_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            // use depth testing so as to not run where no geometry is
            Some(gpu::DepthStencilState {
                depth: Some(gpu::DepthState {
                    test_enable: true,
                    write_enable: false,
                    compare_op: gpu::CompareOp::Greater,
                }),
                stencil_front: None,
                stencil_back: None,
            }),
            name.as_ref().map(|n| format!("{}_calc_renderer", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            }
        };

        let blur_pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &screen_spv,
            None,
            Some(&blur_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            name.as_ref().map(|n| format!("{}_blur_renderer", n)),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            }
        };

        Ok([calc_pipeline, blur_pipeline])
    }

    pub fn noise_texture(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device, 
        resolution: u32,
        name: Option<String>,
    ) -> Result<gfx::GTexture2D, gpu::Error> {
        use rand::prelude::*;

        let mut noise = Vec::with_capacity(resolution as usize * resolution as usize);
        let mut rng = rand::thread_rng();
        
        for _ in 0..(resolution * resolution) {
            let u = rng.gen_range(-1.0f32..1.0);
            let v = rng.gen_range(-1.0f32..1.0);
            noise.push([u, v]);
        }

        gfx::GTexture2D::from_raw_image(
            encoder, 
            device, 
            &noise, 
            resolution, 
            resolution, 
            gpu::TextureUsage::SAMPLED, 
            1, 
            name,
        )
    }

    /// Create and insert or get a bundle referencing the geometry buffer and camera and return it
    pub fn calc_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
        camera: &Camera,
    ) -> Result<gfx::Bundle, gpu::Error> {
        if let Some(b) = self.calc_bundles.get(&(buffer.id, camera.buffer.id())) {
            Ok(b.clone())
        } else {
            let b = match self.calc_pipeline.bundle().unwrap()
                .set_resource("u_position", buffer.get("position").unwrap())
                .unwrap()
                .set_resource("u_normal", buffer.get("normal").unwrap())
                .unwrap()
                .set_resource("u_buf_sampler", &buffer.sampler)
                .unwrap()
                .set_resource("u_noise", &self.noise_texture)
                .unwrap()
                .set_resource("u_noise_sampler", &self.noise_sampler)
                .unwrap()
                .set_resource("u_data", &self.uniform)
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
            self.calc_bundles.insert((buffer.id, camera.buffer.id()), b.clone());
            Ok(b)
        }
    }

    pub fn blur_bundle(
        &mut self,
        device: &gpu::Device,
        buffer: &GeometryBuffer,
    ) -> Result<gfx::Bundle, gpu::Error> {
        if let Some(b) = self.blur_bundles.get(&buffer.id) {
            Ok(b.clone())
        } else {
            let b = match self.blur_pipeline.bundle().unwrap()
                .set_resource("u_ao_input", buffer.get("ao_tmp").unwrap())
                .unwrap()
                .set_resource("u_sampler", &buffer.sampler)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            self.blur_bundles.insert(buffer.id, b.clone());
            Ok(b)
        }
    }

    pub fn ao_pass<'a>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a GeometryBuffer,
        camera: &Camera,
    ) -> Result<(), gpu::Error> {

        let mut calc_pass = encoder.graphics_pass_reflected::<()>(
            device, 
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("ao_tmp").unwrap().view), 
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
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
            &self.calc_pipeline
        )?;

        let bundle = self.calc_bundle(device, buffer, camera)?;

        let (noise_width, noise_height) = (self.noise_texture.dimension.0, self.noise_texture.dimension.1);
        let noise_scale = [buffer.width as f32 / noise_width as f32, buffer.height as f32 / noise_height as f32];

        calc_pass.push_vec2("noise_scale", noise_scale);
        calc_pass.set_bundle_into(bundle);
        calc_pass.draw(0, 3, 0, 1);

        calc_pass.finish();

        let mut blur_pass = encoder.graphics_pass_reflected::<()>(
            device, 
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&buffer.get("ao").unwrap().view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::DontCare,
                store: gpu::StoreOp::Store,
            }], 
            &[], 
            None, 
            &self.blur_pipeline
        )?;

        let bundle = self.blur_bundle(device, buffer)?;

        let texel_size = 1.0 / glam::vec2(buffer.width as f32, buffer.height as f32);

        blur_pass.push_vec2("texel_size", texel_size.into());
        blur_pass.set_bundle_into(bundle);
        blur_pass.draw(0, 3, 0, 1);

        blur_pass.finish();

        Ok(())
    }
}