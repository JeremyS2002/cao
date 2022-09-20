use gfx::GraphicsPass;

use std::sync::Arc;
use std::sync::Mutex;
use std::{borrow::Cow, collections::HashMap};
use std::cell::RefCell;

use crate::utils::*;

use either::*;
use glam::Vec4Swizzles;

pub type MaterialParams = gfx::Uniform<MaterialData>;

/// Used to send default values when not sampling from textures
#[repr(C)]
#[derive(Debug, Clone, Copy, spv::AsStructType)]
pub struct MaterialData {
    pub albedo: glam::Vec4,
    pub subsurface: glam::Vec4,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            albedo: glam::vec4(0.7, 0.7, 0.7, 1.0),
            subsurface: glam::vec4(0.0, 0.0, 0.0, 0.0),
            roughness: 0.5,
            metallic: 0.0,
        }
    }
}

unsafe impl bytemuck::Pod for MaterialData {}
unsafe impl bytemuck::Zeroable for MaterialData {}

/// Builds a Materials shader modules as well a bundle
pub struct MaterialBuilder<'a> {
    /// builds the vertex module
    vertex: spv::Builder,
    /// builds the fragment module
    fragment: spv::Builder,
    /// resources used in the shaders
    resources: RefCell<Vec<&'a dyn gfx::Resource>>,

    /// outputs required for the fragment material

    /// the position of the fragment in world space
    pub world_pos: spv::Output<spv::IOVec3>,
    /// the position of the fragment in view space
    pub view_pos: spv::Output<spv::IOVec3>,
    /// the normal of the fragment in world space
    pub normal: spv::Output<spv::IOVec3>,
    /// the albedo of the fragment
    pub albedo: spv::Output<spv::IOVec4>,
    /// the roughness of the fragment
    pub roughness: spv::Output<spv::IOFloat>,
    /// how metallic the fragment is
    pub metallic: spv::Output<spv::IOFloat>,
    /// optional subsurface output
    pub subsurface: spv::Output<spv::IOVec4>,
    /// the uv coordinate at that point
    pub uv: spv::Output<spv::IOVec2>,
}

impl<'a> MaterialBuilder<'a> {
    /// Create a new MaterialBuilder
    pub fn new() -> Self {
        let vertex = spv::Builder::new();
        let fragment = spv::Builder::new();
        let world_pos = fragment.output(0, false, Some("out_world_pos"));
        let view_pos = fragment.output(1, false, Some("out_view_pos"));
        let normal = fragment.output(2, false, Some("out_normal"));
        let albedo = fragment.output(3, false, Some("out_albedo"));
        let roughness = fragment.output(4, false, Some("out_roughness"));
        let metallic = fragment.output(5, false, Some("out_metallic"));
        let subsurface = fragment.output(6, false, Some("out_subsurface"));
        let uv = fragment.output(7, false, Some("out_uv"));

        Self {
            vertex,
            fragment,
            resources: RefCell::new(Vec::new()),

            world_pos,
            view_pos,
            normal,
            albedo,
            roughness,
            metallic,
            subsurface,
            uv,
        }
    }

    /// Sets a camera in the vertex shader and returns the spir-v uniform
    pub fn camera(&self) -> spv::Uniform<crate::utils::camera::SpvCameraData> {
        self.vertex.uniform::<SpvCameraData>(0, 0, Some("u_camera"))
    }

    pub fn instances(&self) -> spv::Storage<SpvInstanceData> {
        self.vertex.storage::<SpvInstanceData>(
            // spv::StorageAccessDesc {
            //     read: true,
            //     write: false,
            //     atomic: false,
            // },
            1,
            0,
            Some("u_instances"),
        )
    }

