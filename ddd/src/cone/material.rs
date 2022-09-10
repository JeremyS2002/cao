use gfx::GraphicsPass;
use spv::prelude::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::{borrow::Cow, collections::HashMap};

use crate::utils::*;

use either::*;
use glam::Vec4Swizzles;

pub type MaterialParams = gfx::Uniform<MaterialData>;

/// Used to send default values when not sampling from textures
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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

unsafe impl spv::AsSpvStruct for MaterialData {
    const DESC: spv::StructDesc = spv::StructDesc {
        name: "MaterialData",
        names: &["albedo", "subsurface", "roughness", "metallic"],
        fields: &[
            spv::DataType::Primitive(spv::PrimitiveType::Vec4),
            spv::DataType::Primitive(spv::PrimitiveType::Vec4),
            spv::DataType::Primitive(spv::PrimitiveType::Float),
            spv::DataType::Primitive(spv::PrimitiveType::Float),
        ],
    };

    fn fields<'a>(&'a self) -> Vec<&'a dyn spv::AsData> {
        vec![
            &self.albedo,
            &self.subsurface,
            &self.roughness,
            &self.metallic,
        ]
    }
}

/// Builds a Materials shader modules as well a bundle
pub struct MaterialBuilder<'a> {
    /// builds the vertex module
    vertex: spv::VertexBuilder,
    /// builds the fragment module
    fragment: spv::FragmentBuilder,
    /// resources used in the shaders
    resources: Vec<&'a dyn gfx::Resource>,

    /// outputs required for the fragment material

    /// the position of the fragment in world space
    pub world_pos: spv::Output<spv::Vec3>,
    /// the position of the fragment in view space
    pub view_pos: spv::Output<spv::Vec3>,
    /// the normal of the fragment in world space
    pub normal: spv::Output<spv::Vec3>,
    /// the albedo of the fragment
    pub albedo: spv::Output<spv::Vec4>,
    /// the roughness of the fragment
    pub roughness: spv::Output<spv::Float>,
    /// how metallic the fragment is
    pub metallic: spv::Output<spv::Float>,
    /// optional subsurface output
    pub subsurface: spv::Output<spv::Vec4>,
    /// the uv coordinate at that point
    pub uv: spv::Output<spv::Vec2>,
}

