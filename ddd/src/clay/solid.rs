//! Simple render for drawin meshes in solid color

use std::collections::HashMap;

use crate::utils::{Camera, Instance};

pub struct SolidRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub bundles: HashMap<(u64, u64), gfx::Bundle>,
}

impl SolidRenderer {
    pub fn new(device: &gpu::Device, name: Option<String>) -> Result<Self, gpu::Error> {
        let pipeline = Self::pipeline(device, name)?;
        Ok(Self {
            pipeline,
            bundles: HashMap::new(),
        })
    }

    pub fn pipeline(device: &gpu::Device, name: Option<String>) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vert_spv = gpu::include_spirv!("../../shaders/clay/camera.vert.spv");
        let frag_spv = gpu::include_spirv!("../../shaders/clay/solid.frag.spv");
        
        let g = match gfx::ReflectedGraphics::from_spv(
            device,
            &vert_spv,
            None,
            Some(&frag_spv),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::ALPHA],
            Some(gpu::DepthStencilState {
                depth: Some(gpu::DepthState {
                    test_enable: true,
                    write_enable: true,
                    compare_op: gpu::CompareOp::LessEqual,
                }),
                stencil_front: None,
                stencil_back: None,
            }),
            name.as_ref().map(|n| format!("{}_renderer", n))
        ) {
            Ok(g) => g,
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                e => unreachable!("{}", e),
            }
        };

        Ok(g)
    }

    pub fn bundle(
        &mut self,
        device: &gpu::Device,
        camera: &Camera,
        instance: &Instance,
    ) -> Result<gfx::Bundle, gpu::Error> {
        if let Some(b) = self.bundles.get(&(camera.buffer.id(), instance.buffer.id())) {
            Ok(b.clone())
        } else {
            let b = match self.pipeline.bundle().unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .set_resource("u_instance", instance)
                .unwrap()
                .build(device) {
                Ok(b) => b,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                }
            };
            
            self.bundles.insert((camera.buffer.id(), instance.buffer.id()), b.clone());
            Ok(b)
        }
    }

    pub fn pass<'a, 'b, V: gfx::Vertex>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        target: gfx::Attachment<'a>,
        depth: gfx::Attachment<'a>,
        meshes: impl IntoIterator<Item=(&'a dyn gfx::Mesh<V>, &'b Instance, [f32; 4])>,
        camera: &Camera,
    ) -> Result<(), gpu::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            device,
            &[target],
            &[],
            Some(depth),
            &self.pipeline
        )?;


        for (mesh, instance, color) in meshes.into_iter() {
            let bundle = self.bundle(device, camera, instance)?;

            pass.set_bundle_into(bundle);
            pass.push_vec4("u_color", color);
            pass.draw_mesh_ref(mesh);
        }
        
        pass.finish();

        Ok(())
    }
}