//! Forward rendering for debugging applications

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub normal: glam::Vec3,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl gfx::Vertex for Vertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
            "in_normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            "pos" => Some((0, gpu::VertexFormat::Vec3)),
            "normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            _ => None,
        }
    }
}

impl mesh::Vertex for Vertex {
    fn new(
        pos: glam::Vec3,
        _: glam::Vec2,
        normal: glam::Vec3,
        _: Option<glam::Vec3>,
        _: Option<glam::Vec3>,
    ) -> Self {
        Self { pos, normal }
    }

    fn set_tangents(&mut self, _: glam::Vec3, _: glam::Vec3) {
        println!("Call to set tangents of ddd::clay::Vertex, no tangent fields so no action taken")
    }

    fn pos(&self) -> glam::Vec3 {
        self.pos
    }

    fn uv(&self) -> Option<glam::Vec2> {
        None
    }

    fn normal(&self) -> Option<glam::Vec3> {
        Some(self.normal)
    }

    fn tangent_u(&self) -> Option<glam::Vec3> {
        None
    }

    fn tangent_v(&self) -> Option<glam::Vec3> {
        None
    }
}

use std::collections::HashMap;

use crate::utils::{Camera, Instances};   

#[macro_export]
macro_rules! impl_renderer {
    ($(#[$attr:meta])* => $name:ident, $vert:literal, $frag:literal) => {
        $(#[$attr])*
        pub struct $name {
            pub pipeline: gfx::ReflectedGraphics,
            pub bundles: HashMap<(u64, u64), gfx::Bundle>,
        }
        
        impl $name {
            pub fn new(device: &gpu::Device, name: Option<String>) -> Result<Self, gpu::Error> {
                let pipeline = Self::pipeline(device, name)?;
                Ok(Self {
                    pipeline,
                    bundles: HashMap::new(),
                })
            }
        
            pub fn pipeline(device: &gpu::Device, name: Option<String>) -> Result<gfx::ReflectedGraphics, gpu::Error> {
                let vert_spv = gpu::include_spirv!($vert);
                let frag_spv = gpu::include_spirv!($frag);
                
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
                instance: &Instances,
            ) -> Result<gfx::Bundle, gpu::Error> {
                if let Some(b) = self.bundles.get(&(camera.buffer.id(), instance.buffer.id())) {
                    Ok(b.clone())
                } else {
                    let b = match self.pipeline.bundle().unwrap()
                        .set_resource("u_camera", camera)
                        .unwrap()
                        .set_resource("u_instances", instance)
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
                meshes: impl IntoIterator<Item=(&'a dyn gfx::Mesh<V>, &'b Instances, [f32; 4])>,
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
                    //pass.draw_mesh_ref(mesh);
                    pass.draw_instanced_mesh_owned(mesh, 0, instance.length as _);
                }
                
                pass.finish();
        
                Ok(())
            }

            /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
            /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
            /// This function drops Descriptor sets cached by self
            pub fn clean(&mut self) {
                self.bundles.clear();
            }
        }
    };
}


impl_renderer!(
    /// A simple forward renderer for drawing objects in a solid color
    /// 
    /// This can function as a simple shader for transparent objects.
    /// To do this first draw all non-transparent objects into a [`crate::clay::GeometryBuffer`] 
    /// Then draw transparent objects into the geometry buffers output useing it's depth texture and color with alpha component less than 1.0
    =>
    SolidRenderer, 
    "../shaders/clay/solid.vert.spv", 
    "../shaders/clay/solid.frag.spv"
);

impl_renderer!(
    /// A simple forward renderer for debugging objects geometry
    /// 
    /// Brightness is determined by how closely the normal aligns with the view vector
    /// No interpolation is performed on the normals
    =>
    FlatRenderer,
    "../shaders/clay/flat.vert.spv",
    "../shaders/clay/flat.frag.spv"
);

impl_renderer!(
    /// A simple forward renderer for debugging objects geometry
    /// 
    /// Brightness is determined by how closely the normal aligns with the view vector
    /// Normal vectors are interpolated across faces
    =>
    SmoothRenderer,
    "../shaders/clay/smooth.vert.spv",
    "../shaders/clay/smooth.frag.spv"
);