    /// Creates a simple vertex state, can have one or multiple instances
    ///
    /// The vertex builder can't be used after this function
    /// returns (in_world_pos, in_view_pos, in_normal, in_uv) for the fragment shader
    pub fn default_vertex(
        &self,
    ) -> (
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec2>,
    ) {
        let in_pos = self.vertex.in_vec3(0, "in_pos");
        let in_normal = self.vertex.in_vec3(1, "in_normal");
        let in_uv = self.vertex.in_vec2(2, "in_uv");

        let out_world_pos = self.vertex.out_vec3(0, "out_world_pos");
        let out_view_pos = self.vertex.out_vec3(1, "out_view_pos");
        let out_normal = self.vertex.out_vec3(2, "out_normal");
        let out_uv = self.vertex.out_vec2(3, "out_uv");

        let camera = self.camera();
        let instances = self.instances();

        let instance_idx = self.vertex.instance_index();

        let vk_pos = self.vertex.vk_position();

        let b = &self.vertex;

        self.vertex.entry(spv::ShaderStage::Vertex, "main", || {
            let camera = camera.load();
            let projection = camera.projection();
            let view = camera.view();

            let idx = instance_idx.load();

            let model = instances.load_element(&idx).model();
            let pos = in_pos.load();
            let world_pos = model * b.vec4(&pos.x(), &pos.y(), &pos.z(), &1.0);
            out_world_pos.store(world_pos.xyz());
            let view_pos = view * world_pos;
            out_view_pos.store(view_pos.xyz());
            let screen_pos = projection * view_pos;
            vk_pos.store(screen_pos);

            let normal = in_normal.load();
            let model_x = model.col(0).xyz();
            let model_y = model.col(1).xyz();
            let model_z = model.col(2).xyz();
            let model3 = b.mat3(&model_x, &model_y, &model_z);
            let normal = model3 * normal;
            out_normal.store(normal.normalized());

            out_uv.store(in_uv.load());
        });

        let in_world_pos = self.fragment.in_vec3(0, "in_pos");
        let in_view_pos = self.fragment.in_vec3(1, "in_view_pos");
        let in_normal = self.fragment.in_vec3(2, "in_normal");
        let in_uv = self.fragment.in_vec2(3, "in_uv");

        (in_world_pos, in_view_pos, in_normal, in_uv)
    }

    /// Returns a vertex shader with a single instance
    ///
    /// The vertex builder can't be used after this function
    /// returns (in_world_pos, in_view_pos, in_uv, in_t, in_b, in_n) for the fragment shader
    pub fn tbn_vertex(
        &mut self,
    ) -> (
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec2>,
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec3>,
        spv::Input<spv::IOVec3>,
    ) {
        let in_pos = self.vertex.in_vec3(0, "in_pos");
        let in_normal = self.vertex.in_vec3(1, "in_normal");
        let in_uv = self.vertex.in_vec2(2, "in_uv");
        let in_tangent = self.vertex.in_vec3(3, "in_tangent");

        let out_world_pos = self.vertex.out_vec3(0, "out_world_pos");
        let out_view_pos = self.vertex.out_vec3(1, "out_view_pos");
        let out_uv = self.vertex.out_vec2(2, "out_uv");
        let out_t = self.vertex.out_vec3(3, "out_t");
        let out_b = self.vertex.out_vec3(4, "out_b");
        let out_n = self.vertex.out_vec3(5, "out_n");

        let camera = self.camera();
        let instances = self.instances();

        let instance_idx = self.vertex.instance_index();

        let vk_pos = self.vertex.vk_position();

        let b = &self.vertex;

        self.vertex.entry(spv::ShaderStage::Vertex, "main", || {
            let camera = camera.load();
            let projection = camera.projection();
            let view = camera.view();

            let idx = instance_idx.load();

            let model = instances.load_element(&idx).model();
            
            let pos = in_pos.load();
            let world_pos = model * b.vec4(&pos.x(), &pos.y(), &pos.z(), &1.0);
            out_world_pos.store(world_pos.xyz());
            let view_pos = view * world_pos;
            out_view_pos.store(view_pos.xyz());
            let screen_pos = projection * view_pos;
            vk_pos.store(screen_pos);

            let tangent = in_tangent.load();
            let tangent = (model * b.vec4(&tangent.x(), &tangent.y(), &tangent.z(), &0.0)).xyz().normalized();
            let normal = in_normal.load();
            let normal = (model * b.vec4(&normal.x(), &normal.y(), &normal.z(), &0.0)).xyz().normalized();
            let tangent = (tangent - (tangent.dot(&normal) * normal)).normalized();
            let bitangent = normal.cross(&tangent);

            out_t.store(tangent);
            out_b.store(bitangent);
            out_n.store(normal);

            out_uv.store(in_uv.load());
        });

        let in_world_pos = self.fragment.in_vec3(0, "in_world_pos");
        let in_view_pos = self.fragment.in_vec3(1, "in_view_pos");
        let in_uv = self.fragment.in_vec2(2, "in_uv");
        let in_t = self.fragment.in_vec3(3, "in_t");
        let in_b = self.fragment.in_vec3(4, "in_b");
        let in_n = self.fragment.in_vec3(5, "in_n");

        (in_world_pos, in_view_pos, in_uv, in_t, in_b, in_n)
    }

