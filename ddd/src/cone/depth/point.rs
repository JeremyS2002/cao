//! Shadow and Subsurface maps to be used with [`crate::cone::PointLight`] or [`crate::cone::PointLights`] as well as pipeline for drawing to shadow maps
//!
//! # Data and Map types
//! [`PointDepthData`] information about shadow or subsurface maps sent to the gpu
//! [`PointDepthMap`] a depth map to be used with [`crate::cone::PointLight`] stored as [`gfx::GTextureCube`] and a [`gfx::Uniform<PointDepthData>`]
//! [`PointSubsurfaceMap`] a [`PointDepthMap`] combined with a look up table used for depth to intensity look up
//! [`PointDepthMaps`] a list of depth maps to be used with [`crate::cone::PointLights`] stored as [`gfx::GTextureCubeArray`] and a [`gfx::Storage<PointDepthData>`]
//! [`PointSubsurfaceMaps`] a [`PointDepthMaps`] combined with a list ok look up tables for depth to intensity look up
//!
//! # Renderer types
//! [`PointDepthMapRenderer`] used for rendering to [`PointDepthMap`] or [`PointDepthMaps`] or their subsurface equivalents
//! see [`PointDepthMapRenderer::single_pass`] to draw to [`PointDepthMap`] or [`PointSubsurfaceMap`]
//! see [`PointDepthMapRenderer::multi_pass`] to draw to [`PointDepthMaps`] or [`PointSubsurfaceMaps`]

use crate::cone::*;
use crate::utils::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::{borrow::Cow, collections::HashMap};

/// projection + view matrices and strength for point shadow
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
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