impl<'a> MaterialBuilder<'a> {
    /// Create a new MaterialBuilder
    pub fn new() -> Self {
        let vertex = spv::VertexBuilder::new();
        let fragment = spv::FragmentBuilder::new();
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
            resources: Vec::new(),

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
    pub fn camera(&mut self) -> spv::Uniform<spv::Struct<CameraData>> {
        self.vertex
            .uniform_struct::<CameraData>(0, 0, Some("u_camera"))
    }

    pub fn instances(&mut self) -> spv::Storage<spv::Struct<InstanceData>> {
        self.vertex.storage::<spv::Struct<InstanceData>>(
            spv::StorageAccessDesc {
                read: true,
                write: false,
                atomic: false,
            },
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
        &mut self,
    ) -> (
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec2>,
    ) {
        let in_pos = self.vertex.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.vertex.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.vertex.in_vec2(2, false, Some("in_uv"));

        let out_world_pos = self.vertex.out_vec3(0, false, Some("out_world_pos"));
        let out_view_pos = self.vertex.out_vec3(1, false, Some("out_view_pos"));
        let out_normal = self.vertex.out_vec3(2, false, Some("out_normal"));
        let out_uv = self.vertex.out_vec2(3, false, Some("out_uv"));

        let camera = self.camera();
        let instances = self.instances();

        let instance_idx = self.vertex.instance_index();

        let vk_position = self.vertex.position();

        self.vertex.main(|b| {
            let projection = b.load_uniform_field::<_, spv::Mat4>(camera, "projection");
            let view = b.load_uniform_field::<_, spv::Mat4>(camera, "view");
            let idx = b.load_in(instance_idx);
            let u_idx = b.convert::<spv::UInt>(idx);
            let model = b.load_storage_element_field::<_, spv::Mat4>(instances, &u_idx, "model");
            //let model = b.load_storage_field::<_, spv::Mat4>(instance, "model");
            let in_pos = b.load_in(in_pos);
            let x = in_pos.x(b);
            let y = in_pos.y(b);
            let z = in_pos.z(b);
            let in_pos4 = b.vec4(&x, &y, &z, &1.0);
            let world_pos = b.mul(model, in_pos4);
            b.store_out(out_world_pos, world_pos.xyz(b));
            let view_pos = b.mul(view, world_pos);
            b.store_out(out_view_pos, view_pos.xyz(b));
            let screen_pos = b.mul(projection, view_pos);
            b.store_out(vk_position, screen_pos);
            let in_norm = b.load_in(in_normal);
            let model_x = b.mat_col(model, 0).xyz(b);
            let model_y = b.mat_col(model, 1).xyz(b);
            let model_z = b.mat_col(model, 2).xyz(b);
            let model3 = b.mat3(&model_x, &model_y, &model_z);
            let mut out_norm = b.mul(model3, in_norm);
            out_norm = out_norm.normalize(b);
            b.store_out(out_normal, out_norm);

            b.store_out(out_uv, b.load_in(in_uv));
        });

        let in_world_pos = self.fragment.in_vec3(0, false, Some("in_pos"));
        let in_view_pos = self.fragment.in_vec3(1, false, Some("in_view_pos"));
        let in_normal = self.fragment.in_vec3(2, false, Some("in_normal"));
        let in_uv = self.fragment.in_vec2(3, false, Some("in_uv"));

        (in_world_pos, in_view_pos, in_normal, in_uv)
    }

    /// Returns a vertex shader with a single instance
    ///
    /// The vertex builder can't be used after this function
    /// returns (in_world_pos, in_view_pos, in_uv, in_t, in_b, in_n) for the fragment shader
    pub fn tbn_vertex(
        &mut self,
    ) -> (
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec2>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
    ) {
        let in_pos = self.vertex.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.vertex.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.vertex.in_vec2(2, false, Some("in_uv"));
        let in_tangent = self.vertex.in_vec3(3, false, Some("in_tangent"));

        let out_world_pos = self.vertex.out_vec3(0, false, Some("out_world_pos"));
        let out_view_pos = self.vertex.out_vec3(1, false, Some("out_view_pos"));
        let out_uv = self.vertex.out_vec2(2, false, Some("out_uv"));
        let out_t = self.vertex.out_vec3(3, false, Some("out_t"));
        let out_b = self.vertex.out_vec3(4, false, Some("out_b"));
        let out_n = self.vertex.out_vec3(5, false, Some("out_n"));

        let camera = self.camera();
        let instances = self.instances();

        let instance_idx = self.vertex.instance_index();

        let vk_position = self.vertex.position();

        self.vertex.main(|b| {
            let projection: spv::Mat4 = b.load_uniform_field(camera, "projection");
            let view: spv::Mat4 = b.load_uniform_field(camera, "view");
            //let model: spv::Mat4 = b.load_uniform_field(instance, "model");
            let idx = b.load_in(instance_idx);
            let u_idx = b.convert::<spv::UInt>(idx);
            let model = b.load_storage_element_field::<_, spv::Mat4>(instances, &u_idx, "model");
            let pos = b.load_in(in_pos);
            let (x, y, z) = (pos.x(b), pos.y(b), pos.z(b));
            let world_pos = b.mul(model, b.vec4(&x, &y, &z, &1.0));
            b.store_out(out_world_pos, world_pos.xyz(b));
            let view_pos = b.mul(view, world_pos);
            b.store_out(out_view_pos, view_pos.xyz(b));
            let screen_pos = b.mul(projection, view_pos);
            b.store_out(vk_position, screen_pos);

            let uv = b.load_in(in_uv);
            b.store_out(out_uv, uv);

            let mut t = b.load_in(in_tangent);
            t = b
                .mul(model, b.vec4(&t.x(b), &t.y(b), &t.z(b), &0.0))
                .xzy(b)
                .normalize(b);
            let mut n = b.load_in(in_normal);
            n = b
                .mul(model, b.vec4(&n.x(b), &n.y(b), &n.z(b), &0.0))
                .xyz(b)
                .normalize(b);
            t = b.sub(t, b.mul(t.dot(&n, b), n)).normalize(b);
            let bi = n.cross(&t, b);

            b.store_out(out_t, t);
            b.store_out(out_b, bi);
            b.store_out(out_n, n);
        });

        let in_world_pos = self.fragment.in_vec3(0, false, Some("in_world_pos"));
        let in_view_pos = self.fragment.in_vec3(1, false, Some("in_view_pos"));
        let in_uv = self.fragment.in_vec2(2, false, Some("in_uv"));
        let in_t = self.fragment.in_vec3(3, false, Some("in_t"));
        let in_b = self.fragment.in_vec3(4, false, Some("in_b"));
        let in_n = self.fragment.in_vec3(5, false, Some("in_n"));

        (in_world_pos, in_view_pos, in_uv, in_t, in_b, in_n)
    }

    /// Set the outputs to sample from the textures
    ///
    /// The fragment builder can't be used after this function
    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn textured_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
        view_pos: spv::Input<spv::Vec3>,
        normal: spv::Input<spv::Vec3>,
        uv: spv::Input<spv::Vec2>,
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
        world_pos: spv::Input<spv::Vec3>,
        view_pos: spv::Input<spv::Vec3>,
        normal: spv::Input<spv::Vec3>,
        uniform: &'a super::MaterialParams,
        discard: bool,
    ) {
        let params = self.set_fragment_uniform(&uniform, Some("u_params"));

        self.fragment.main(|b| {
            let albedo: spv::Vec4 = b.load_uniform_field(params, "albedo");
            if discard {
                let a = albedo.w(b);
                let bl = b.eq(&a, &0.0f32);
                b.spv_if(bl, |b| {
                    b.discard();
                });
            }
            let roughness: spv::Float = b.load_uniform_field(params, "metallic");
            let metallic: spv::Float = b.load_uniform_field(params, "metallic");
            let mut subsurface: spv::Vec4 = b.load_uniform_field(params, "subsurface");
            let mut tmp = subsurface.xyz(b);
            tmp = b.div(-1.0f32, tmp).exp(b);
            subsurface = b.vec4(&tmp.x(b), &tmp.y(b), &tmp.z(b), &subsurface.z(b));

            b.store_out(self.world_pos, b.load_in(world_pos));
            b.store_out(self.view_pos, b.load_in(view_pos));
            b.store_out(self.normal, b.load_in(normal));
            b.store_out(self.albedo, albedo);
            b.store_out(self.roughness, roughness);
            b.store_out(self.metallic, metallic);
            b.store_out(self.subsurface, subsurface);
            b.store_out(self.uv, b.vec2(&0.0, &0.0));
        });
    }

    /// Set the output to be constant values
    ///
    /// The fragment builder can't be used after this function
    pub fn constant_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
        view_pos: spv::Input<spv::Vec3>,
        normal: spv::Input<spv::Vec3>,
        constants: &MaterialData,
    ) {
        let mut tmp = constants.subsurface.xyz();
        tmp = (-1.0 / tmp).exp();
        let subsurface = glam::vec4(tmp.x, tmp.y, tmp.z, constants.subsurface.w);
        self.fragment.main(|b| {
            b.store_out(self.world_pos, b.load_in(world_pos));
            b.store_out(self.view_pos, b.load_in(view_pos));
            b.store_out(self.normal, b.load_in(normal));
            b.store_out(self.albedo, constants.albedo);
            b.store_out(self.roughness, constants.roughness);
            b.store_out(self.metallic, constants.metallic);
            b.store_out(self.subsurface, subsurface);
            b.store_out(self.uv, b.vec2(&0.0, &0.0));
        });
    }