    /// Set the outputs to sample from the textures
    ///
    /// The fragment builder can't be used after this function
    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn textured_fragment(
        &mut self,
        world_pos: spv::Input<spv::IOVec3>,
        view_pos: spv::Input<spv::IOVec3>,
        normal: spv::Input<spv::IOVec3>,
        uv: spv::Input<spv::IOVec2>,
        albedo: &'a gfx::Texture2D,
        roughness: &'a gfx::Texture2D,
        metallic: Option<&'a gfx::Texture2D>,
        subsurface: Option<&'a gfx::Texture2D>,
        sampler: &'a gpu::Sampler,
        discard: bool,
    ) {
        self.textured_or_default_fragment(
            world_pos,
            view_pos,
            Left(normal),
            uv,
            Some(albedo),
            Some(roughness),
            metallic,
            subsurface,
            sampler,
            discard,
            &MaterialData::default(),
        )
    }

    /// Set the outputs to read from the uniform buffer
    ///
    /// The fragment builder can't be used after this function
    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn uniform_fragment(
        &mut self,
        world_pos: spv::Input<spv::IOVec3>,
        view_pos: spv::Input<spv::IOVec3>,
        normal: spv::Input<spv::IOVec3>,
        uniform: &'a super::MaterialParams,
        _discard: bool,
    ) {
        let params = self.set_fragment_uniform(&uniform, Some("u_params"));

        let b = &self.fragment;

        b.entry(spv::ShaderStage::Fragment, "main", || {
            let params = params.load();
            self.world_pos.store(world_pos.load());
            self.view_pos.store(view_pos.load());
            self.normal.store(normal.load());
            self.albedo.store(params.albedo());
            self.roughness.store(params.roughness());
            self.metallic.store(params.roughness());
            let subsurface = params.subsurface();
            let tmp = (-1.0 / subsurface.xyz()).exp();
            self.subsurface.store(b.vec4(&tmp.x(), &tmp.y(), &tmp.z(), &subsurface.w()));
            self.uv.store(b.vec2(&0.0, &0.0));
        });
    }

