use crate::cone::*;
use crate::utils::*;
use crate::prelude::*;

use std::{borrow::Cow, collections::HashMap};

/// projection + view matrices and strength for point shadow
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PointDepthData {
    /// one view matrix for each face in the shadow map
    pub views: [glam::Mat4; 6],
    /// each face uses the same projection matrix
    pub projection: glam::Mat4,
    /// position of the shadow caster
    pub position: glam::Vec3,
    /// the distance to the far plane of the projection matrix
    pub z_far: f32,
    /// strength of the shadow, effects how hard or soft the shadows are
    pub strength: f32,
    /// bias of the shadow map, added to test depths to avoid z fighting effects
    pub bias: f32,
}

unsafe impl bytemuck::Pod for PointDepthData {}
unsafe impl bytemuck::Zeroable for PointDepthData {}

impl PointDepthData {
    /// create PointDepthData from raw matrices
    pub fn from_raw(
        views: [glam::Mat4; 6],
        projection: glam::Mat4,
        position: glam::Vec3,
        z_far: f32,
        strength: f32,
        bias: f32,
    ) -> Self {
        Self {
            projection,
            views,
            strength,
            position,
            z_far,
            bias,
        }
    }

    /// create PointDepthData
    pub fn new(
        position: glam::Vec3,
        projection: glam::Mat4,
        z_far: f32,
        strength: f32,
        bias: f32,
    ) -> Self {
        let views = [
            glam::Mat4::look_at_rh(position, position + glam::Vec3::X, -glam::Vec3::Y),
            glam::Mat4::look_at_rh(position, position - glam::Vec3::X, -glam::Vec3::Y),
            glam::Mat4::look_at_rh(position, position - glam::Vec3::Y, -glam::Vec3::Z),
            glam::Mat4::look_at_rh(position, position + glam::Vec3::Y, glam::Vec3::Z),
            glam::Mat4::look_at_rh(position, position + glam::Vec3::Z, -glam::Vec3::Y),
            glam::Mat4::look_at_rh(position, position - glam::Vec3::Z, -glam::Vec3::Y),
        ];
        Self::from_raw(views, projection, position, z_far, strength, bias)
    }

    /// Create the shadow data from a light
    pub fn from_light(
        light: &PointLightData,
        z_near: f32,
        z_far: f32,
        strength: f32,
        bias: f32,
    ) -> Self {
        Self::from_flipped_perspective(
            light.position,
            std::f32::consts::FRAC_PI_2,
            1.0,
            z_near,
            z_far,
            strength,
            bias,
        )
    }

    /// create PointDepthData from perspective projection
    pub fn from_perspective(
        position: glam::Vec3,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        strength: f32,
        bias: f32,
    ) -> Self {
        let projection = glam::Mat4::perspective_rh(fovy, aspect, z_near, z_far);
        Self::new(position, projection, z_far, strength, bias)
    }

    /// Create PointDepthData from flipped perspective projection
    ///
    /// This flips the y axis to look like it is up (which is down for vulkan)
    pub fn from_flipped_perspective(
        position: glam::Vec3,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        strength: f32,
        bias: f32,
    ) -> Self {
        let t = (fovy / 2.0).tan();
        let sy = 1.0 / t;
        let sx = sy / aspect;
        let nmf = z_near - z_far;
        let projection = glam::Mat4::from_cols(
            glam::vec4(sx, 0.0, 0.0, 0.0),
            glam::vec4(0.0, -sy, 0.0, 0.0),
            glam::vec4(0.0, 0.0, z_far / nmf, -1.0),
            glam::vec4(0.0, 0.0, z_near * z_far / nmf, 0.0),
        );
        Self::new(position, projection, z_far, strength, bias)
    }

    /// create PointDepthData from orthographic projection
    pub fn from_orthographic(
        position: glam::Vec3,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        strength: f32,
        z_near: f32,
        z_far: f32,
        bias: f32,
    ) -> Self {
        let projection = glam::Mat4::orthographic_rh(left, right, bottom, top, z_near, z_far);
        Self::new(position, projection, z_far, strength, bias)
    }