    /// if discard then if the albedo alpha channel is 0.0 the fragment will be discarded
    pub fn textured_or_default_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
        view_pos: spv::Input<spv::Vec3>,
        normal: Either<
            spv::Input<spv::Vec3>,
            (
                spv::Input<spv::Vec3>,
                spv::Input<spv::Vec3>,
                spv::Input<spv::Vec3>,
                &'a gfx::Texture2D,
            ),
        >,
        uv: spv::Input<spv::Vec2>,
        albedo: Option<&'a gfx::Texture2D>,
        roughness: Option<&'a gfx::Texture2D>,
        metallic: Option<&'a gfx::Texture2D>,
        subsurface: Option<&'a gfx::Texture2D>,
        sampler: &'a gpu::Sampler,
        discard: bool,
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

        self.fragment.main(|b| {
            b.store_out(self.world_pos, b.load_in(world_pos));
            b.store_out(self.view_pos, b.load_in(view_pos));
            let uv = b.load_in(uv);
            b.store_out(self.uv, uv);

            match normal {
                Left(n) => {
                    b.store_out(self.normal, b.load_in(n));
                }
                Right((t, bi, n, map)) => {
                    let tbn = b.mat3(&b.load_in(t), &b.load_in(bi), &b.load_in(n));
                    let mut normal = b
                        .sample_texture(b.combine_texture_sampler(map, sampler), uv)
                        .xyz(b);
                    b.mul_assign(&mut normal, 2.0);
                    b.sub_assign(&mut normal, b.vec3(&1.0, &1.0, &1.0));
                    b.mul_assign(&mut normal, tbn);
                    b.store_out(self.normal, normal.normalize(b));
                }
            }

            if let Some(albedo) = albedo {
                let albedo = b.sample_texture(b.combine_texture_sampler(albedo, sampler), uv);
                if discard {
                    let a = albedo.w(b);
                    let bl = b.eq(&a, &0.0f32);
                    b.spv_if(bl, |b| {
                        b.discard();
                    });
                }
                b.store_out(self.albedo, albedo);
            } else {
                b.store_out(self.albedo, defaults.albedo);
            }

            if let Some(roughness) = roughness {
                b.store_out(
                    self.roughness,
                    b.sample_texture(b.combine_texture_sampler(roughness, sampler), uv)
                        .x(b),
                );
            } else {
                b.store_out(self.roughness, defaults.roughness);
            }

            if let Some(metallic) = metallic {
                b.store_out(
                    self.metallic,
                    b.sample_texture(b.combine_texture_sampler(metallic, sampler), uv)
                        .x(b),
                );
            } else {
                b.store_out(self.metallic, defaults.metallic);
            }

            if let Some(subsurface) = subsurface {
                let mut subsurface: spv::Vec4 =
                    b.sample_texture(b.combine_texture_sampler(subsurface, sampler), uv);
                let mut tmp = subsurface.xyz(b);
                tmp = b.div(-1.0f32, tmp).exp(b);
                subsurface = b.vec4(&tmp.x(b), &tmp.y(b), &tmp.z(b), &subsurface.z(b));
                b.store_out(self.subsurface, subsurface);
            } else {
                let mut tmp = defaults.subsurface.xyz();
                tmp = (-1.0 / tmp).exp();
                let subsurface = glam::vec4(tmp.x, tmp.y, tmp.z, defaults.subsurface.w);
                b.store_out(self.subsurface, subsurface);
            }
        });
    }

    /// Set a uniform buffer in the vertex shader
    pub fn set_vertex_uniform<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        uniform: &'a gfx::Uniform<U>,
        name: Option<&'static str>,
    ) -> spv::Uniform<spv::Struct<U>> {
        let binding = self.resources.len() as u32;
        self.resources.push(uniform);
        self.vertex.uniform(2, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_vertex_storage<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<spv::Struct<U>> {
        let binding = self.resources.len() as u32;
        self.resources.push(storage);
        self.vertex.storage(
            spv::StorageAccessDesc {
                read: true,
                write: false,
                atomic: false,
            },
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
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.vertex.texture(2, binding, name)
    }

    pub fn set_vertex_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.vertex.d_texture(2, binding, name)
    }

    pub fn set_vertex_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.vertex.i_texture(2, binding, name)
    }

    pub fn set_vertex_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.vertex.u_texture(2, binding, name)
    }

    /// Set a sampler in the vertex shader
    pub fn set_vertex_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let binding = self.resources.len() as u32;
        self.resources.push(sampler);
        self.vertex.sampler(2, binding, name)
    }

    /// Set a uniform buffer in the fragment shader
    pub fn set_fragment_uniform<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        uniform: &'a gfx::Uniform<U>,
        name: Option<&'static str>,
    ) -> spv::Uniform<spv::Struct<U>> {
        let binding = self.resources.len() as u32;
        self.resources.push(uniform);
        self.fragment.uniform(2, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_fragment_storage<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<spv::Struct<U>> {
        let binding = self.resources.len() as u32;
        self.resources.push(storage);
        self.fragment.storage(
            spv::StorageAccessDesc {
                read: true,
                write: false,
                atomic: false,
            },
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
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.fragment.texture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.fragment.d_texture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.fragment.i_texture(2, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let binding = self.resources.len() as u32;
        self.resources.push(&texture.0);
        self.fragment.u_texture(2, binding, name)
    }

    /// Set a sampler in the fragment shader
    pub fn set_fragment_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let binding = self.resources.len() as u32;
        self.resources.push(sampler);
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
        let set = if let Some(mut bundle) = graphics.bundle() {
            let vertex_len = self.resources.len();
            for (i, v) in self.resources.into_iter().enumerate() {
                // should be ok to unwrap result as any resources set should match the binding in spv
                bundle = bundle.set_resource_by_location(2, i as _, v).unwrap();
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