    /// Set the output to be constant values
    ///
    /// The fragment builder can't be used after this function
    pub fn constant_fragment(
        &mut self,
        world_pos: spv::Input<spv::IOVec3>,
        view_pos: spv::Input<spv::IOVec3>,
        normal: spv::Input<spv::IOVec3>,
        constants: &MaterialData,
    ) {
        let mut tmp = constants.subsurface.xyz();
        tmp = (-1.0 / tmp).exp();
        let subsurface = glam::vec4(tmp.x, tmp.y, tmp.z, constants.subsurface.w);
        let b = &self.fragment;
        b.entry(spv::ShaderStage::Fragment, "main", || {
            self.world_pos.store(world_pos.load());
            self.view_pos.store(view_pos.load());
            self.normal.store(normal.load());
            self.albedo.store(b.const_vec4(constants.albedo));
            self.roughness.store(b.const_float(constants.roughness));
            self.metallic.store(b.const_float(constants.metallic));
            self.subsurface.store(b.const_vec4(subsurface));
            self.uv.store(b.vec2(&0.0, &0.0));
        });
    }

    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn textured_or_default_fragment(
        &mut self,
        world_pos: spv::Input<spv::IOVec3>,
        view_pos: spv::Input<spv::IOVec3>,
        normal: Either<
            spv::Input<spv::IOVec3>,
            (
                spv::Input<spv::IOVec3>,
                spv::Input<spv::IOVec3>,
                spv::Input<spv::IOVec3>,
                &'a gfx::Texture2D,
            ),
        >,
        uv: spv::Input<spv::IOVec2>,
        albedo: Option<&'a gfx::Texture2D>,
        roughness: Option<&'a gfx::Texture2D>,
        metallic: Option<&'a gfx::Texture2D>,
        subsurface: Option<&'a gfx::Texture2D>,
        sampler: &'a gpu::Sampler,
        _discard: bool,
        defaults: &MaterialData,
    ) {
        let albedo = if let Some(albedo) = albedo {
            Some(self.set_fragment_texture(albedo, Some("u_albedo")))
        } else {
            None
        };

        let roughness = if let Some(roughness) = roughness {
            Some(self.set_fragment_texture(roughness, Some("u_roughness")))
        } else {
            None
        };

        let metallic = if let Some(metallic) = metallic {
            Some(self.set_fragment_texture(metallic, Some("u_metallic")))
        } else {
            None
        };

        let subsurface = if let Some(subsurface) = subsurface {
            Some(self.set_fragment_texture(subsurface, Some("u_subsurface")))
        } else {
            None
        };

        let normal = match normal {
            Left(v) => Left(v),
            Right((t, b, n, tex)) => {
                let tex = self.set_fragment_texture(tex, Some("u_normal_map"));
                Right((t, b, n, tex))
            }
        };

        let sampler = self.set_fragment_sampler(sampler, Some("u_sampler"));

        let b = &self.fragment;

        b.entry(spv::ShaderStage::Fragment, "main", || {
            self.world_pos.store(world_pos.load());
            self.view_pos.store(view_pos.load());
            let uv = uv.load();
            self.uv.store(uv);

            match normal {
                Left(n) => {
                    self.normal.store(n.load());
                }
                Right((t, bi, n, map)) => {
                    let tangent = t.load();
                    let bitangent = bi.load();
                    let normal = n.load();
                    let tbn = b.mat3(&tangent, &bitangent, &normal);
                    let combined = spv::combine(&map, sampler);
                    let mut sampled = spv::sample(&combined, uv).xyz();
                    sampled *= 2.0;
                    sampled -= b.vec3(&1.0, &1.0, &1.0);
                    self.normal.store(tbn * sampled);
                }
            };

            if let Some(albedo) = albedo {
                let combined = spv::combine(&albedo, sampler);
                let albedo = spv::sample(&combined, uv);
                self.albedo.store(albedo);
            } else {
                self.albedo.store(b.const_vec4(defaults.albedo));
            };

            if let Some(roughness) = roughness {
                let combined = spv::combine(&roughness, sampler);
                let roughness = spv::sample(&combined, uv).x();
                self.roughness.store(roughness);
            } else {
                self.roughness.store(b.const_float(defaults.roughness))
            };

            if let Some(metallic) = metallic {
                let combined = spv::combine(&metallic, sampler);
                let metallic = spv::sample(&combined, uv).x();
                self.metallic.store(metallic);
            } else {
                self.metallic.store(b.const_float(defaults.metallic))
            };

            if let Some(subsurface) = subsurface {
                let combined = spv::combine(&subsurface, sampler);
                let subsurface = spv::sample(&combined, uv);
                let tmp = (-1.0 / subsurface.xyz()).exp();
                self.subsurface.store(b.vec4(&tmp.x(), &tmp.y(), &tmp.z(), &subsurface.w()));
            } else {
                let subsurface = b.const_vec4(defaults.subsurface);
                let tmp = (-1.0 / subsurface.xyz()).exp();
                self.subsurface.store(b.vec4(&tmp.x(), &tmp.y(), &tmp.z(), &subsurface.w()));
            };
            
        });
    }