/// Depth information to be used with a [`crate::cone::PointLight`]
///
/// Depth is stored as a [`gfx::GTextureCube`] and how to interprate it as a [`gfx::Uniform<PointDepthData>`]
#[derive(Debug, Clone)]
pub struct PointDepthMap {
    pub(crate) id: u64,
    pub texture: gfx::GTextureCube,
    pub faces: [gpu::TextureView; 6],
    pub uniform: gfx::Uniform<PointDepthData>,
    pub sampler: gpu::Sampler,
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
            name.as_ref()
                .map(|n| format!("{}_uniform", n))
                .as_ref()
                .map(|n| &**n),
        )?;
        let texture = gfx::GTextureCube::from_formats(
            device,
            width,
            height,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::DEPTH_OUTPUT,
            1,
            gfx::alt_formats(gpu::Format::Depth32Float),
            name.as_ref()
                .map(|n| format!("{}_texture", n))
                .as_ref()
                .map(|n| &**n),
        )?
        .unwrap();
        let faces = [
            texture.face_view(gfx::CubeFace::PosX)?,
            texture.face_view(gfx::CubeFace::NegX)?,
            texture.face_view(gfx::CubeFace::PosY)?,
            texture.face_view(gfx::CubeFace::NegY)?,
            texture.face_view(gfx::CubeFace::PosZ)?,
            texture.face_view(gfx::CubeFace::NegZ)?,
        ];

        let sampler = device.create_sampler(&gpu::SamplerDesc::new(
            gpu::FilterMode::Linear,
            gpu::WrapMode::ClampToEdge,
            name.as_ref().map(|n| format!("{}_sampler", n)),
        ))?;

        Ok(PointDepthMap {
            id: unsafe { std::mem::transmute(texture.raw_image()) },
            texture,
            faces,
            uniform,
            sampler,
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

/// Subsurface information to be used with [`crate::cone::PointLight`]
///
/// depth information is stored as a [`PointDepthMap`] and a look up table stored as a [`gfx::GTexture1D`]
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

/// Depth information to be used with [`crate::cone::PointLights`]
///
/// depth information is stored as a [`gfx::GTextureCubeArray`] and how to interperate it stored as a [`gfx::Storage<PointDepthData>`]
#[derive(Debug, Clone, PartialEq)]
pub struct PointDepthMaps {
    pub(crate) id: u64,
    pub texture: gfx::GTextureCubeArray,
    pub faces: Vec<[gpu::TextureView; 6]>,
    pub data: Vec<PointDepthData>,
    pub storage: gfx::Storage<PointDepthData>,
    pub sampler: gpu::Sampler,
}

impl PointDepthMaps {
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        data: Vec<PointDepthData>,
        width: u32,
        height: u32,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let storage = gfx::Storage::from_vec(encoder, device, data.clone(), name)?;

        let texture = gfx::GTextureCubeArray::from_formats(
            device,
            width,
            height,
            data.len() as _,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::DEPTH_OUTPUT,
            1,
            gfx::alt_formats(gpu::Format::Depth32Float),
            name.as_ref()
                .map(|n| format!("{}_texture", n))
                .as_ref()
                .map(|n| &**n),
        )?
        .unwrap();
        let faces = (0..data.len())
            .map(|i| {
                Ok([
                    texture.face_view(i as u32, gfx::CubeFace::PosX)?,
                    texture.face_view(i as u32, gfx::CubeFace::NegX)?,
                    texture.face_view(i as u32, gfx::CubeFace::PosY)?,
                    texture.face_view(i as u32, gfx::CubeFace::NegY)?,
                    texture.face_view(i as u32, gfx::CubeFace::PosZ)?,
                    texture.face_view(i as u32, gfx::CubeFace::NegZ)?,
                ])
            })
            .collect::<Result<Vec<_>, gpu::Error>>()?;

        let sampler = device.create_sampler(&gpu::SamplerDesc::new(
            gpu::FilterMode::Linear,
            gpu::WrapMode::ClampToEdge,
            name.as_ref().map(|n| format!("{}_sampler", n)),
        ))?;

        Ok(Self {
            id: unsafe { std::mem::transmute(texture.raw_image()) },
            texture,
            faces,
            storage,
            sampler,
            data,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Subsurface data to be used with [`crate::cone::PointLights`]
///
/// depth data stored as a [`PointDepthMaps`] along with a look up table stored as a [`gfx::GTexture1DArray`]
#[derive(Debug, Clone, PartialEq)]
pub struct PointSubsurfaceMaps {
    pub depth: PointDepthMaps,
    pub lut: gfx::GTexture1DArray,
}

impl PointSubsurfaceMaps {
    /// Creata new subsurface map from depth data
    pub fn new(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        data: Vec<PointDepthData>,
        depth_width: u32,
        depth_height: u32,
        lut_width: u32,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let depth = PointDepthMaps::new(encoder, device, data, depth_width, depth_height, name)?;
        Self::from_depth(encoder, device, depth, lut_width)
    }

    /// Create a subsurface map from a depth map
    pub fn from_depth(
        encoder: &mut gfx::CommandEncoder<'_>,
        device: &gpu::Device,
        depth: PointDepthMaps,
        lut_width: u32,
    ) -> Result<Self, gpu::Error> {
        let lut = gfx::GTexture1DArray::new(
            device,
            lut_width,
            depth.data.len() as _,
            gpu::TextureUsage::SAMPLED,
            1,
            gpu::Format::R32Float,
            None,
        )?;

        // make lut for 0 dist to max dist
        let mut layer = 0;
        for d in &depth.data {
            let mut v = Vec::with_capacity(lut_width as usize);
            let incr = d.z_far / lut_width as f32;
            let mut dist = 0.0f32;
            for _ in 0..lut_width {
                v.push((-0.5 * dist).exp());
                dist += incr;
            }
            lut.write_raw_image(encoder, device, &v, layer)?;
            layer += 1;
        }

        Ok(Self { depth, lut })
    }
}

impl std::ops::Deref for PointSubsurfaceMaps {
    type Target = PointDepthMaps;

    fn deref(&self) -> &Self::Target {
        &self.depth
    }
}

impl std::ops::DerefMut for PointSubsurfaceMaps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.depth
    }
}

/// Used for rendering depth maps that correspond to point lights
pub struct PointDepthMapRenderer {
    pub pipeline: gfx::ReflectedGraphics,
    /// map from (instances, shadow) to bundle
    pub single_bundles: Arc<Mutex<HashMap<(u64, u64), gfx::Bundle>>>,
    /// map from (instances, shadow, index) to bundle
    pub multi_bundles: Arc<Mutex<HashMap<(u64, u64, usize), gfx::Bundle>>>,    
}

impl PointDepthMapRenderer {
    /// Create a new [`PointDepthMapRenderer`]
    ///
    /// Used for rendering depth maps that correspond to point lights
    ///
    /// cull_face determins if to cull a face or not
    /// front_face determins what the front face is
    pub fn new(
        device: &gpu::Device,
        cull_face: gpu::CullFace,
        front_face: gpu::FrontFace,
        // geometry_pass: bool,
        cache: Option<gpu::PipelineCache>,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let multi = Self::pipeline(device, cull_face, front_face, cache, name)?;
        Ok(Self {
            pipeline: multi,
            multi_bundles: Arc::default(),
            single_bundles: Arc::default(),
        })
    }

    /// Create the pipeline used for rendering instanced meshes shadows
    pub fn pipeline(
        device: &gpu::Device,
        cull_face: gpu::CullFace,
        front_face: gpu::FrontFace,
        // geometry_pass: bool,
        cache: Option<gpu::PipelineCache>,
        name: Option<&str>,
    ) -> Result<gfx::ReflectedGraphics, gpu::Error> {
        let vertex_spv = gpu::include_spirv!("../../../shaders/cone/shadow_passes/point.vert.spv");

        let fragment_spv =
            gpu::include_spirv!("../../../shaders/cone/shadow_passes/shadow.frag.spv");

        match gfx::ReflectedGraphics::from_spirv(
            device,
            &vertex_spv,
            None,
            Some(&fragment_spv),
            gpu::Rasterizer {
                cull_face,
                front_face,
                depth_clamp: false,
                polygon_mode: gpu::PolygonMode::Fill,
                primitive_topology: gpu::PrimitiveTopology::TriangleList,
                line_width: 1.0,
                depth_bias: false,
                depth_bias_constant: 0.01,
                depth_bias_slope: 1.0,
            },
            &[],
            Some(gpu::DepthStencilState::default_depth()),
            cache,
            name.map(|n| format!("{}_renderer", n))
                .as_ref()
                .map(|n| &**n),
        ) {
            Ok(p) => Ok(p),
            Err(e) => match e {
                gfx::error::ReflectedError::Gpu(e) => Err(e)?,
                _ => unreachable!(),
            },
        }
    }

    /// Draw each of the meshes shadow into the [`PointDepthMap`] supplied
    pub fn single_pass<'a, V: gfx::Vertex>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        shadow: &'a PointDepthMap,
        meshes: impl IntoIterator<Item = (&'a gfx::Mesh<V>, &'a Instances)>,
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
                    raw: gpu::Attachment::View(Cow::Borrowed(face), gpu::ClearValue::Depth(1.0)),
                    load: if clear {
                        gpu::LoadOp::Clear
                    } else {
                        gpu::LoadOp::Load
                    },
                    store: gpu::StoreOp::Store,
                }),
                &self.pipeline,
            )?;

            let mut bundles = self.single_bundles.lock().unwrap();
            for (mesh, instance) in &meshes {
                let key = (instance.buffer.id(), shadow.uniform.buffer.id());

                if bundles.get(&key).is_none() {
                    let b = match self.pipeline.bundle().unwrap()
                        .set_resource("u_instance", *instance).unwrap()
                        .set_resource("u_shadow", &shadow.buffer).unwrap()
                        .build(device) {
                            Ok(b) => b,
                            Err(e) => match e {
                                gfx::BundleBuildError::Gpu(e) => Err(e)?,
                                e => unreachable!("{}", e)
                            },
                        };
                    bundles.insert(key, b.clone());
                }

                let bundle = bundles.get(&key).unwrap().clone();

                pass.push_u32("face", face_idx);
                pass.set_bundle_owned(bundle);
                pass.draw_instanced_mesh_ref(mesh, 0, instance.length as _);
            }

            face_idx += 1;
        }

        Ok(())
    }

    /// Draw each of the meshes shadow into the [`PointDepthMap`] supplied
    pub fn multi_pass<'a, V: gfx::Vertex>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        shadow: &'a PointDepthMaps,
        meshes: impl IntoIterator<Item = (&'a gfx::Mesh<V>, &'a Instances)>,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let meshes = meshes.into_iter().collect::<Vec<_>>();
        let mut face_idx = 0;
        for index in 0..shadow.data.len() {
            for face in &shadow.faces[index] {
                let mut pass = encoder.graphics_pass_reflected(
                    device,
                    &[],
                    &[],
                    Some(gfx::Attachment {
                        raw: gpu::Attachment::View(
                            Cow::Borrowed(face),
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

                let mut multi_bundles = self.multi_bundles.lock().unwrap();
                for (mesh, instance) in &meshes {
                    let key = (instance.buffer.id(), shadow.storage.buffer.id(), index);

                    if multi_bundles.get(&key).is_none() {
                        let b = match self.pipeline.bundle().unwrap()
                            .set_resource("u_instance", *instance).unwrap()
                            .set_buffer("u_shadow", shadow.storage.buffer.slice_ref(
                                (index as u64 * std::mem::size_of::<PointDepthData>() as u64)
                                    ..((index as u64 + 1)
                                        * std::mem::size_of::<PointDepthData>() as u64),
                            )).unwrap()
                            .build(device) {
                                Ok(b) => b,
                                Err(e) => match e {
                                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                                    _ => unreachable!()
                                },
                            };
                        multi_bundles.insert(key, b.clone());
                    }

                    let bundle = multi_bundles.get(&key).unwrap().clone();

                    pass.push_u32("face", face_idx);
                    pass.set_bundle_owned(bundle);
                    // pass.bind_descriptors_owned(0, vec![shadow_set.clone(), instance_set]);
                    pass.draw_instanced_mesh_ref(mesh, 0, instance.length as _);
                }

                face_idx += 1;
            }
        }

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clear(&mut self) {
        self.single_bundles.lock().unwrap().clear();
        self.multi_bundles.lock().unwrap().clear();
        self.pipeline.clear();
    }
}
