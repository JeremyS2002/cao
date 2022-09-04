use gfx::prelude::*;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct GaussBlurRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub targets: Arc<Mutex<HashMap<(u32, u32, gpu::Format), gfx::GTexture2D>>>,
    pub bundles: Arc<Mutex<HashMap<u64, gfx::Bundle>>>,
    pub sampler: gpu::Sampler,
    split: bool,
    name: Option<String>,
}

impl std::clone::Clone for GaussBlurRenderer {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
            bundles: Arc::clone(&self.bundles),
            targets: Arc::clone(&self.targets),
            sampler: self.sampler.clone(),
            split: self.split,
            name: self.name.clone(),
        }
    }
}

impl GaussBlurRenderer {
    pub fn new(device: &gpu::Device, split: bool, name: Option<&str>) -> Result<Self, gpu::Error> {
        let vert_spv = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag_spv = if split {
            gpu::include_spirv!("../../../shaders/cone/postprocess/split_gauss_blur.frag.spv")
        } else {
            gpu::include_spirv!("../../../shaders/cone/postprocess/full_gauss_blur.frag.spv")
        };

        let pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&frag_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::ADD],
            None,
            name.map(|n| format!("{}_pipeline", n))
                .as_ref()
                .map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let sampler = device.create_sampler(&gpu::SamplerDesc::new(
            gpu::FilterMode::Linear,
            gpu::WrapMode::ClampToEdge,
            name.map(|n| format!("{}_sampler", n)),
        ))?;