    /// Set a uniform buffer in the vertex shader
    pub fn set_vertex_uniform<'b, U: spv::RustStructType + bytemuck::Pod>(
        &'b mut self,
        uniform: &'a gfx::Uniform<U>,
        name: Option<&'static str>,
    ) -> spv::Uniform<U::Spv<'b>> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(uniform);
        self.vertex.uniform(2, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_vertex_storage<'b, U: spv::RustStructType + bytemuck::Pod>(
        &'b mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<U::Spv<'b>> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(storage);
        self.vertex.storage(
            // spv::StorageAccessDesc {
            //     read: true,
            //     write: false,
            //     atomic: false,
            // },
            2,
            binding,
            name,
        )
    }

    /// Set a texture in the vertex shader
    pub fn set_vertex_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::Texture<D>,
        name: Option<&'static str>,
    ) -> spv::Texture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.vertex.texture(2, binding, name)
    }

    pub fn set_vertex_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.vertex.dtexture(2, binding, name)
    }

    pub fn set_vertex_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.vertex.itexture(2, binding, name)
    }

    pub fn set_vertex_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.vertex.utexture(2, binding, name)
    }

    /// Set a sampler in the vertex shader
    pub fn set_vertex_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(sampler);
        self.vertex.sampler(2, binding, name)
    }

    /// Set a uniform buffer in the fragment shader
    pub fn set_fragment_uniform<'b, U: spv::RustStructType + bytemuck::Pod>(
        &'b self,
        uniform: &'a gfx::Uniform<U>,
        name: Option<&'static str>,
    ) -> spv::Uniform<U::Spv<'b>> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(uniform);
        self.fragment.uniform(2, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_fragment_storage<'b, U: spv::RustStructType + bytemuck::Pod>(
        &'b mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<U::Spv<'b>> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(storage);
        self.fragment.storage(
            // spv::StorageAccessDesc {
            //     read: true,
            //     write: false,
            //     atomic: false,
            // },
            2,
            binding,
            name,
        )
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::Texture<D>,
        name: Option<&'static str>,
    ) -> spv::Texture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.fragment.texture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.fragment.dtexture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.fragment.itexture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(&texture.0);
        self.fragment.utexture(2, binding, name)
    }

    /// Set a sampler in the fragment shader
    pub fn set_fragment_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let mut resources = self.resources.borrow_mut();
        let binding = resources.len() as u32;
        resources.push(sampler);
        self.fragment.sampler(2, binding, name)
    }

    /// Build a material from defalt graphics pipeline parameters
    pub fn build(self, device: &gpu::Device, cache: Option<gpu::PipelineCache>) -> Result<Material, gfx::error::ReflectedError> {
        self.build_from_info(
            device,
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE; 8],
            Some(gpu::DepthState::default()),
            cache,
        )
    }

    /// Build a material from custom graphics pipeline parameters
    pub fn build_from_info(
        self,
        device: &gpu::Device,
        rasterizer: gpu::Rasterizer,
        blend_states: &[gpu::BlendState],
        depth_state: Option<gpu::DepthState>,
        cache: Option<gpu::PipelineCache>,
    ) -> Result<Material, gfx::error::ReflectedError> {
        if blend_states.len() < 7 {
            panic!("ERROR: Attempt to build material with less than 7 blend states\nOne state must be supplied for each output write")
        }

        let graphics = gfx::ReflectedGraphics::from_builders(
            device,
            &self.vertex,
            None,
            Some(&self.fragment),
            rasterizer,
            blend_states,
            Some(gpu::DepthStencilState {
                depth: depth_state,
                ..Default::default()
            }),
            cache,
            None,
        )?;
        let resources = self.resources.into_inner();
        let set = if let Some(mut bundle) = graphics.bundle() {
            let vertex_len = resources.len();
            for (i, v) in resources.iter().enumerate() {
                // should be ok to unwrap result as any resources set should match the binding in spv
                bundle = bundle.set_resource_by_location(2, i as _, *v).unwrap();
            }

            let set0 = if vertex_len != 0 {
                match bundle.build_set(device, 2) {
                    Ok(b) => Some(b),
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    },
                }
            } else {
                None
            };

            set0
        } else {
            None
        };

        Ok(Material {
            graphics,
            camera_set_map: Arc::new(Mutex::new(HashMap::new())),
            instance_set_map: Arc::new(Mutex::new(HashMap::new())),
            set,
        })
    }
}

/// contains fragment module resources for module
pub struct Material {
    pub graphics: gfx::ReflectedGraphics,
    pub instance_set_map: Arc<Mutex<HashMap<u64, gpu::DescriptorSet>>>,
    pub camera_set_map: Arc<Mutex<HashMap<u64, gpu::DescriptorSet>>>,
    pub set: Option<gpu::DescriptorSet>,
}

impl Material {
    /// Create a default material with a single instance
    ///
    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn textured(
        device: &gpu::Device,
        albedo: &gfx::Texture2D,
        roughness: &gfx::Texture2D,
        metallic: Option<&gfx::Texture2D>,
        normal: Option<&gfx::Texture2D>,
        sampler: &gpu::Sampler,
        discard: bool,
        cache: Option<gpu::PipelineCache>,
    ) -> Result<Self, gfx::error::ReflectedError> {
        if let Some(normal) = normal {
            let mut builder = MaterialBuilder::new();
            let (world_pos, view_pos, uv, t, b, n) = builder.tbn_vertex();
            builder.textured_or_default_fragment(
                world_pos,
                view_pos,
                Right((t, b, n, normal)),
                uv,
                Some(albedo),
                Some(roughness),
                metallic,
                None,
                sampler,
                discard,
                &Default::default(),
            );
            builder.build(device, cache)
        } else {
            let mut builder = MaterialBuilder::new();
            let (world_pos, view_pos, normal, uv) = builder.default_vertex();
            builder.textured_fragment(
                world_pos, view_pos, normal, uv, albedo, roughness, metallic, None, sampler,
                discard,
            );
            builder.build(device, cache)
        }
    }