    pub fn set_position(&mut self, postion: glam::Vec3) {
        self.position = postion;
        let views = [
            glam::Mat4::look_at_rh(self.position, self.position + glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(self.position, self.position - glam::Vec3::X, glam::Vec3::Y),
            glam::Mat4::look_at_rh(self.position, self.position + glam::Vec3::Y, glam::Vec3::Z),
            glam::Mat4::look_at_rh(self.position, self.position - glam::Vec3::Y, -glam::Vec3::Z),
            glam::Mat4::look_at_rh(self.position, self.position + glam::Vec3::Z, glam::Vec3::Y),
            glam::Mat4::look_at_rh(self.position, self.position - glam::Vec3::Z, glam::Vec3::Y),
        ];
        self.views = views;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointSubsurfaceMap {
    pub depth: PointDepthMap,
    pub lut: gfx::GTexture1D,
}

impl PointSubsurfaceMap {
    /// Creata new subsurface map from depth data
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        data: PointDepthData,
        depth_width: u32,
        depth_height: u32,
        lut_width: u32,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let depth = PointDepthMap::new(encoder, device, data, depth_width, depth_height, name)?;
        Self::from_depth(encoder, device, depth, lut_width)
    }

    /// Create a subsurface map from a depth map
    pub fn from_depth(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        depth: PointDepthMap,
        width: u32,
    ) -> Result<Self, gpu::Error> {
        // make lut for 0 dist to max dist
        let mut vec = Vec::with_capacity(width as _);
        let incr = depth.uniform.data.z_far / width as f32;
        let mut dist = 0.0f32;
        for _ in 0..width {
            vec.push((-0.5 * dist).exp());
            dist += incr;
        }

        let lut = gfx::GTexture1D::from_raw_image(
            encoder,
            device,
            width,
            &vec,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        Ok(Self { depth, lut })
    }
}

impl std::ops::Deref for PointSubsurfaceMap {
    type Target = PointDepthMap;

    fn deref(&self) -> &Self::Target {
        &self.depth
    }
}

impl std::ops::DerefMut for PointSubsurfaceMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.depth
    }
}

#[derive(Debug, Clone)]
pub struct PointDepthMap {
    pub(crate) id: u64,
    pub texture: gfx::GTextureCube,
    pub faces: [gpu::TextureView; 6],
    pub uniform: gfx::Uniform<PointDepthData>,
}

impl std::hash::Hash for PointDepthMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::cmp::PartialEq for PointDepthMap {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Eq for PointDepthMap {}

impl PointDepthMap {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        data: PointDepthData,
        width: u32,
        height: u32,
        name: Option<&str>,
    ) -> Result<PointDepthMap, gpu::Error> {
        let uniform = gfx::Uniform::new(
            encoder, 
            device, 
            data, 
            name.as_ref().map(|n| format!("{}_uniform", n)).as_ref().map(|n| &**n)
        )?;
        let texture = gfx::GTextureCube::new(
            device,
            width,
            height,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::DEPTH_OUTPUT,
            1,
            gpu::Format::Depth32Float,
            name.as_ref().map(|n| format!("{}_texture", n)).as_ref().map(|n| &**n),
        )?;
        let faces = [
            texture.face_view(gfx::CubeFace::PosX)?,
            texture.face_view(gfx::CubeFace::NegX)?,
            texture.face_view(gfx::CubeFace::PosY)?,
            texture.face_view(gfx::CubeFace::NegY)?,
            texture.face_view(gfx::CubeFace::PosZ)?,
            texture.face_view(gfx::CubeFace::NegZ)?,
        ];
        Ok(PointDepthMap {
            id: unsafe { std::mem::transmute(texture.raw_image()) },
            texture,
            faces,
            uniform,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl std::ops::Deref for PointDepthMap {
    type Target = gfx::Uniform<PointDepthData>;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl std::ops::DerefMut for PointDepthMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}

pub struct PointDepthMapRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub shadow_map: HashMap<u64, gpu::DescriptorSet>,
    pub instances_map: HashMap<u64, gpu::DescriptorSet>,
}

impl PointDepthMapRenderer {
    pub fn new(
        device: &gpu::Device,
        cull_face: Option<gpu::FrontFace>,
    ) -> Result<Self, gpu::Error> {
        let multi = Self::pipeline(device, cull_face)?;
        Ok(Self {
            pipeline: multi,
            shadow_map: HashMap::new(),
            instances_map: HashMap::new(),
        })
    }

    /// Create the pipeline used for rendering instanced meshes shadows
    pub fn pipeline(
        device: &gpu::Device,
        cull_face: Option<gpu::FrontFace>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vertex_spv =
            gpu::include_spirv!("../../../shaders/cone/shadow_passes/point.vert.spv");
        let fragment_spv =
            gpu::include_spirv!("../../../shaders/cone/shadow_passes/shadow.frag.spv");
        match gfx::ReflectedGraphics::from_spv(
            device,
            &vertex_spv,
            None,
            Some(&fragment_spv),
            gpu::Rasterizer {
                cull_face: cull_face
                    .map(|_| gpu::CullFace::Front)
                    .unwrap_or(gpu::CullFace::None),
                front_face: cull_face.unwrap_or(gpu::FrontFace::Clockwise),
                ..Default::default()
            },
            &[],
            Some(gpu::DepthStencilState::default_depth()),
            None,
        ) {
            Ok(p) => Ok(p),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }

    /// Draw each of the meshes shadows into the supplied shadow map with the instance they are paired with
    pub fn pass<'a, 'b, V: gfx::Vertex>(
        &mut self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        shadow: &PointDepthMap,
        meshes: impl IntoIterator<Item = (&'b gfx::Mesh<V>, &'b Instances)>,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let meshes = meshes.into_iter().collect::<Vec<_>>();
        let mut face_idx = 0;
        for face in &shadow.faces {
            let mut pass = encoder.graphics_pass_reflected(
                device,
                &[],
                &[],
                Some(gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(face.clone()),
                        gpu::ClearValue::Depth(1.0),
                    ),
                    load: if clear {
                        gpu::LoadOp::Clear
                    } else {
                        gpu::LoadOp::Load
                    },
                    store: gpu::StoreOp::Store,
                }),
                &self.pipeline,
            )?;
            let key = unsafe { std::mem::transmute(shadow.uniform.buffer.raw_buffer()) };
            let shadow_set = if let Some(s) = self.shadow_map.get(&key) {
                s.clone()
            } else {
                let s = device.create_descriptor_set(&gpu::DescriptorSetDesc {
                    name: None,
                    entries: &[gpu::DescriptorSetEntry::Buffer(
                        shadow.uniform.buffer.slice_ref(..),
                    )],
                    layout: self
                        .pipeline
                        .reflect_data
                        .descriptor_layouts()
                        .unwrap()
                        .get(0)
                        .unwrap(),
                })?;
                self.instances_map.insert(key, s.clone());
                s
            };
            for (mesh, instance) in &meshes {
                let key = unsafe { std::mem::transmute(instance.buffer.raw_buffer()) };

                let instance_set = if let Some(b) = self.instances_map.get(&key) {
                    b.clone()
                } else {
                    let s = device.create_descriptor_set(&gpu::DescriptorSetDesc {
                        name: None,
                        entries: &[gpu::DescriptorSetEntry::Buffer(
                            instance.buffer.slice_ref(..),
                        )],
                        layout: self
                            .pipeline
                            .reflect_data
                            .descriptor_layouts()
                            .unwrap()
                            .get(1)
                            .unwrap(),
                    })?;
                    self.instances_map.insert(key, s.clone());
                    s
                };

                pass.push_u32("face", face_idx);
                pass.bind_descriptors_owned(0, vec![shadow_set.clone(), instance_set.clone()]);
                pass.draw_instanced_mesh_owned(*mesh, 0, instance.length as _);
            }

            face_idx += 1;
        }

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.instances_map.clear();
        self.shadow_map.clear();
    }
}

pub struct PointDepthMaps {
    pub texture: gfx::GTexture2DArray,
    pub storages: gfx::Storage<PointDepthData>,
}

pub struct PointDepthMapsRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    pub multi_pipeline: gfx::ReflectedGraphics,
    pub single_shadow_map: HashMap<u64, gpu::DescriptorSet>,
    pub single_instance_map: HashMap<u64, gpu::DescriptorSet>,
    pub multi_shadow_map: HashMap<u64, gpu::DescriptorSet>,
    pub multi_instances_map: HashMap<u64, gpu::DescriptorSet>,
}