        Ok(Self {
            pipeline,
            bundles: Arc::default(),
            targets: Arc::default(),
            split,
            sampler,
            name: name.map(|n| n.to_string()),
        })
    }

    fn split_pass(
        &self,
        encoder: &mut gfx::CommandEncoder,
        device: &gpu::Device,
        src_view: &gpu::TextureView,
        dst_view: &gpu::TextureView,
        clear_dst: bool,
        radius: f32,
    ) -> Result<(), gpu::Error> {
        let mut targets = self.targets.lock().unwrap();

        let width = src_view.extent().width;
        let height = src_view.extent().height;
        let format = src_view.format();

        if targets.get(&(width, height, format)).is_none() {
            // can't be both split and full so fine not to have usage COPY_DST
            // and know if already cached will have usage COLOR_OUTPUT since can't have been created in full_pass
            let t = gfx::GTexture2D::new(
                device,
                width,
                height,
                gpu::Samples::S1,
                gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
                1,
                format,
                self.name
                    .as_ref()
                    .map(|n| {
                        format!(
                            "{}_tmp_texture_width_{}_height_{}_format_{:?}",
                            n, width, height, format
                        )
                    })
                    .as_ref()
                    .map(|n| &**n),
            )?;
            targets.insert((width, height, format), t);
        }

        let tmp = targets.get(&(width, height, format)).unwrap();

        let mut bundles = self.bundles.lock().unwrap();

        if bundles.get(&src_view.id()).is_none() {
            let b = match self
                .pipeline
                .bundle()
                .unwrap()
                .set_resource("u_color", src_view)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(src_view.id(), b);
        }
        let b1 = bundles.get(&src_view.id()).unwrap().clone();

        if bundles.get(&tmp.view.id()).is_none() {
            let b = match self
                .pipeline
                .bundle()
                .unwrap()
                .set_resource("u_color", &tmp.view)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(tmp.view.id(), b);
        }
        let b2 = bundles.get(&src_view.id()).unwrap().clone();

        if bundles.get(&dst_view.id()).is_none() {
            let b = match self
                .pipeline
                .bundle()
                .unwrap()
                .set_resource("u_color", dst_view)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(dst_view.id(), b);
        }
        let b3 = bundles.get(&dst_view.id()).unwrap().clone();

        let mut pass =
            |src: &gfx::Bundle, dst: &gpu::TextureView, x: bool| -> Result<(), gpu::Error> {
                let mut pass = encoder.graphics_pass_reflected::<()>(
                    device,
                    &[gfx::Attachment {
                        raw: gpu::Attachment::View(
                            Cow::Owned(dst.clone()),
                            gpu::ClearValue::ColorFloat([0.0; 4]),
                        ),
                        load: if clear_dst {
                            gpu::LoadOp::Clear
                        } else {
                            gpu::LoadOp::Load
                        },
                        store: gpu::StoreOp::Store,
                    }],
                    &[],
                    None,
                    &self.pipeline,
                )?;

                pass.set_bundle_owned(src.clone());

                let (width, height) = (dst_view.extent().width, dst_view.extent().height);
                let texel_size = 1.0 / glam::vec2(width as f32, height as f32);
                pass.push_vec2("texel_size", texel_size.into());
                pass.push_f32("radius", radius);
                if x {
                    pass.push_i32("axis", 0);
                } else {
                    pass.push_i32("axis", 1);
                }
                pass.draw(0, 3, 0, 1);
                pass.finish();
                Ok(())
            };

        pass(&b1, &dst_view, true)?;
        pass(&b3, &tmp.view, false)?;
        pass(&b2, dst_view, true)?;
        // pass(&b3, &tmp.view, false)?;
        // pass(&b2, dst_view, true)?;

        Ok(())
    }

    fn full_pass(
        &self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        src_view: &gpu::TextureView,
        dst_view: &gpu::TextureView,
        clear_dst: bool,
        radius: f32,
    ) -> Result<(), gpu::Error> {
        let src = if src_view.texture().id() == dst_view.texture().id() {
            let mut targets = self.targets.lock().unwrap();

            let width = src_view.extent().width;
            let height = src_view.extent().height;
            let format = src_view.format();
            if targets.get(&(width, height, format)).is_none() {
                // can't be both split and full so fine not to have usage COPY_DST
                // and know if already cached will have usage COPY_DST since can't have been created in split_pass
                let t = gfx::GTexture2D::new(
                    device,
                    width,
                    height,
                    gpu::Samples::S1,
                    gpu::TextureUsage::COPY_DST | gpu::TextureUsage::SAMPLED,
                    1,
                    format,
                    self.name
                        .as_ref()
                        .map(|n| {
                            format!(
                                "{}_tmp_texture_width_{}_height_{}_format_{:?}",
                                n, width, height, format
                            )
                        })
                        .as_ref()
                        .map(|n| &**n),
                )?;
                targets.insert((width, height, format), t);
            }

            let t = targets.get(&(width, height, format)).unwrap();

            encoder.blit_textures(
                src_view.texture().whole_slice_owned(),
                t.whole_slice_owned(),
                gpu::FilterMode::Nearest,
            );

            t.view.clone()
        } else {
            src_view.clone()
        };

        let mut bundles = self.bundles.lock().unwrap();
        if bundles.get(&src.id()).is_none() {
            let b = match self
                .pipeline
                .bundle()
                .unwrap()
                .set_resource("u_color", &src)
                .unwrap()
                .set_resource("u_sampler", &self.sampler)
                .unwrap()
                .build(device)
            {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            bundles.insert(src.id(), b);
        }
        let b = bundles.get(&src.id()).unwrap().clone();

        let mut pass = encoder.graphics_pass_reflected::<()>(
            device,
            &[gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(dst_view.clone()),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: if clear_dst {
                    gpu::LoadOp::Clear
                } else {
                    gpu::LoadOp::Load
                },
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.pipeline,
        )?;

        pass.set_bundle_owned(b);

        let (width, height) = (dst_view.extent().width, dst_view.extent().height);
        let texel_size = 1.0 / glam::vec2(width as f32, height as f32);
        pass.push_vec2("texel_size", texel_size.into());
        pass.push_f32("radius", radius);
        pass.draw(0, 3, 0, 1);
        pass.finish();

        Ok(())
    }

    /// Blur the src_view into dst_view
    ///
    /// src_view and dst_view _can_ be the same as the blur is performed from a tempary texture
    pub fn pass(
        &self,
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        src_view: &gpu::TextureView,
        dst_view: &gpu::TextureView,
        clear_dst: bool,
        radius: f32,
    ) -> Result<(), gpu::Error> {
        if self.split {
            self.split_pass(encoder, device, src_view, dst_view, clear_dst, radius)
        } else {
            self.full_pass(encoder, device, src_view, dst_view, clear_dst, radius)
        }
    }
}

/// Postprocessing blur pipeline
#[derive(Debug)]
pub struct ChainBlurRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub targets: Arc<Mutex<HashMap<u64, (gfx::Bundle, Vec<(gfx::GTexture2D, gfx::Bundle)>)>>>,
    pub sampler: gpu::Sampler,
    name: Option<String>,
}

impl std::clone::Clone for ChainBlurRenderer {
    fn clone(&self) -> Self {
        Self {
            pipeline: self.pipeline.clone(),
            targets: Arc::clone(&self.targets),
            sampler: self.sampler.clone(),
            name: self.name.clone(),
        }
    }
}