    /// Create a material with uniform parameters with a single instance
    ///
    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn uniform(
        device: &gpu::Device,
        uniform: &super::MaterialParams,
        discard: bool,
        cache: Option<gpu::PipelineCache>,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, view_pos, normal, _) = builder.default_vertex();
        builder.uniform_fragment(world_pos, view_pos, normal, uniform, discard);
        builder.build(device, cache)
    }

    /// Create a material with constant parameters with a single instance
    pub fn constant(
        device: &gpu::Device,
        constants: &super::MaterialData,
        cache: Option<gpu::PipelineCache>,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, view_pos, normal, _) = builder.default_vertex();
        builder.constant_fragment(world_pos, view_pos, normal, constants);
        builder.build(device, cache)
    }

    /// Draw all the meshes with the material into self
    pub fn pass<'a, V: gfx::Vertex>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a super::GeometryBuffer,
        camera: &'a Camera,
        meshes: impl IntoIterator<Item = (&'a gfx::Mesh<V>, &'a Instances)>,
        clear: bool,
    ) -> Result<(), gpu::Error> {
        let load = if clear {
            gpu::LoadOp::Clear
        } else {
            gpu::LoadOp::Load
        };
        let clear_color = gpu::ClearValue::ColorFloat([0.0; 4]);
        let attachments = &[
            "world_pos",
            "view_pos",
            "normal",
            "albedo",
            "roughness",
            "metallic",
            "subsurface",
            "uv",
        ];

        let (color_attachments, resolve_attachments) = if buffer.ms() {
            let mut colors = Vec::with_capacity(attachments.len());
            let mut resolves = Vec::with_capacity(attachments.len());
            for attachment in attachments {
                colors.push(gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms(attachment).unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                });
                resolves.push(gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get(attachment).unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                });
            }
            (colors, resolves)
        } else {
            let mut colors = Vec::with_capacity(attachments.len());

            for attachment in attachments {
                colors.push(gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get(attachment).unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                });
            }

            (colors, vec![])
        };

        let mut pass = encoder.graphics_pass_reflected::<V>(
            device,
            &color_attachments,
            &resolve_attachments,
            Some(gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Owned(if buffer.ms() {
                        buffer.ms_depth.as_ref().unwrap().view.clone()
                    } else {
                        buffer.depth.view.clone()
                    }),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: if clear {
                    gpu::LoadOp::Clear
                } else {
                    gpu::LoadOp::Load
                },
                store: gpu::StoreOp::Store,
            }),
            &self.graphics,
        )?;

        let mut camera_set_map = self.camera_set_map.lock().unwrap();
        let camera_set = if let Some(s) = camera_set_map.get(&camera.buffer.id()) {
            s.clone()
        } else {
            let s = match self
                .graphics
                .bundle()
                .unwrap()
                .set_resource("u_camera", camera)
                .unwrap()
                .build_set(device, 0)
            {
                Ok(s) => s,
                Err(e) => match e {
                    gfx::BundleBuildError::Gpu(e) => Err(e)?,
                    e => unreachable!("{}", e),
                },
            };
            camera_set_map.insert(camera.buffer.id(), s.clone());
            s
        };

        pass.bind_descriptor_owned(0, camera_set);

        if let Some(set) = &self.set {
            // pass.set_bundle_ref(bundle);
            pass.bind_descriptor_ref(2, set);
        }

        for (mesh, instances) in meshes {
            let mut instance_set_map = self.instance_set_map.lock().unwrap();
            let instance_set = if let Some(i) = instance_set_map.get(&instances.buffer.id()) {
                i.clone()
            } else {
                let s = match self
                    .graphics
                    .bundle()
                    .unwrap()
                    .set_resource("u_instances", instances)
                    .unwrap()
                    .build_set(device, 1)
                {
                    Ok(s) => s,
                    Err(e) => match e {
                        gfx::BundleBuildError::Gpu(e) => Err(e)?,
                        e => unreachable!("{}", e),
                    },
                };
                instance_set_map.insert(instances.buffer.id(), s.clone());
                s
            };
            pass.bind_descriptor_owned(1, instance_set);
            pass.draw_instanced_mesh_ref(mesh, 0, instances.length as _);
        }

        Ok(())
    }

    /// To avoid memory use after free issues vulkan objects are kept alive as long as they can be used
    /// Specifically references in command buffers or descriptor sets keep other objects alive until the command buffer is reset or the descriptor set is destroyed
    /// This function drops Descriptor sets cached by self
    pub fn clean(&mut self) {
        self.camera_set_map.lock().unwrap().clear();
        self.instance_set_map.lock().unwrap().clear();
    }
}