impl ChainBlurRenderer {
    pub fn new(device: &gpu::Device, name: Option<&str>) -> Result<Self, gpu::Error> {
        let vert_spv = gpu::include_spirv!("../../../shaders/screen.vert.spv");
        let frag_spv = gpu::include_spirv!("../../../shaders/cone/postprocess/chain_blur.frag.spv");

        let pipeline = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&frag_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::ADD],
            None,
            name.map(|n| format!("{}_renderer", n))
                .as_ref()
                .map(|n| &**n),
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            },
        };

        let sampler = device.create_sampler(&gpu::SamplerDesc::new(
            gpu::FilterMode::Linear,
            gpu::WrapMode::ClampToEdge,
            name.map(|n| format!("{}_sampler", n)),
        ))?;

        Ok(Self {
            pipeline,
            targets: Arc::default(),
            sampler,
            name: name.map(|n| n.to_string()),
        })
    }

    /// Blur the view in place
    pub fn pass<'a>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        src_view: &gpu::TextureView,
        dst_view: &gpu::TextureView,
        mut iterations: usize,
        strength: f32,
        clear_dst: bool,
    ) -> Result<(), gpu::Error> {
        iterations = iterations.max(2);

        let pipeline = &self.pipeline;

        let width = src_view.extent().width;
        let height = src_view.extent().height;

        let make_target = |i: usize| {
            let w = width >> i;
            let h = height >> i;
            if w < 2 || h < 2 {
                Result::<_, gpu::Error>::Ok(None)
            } else {
                let n = self
                    .name
                    .as_ref()
                    .map(|n| format!("{}_{:?}_target_{}", n, src_view, i));
                let t = gfx::GTexture2D::from_formats(
                    device,
                    w,
                    h,
                    gpu::Samples::S1,
                    gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
                    1,
                    gfx::alt_formats(src_view.format()),
                    n.as_ref().map(|n| &**n),
                )?
                .unwrap();
                Ok(Some(t))
            }
        };

        let make_bundle = |t: &gfx::GTexture2D| match pipeline
            .bundle()
            .unwrap()
            .set_resource("u_color", t)
            .unwrap()
            .set_resource("u_sampler", &self.sampler)
            .unwrap()
            .build(device)
        {
            Ok(g) => Ok(g),
            Err(e) => match e {
                gfx::BundleBuildError::Gpu(e) => Err(e),
                e => unreachable!("{}", e),
            },
        };

        let mut targets_map = self.targets.lock().unwrap();
        let (view_bundle, targets) =
            if let Some((view_bundle, targets)) = targets_map.get_mut(&src_view.id()) {
                let len = targets.len();
                if len < iterations {
                    for i in len..iterations {
                        if let Some(t) = make_target(i)? {
                            let b = make_bundle(&t)?;
                            targets.push((t, b))
                        } else {
                            iterations = i;
                            break;
                        }
                    }
                }
                (view_bundle, targets)
            } else {
                let view_bundle = match pipeline
                    .bundle()
                    .unwrap()
                    .set_resource("u_color", src_view)
                    .unwrap()
                    .set_resource("u_sampler", &self.sampler)
                    .unwrap()
                    .build(device)
                {
                    Ok(g) => Ok(g),
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e),
                        e => unreachable!("{}", e),
                    },
                }?;
                let mut targets = Vec::new();
                for i in 0..iterations {
                    if let Some(t) = make_target(i)? {
                        let b = make_bundle(&t)?;
                        targets.push((t, b));
                    } else {
                        iterations = i;
                        break;
                    }
                }
                targets_map.insert(src_view.id(), (view_bundle.clone(), targets));
                let (view_bundle, targets) = targets_map.get_mut(&src_view.id()).unwrap();
                (view_bundle, targets)
            };

        let mut pass = |src: &gfx::Bundle,
                        dst: &gpu::TextureView,
                        load: gpu::LoadOp|
         -> Result<(), gpu::Error> {
            let (width, height) = (dst.extent().width, dst.extent().height);
            let texel_size = 1.0 / glam::vec2(width as f32, height as f32);
            let mut pass = encoder.graphics_pass_reflected::<()>(
                device,
                &[gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(dst.clone()),
                        gpu::ClearValue::ColorFloat([0.0; 4]),
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                }],
                &[],
                None,
                &pipeline,
            )?;
            pass.push_vec2("texel_size", texel_size.into());
            pass.push_f32("strength", strength);
            pass.set_bundle_owned(src.clone());
            pass.draw(0, 3, 0, 1);
            pass.finish();
            Ok(())
        };

        // draw from view into the first texture in the chain
        let src = view_bundle;
        let dst = &targets[0].0.view;
        pass(src, dst, gpu::LoadOp::Clear)?;

        // draw down the chain
        for i in 0..(iterations - 1) {
            let src = &targets[i].1;
            let dst = &targets[i + 1].0.view;
            pass(src, dst, gpu::LoadOp::Clear)?;
        }

        let dst_load = if clear_dst {
            gpu::LoadOp::Clear
        } else {
            gpu::LoadOp::Load
        };

        // draw back up the chain
        for i in (1..(iterations)).rev() {
            let src = &targets[i].1;
            let dst = &targets[i - 1].0.view;
            pass(src, dst, dst_load)?;
        }

        // draw from the first texture in the chain into the view
        let src = &targets[0].1;
        let dst = dst_view;
        pass(src, &dst, dst_load)?;

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&self) {
        self.targets.lock().unwrap().clear();
    }
}
