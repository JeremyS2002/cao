use spv::prelude::*;

use std::borrow::Cow;

use crate::cone::*;

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
            albedo: glam::vec4(1.0, 1.0, 1.0, 1.0),
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
    /// resources used in the vertex shader
    vertex_resources: Vec<&'a dyn gfx::Resource>,

    /// builds the fragment module
    fragment: spv::FragmentBuilder,
    /// resources used in the fragment shader
    fragment_resources: Vec<&'a dyn gfx::Resource>,

    /// outputs required for the fragment material

    /// the position of the fragment
    pub position: spv::Output<spv::Vec3>,
    /// the normal of the fragment
    pub normal: spv::Output<spv::Vec3>,
    /// the albedo of the fragment
    pub albedo: spv::Output<spv::Vec4>,
    /// the roughness of the fragment
    pub roughness: spv::Output<spv::Float>,
    /// how metallic the fragment is
    pub metallic: spv::Output<spv::Float>,
    /// optional subsurface output
    pub subsurface: spv::Output<spv::Vec4>,
    pub uv: spv::Output<spv::Vec2>,
}

impl<'a> MaterialBuilder<'a> {
    /// Create a new MaterialBuilder
    pub fn new() -> Self {
        let vertex = spv::VertexBuilder::new();
        let fragment = spv::FragmentBuilder::new();
        let position = fragment.output(0, false, Some("out_position"));
        let normal = fragment.output(1, false, Some("out_normal"));
        let albedo = fragment.output(2, false, Some("out_albedo"));
        let roughness = fragment.output(3, false, Some("out_roughness"));
        let metallic = fragment.output(4, false, Some("out_metallic"));
        let subsurface = fragment.output(5, false, Some("out_subsurface"));
        let uv = fragment.output(6, false, Some("out_uv"));

        Self {
            vertex,
            vertex_resources: Vec::new(),

            fragment,
            fragment_resources: Vec::new(),

            position,
            normal,
            albedo,
            roughness,
            metallic,
            subsurface,
            uv,
        }
    }

    /// Creates a vertex state with a single instance
    ///
    /// The vertex builder can't be used after this function
    /// returns (in_position, in_normal, in_uv) for the fragment shader
    pub fn default_vertex(
        &mut self,
        camera: &'a Camera,
        instance: &'a crate::utils::Instance,
    ) -> (
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec2>,
    ) {
        let in_pos = self.vertex.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.vertex.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.vertex.in_vec2(2, false, Some("in_uv"));

        let out_pos = self.vertex.out_vec3(0, false, Some("out_pos"));
        let out_normal = self.vertex.out_vec3(1, false, Some("out_normal"));
        let out_uv = self.vertex.out_vec2(2, false, Some("out_uv"));

        let camera = self.set_vertex_uniform(camera, Some("u_camera"));
        let instance = self.set_vertex_uniform(instance, Some("u_instance"));

        let out_position = self.vertex.position();

        self.vertex.main(|b| {
            let projection = b.load_uniform_field::<_, spv::Mat4>(camera, "projection");
            let view = b.load_uniform_field::<_, spv::Mat4>(camera, "view");
            let model = b.load_uniform_field::<_, spv::Mat4>(instance, "model");
            let in_pos = b.load_in(in_pos);
            let x = in_pos.x(b);
            let y = in_pos.y(b);
            let z = in_pos.z(b);
            let in_pos4 = b.vec4(&x, &y, &z, &1.0);
            let world_pos = b.mul(model, in_pos4);
            let camera_pos = b.mul(view, world_pos);
            let screen_pos = b.mul(projection, camera_pos);
            b.store_out(out_position, screen_pos);
            b.store_out(out_pos, world_pos.xyz(b));
            let in_norm = b.load_in(in_normal);
            let model_x = b.mat_col(model, 0).xyz(b);
            let model_y = b.mat_col(model, 1).xyz(b);
            let model_z = b.mat_col(model, 2).xyz(b);
            let model3 = b.mat3(&model_x, &model_y, &model_z);
            let out_norm = b.mul(model3, in_norm);
            b.store_out(out_normal, out_norm);

            b.store_out(out_uv, b.load_in(in_uv));
        });

        let in_pos = self.fragment.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.fragment.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.fragment.in_vec2(2, false, Some("in_uv"));

        (in_pos, in_normal, in_uv)
    }

    /// Creates a vertex state with multiple instances
    ///
    /// The vertex builder can't be used by this function
    /// returns (in_position, in_normal, in_uv) for the fragment shader
    pub fn instanced_vertex(
        &mut self,
        camera: &'a Camera,
        instances: &'a crate::utils::Instances,
    ) -> (
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec3>,
        spv::Input<spv::Vec2>,
    ) {
        let in_pos = self.vertex.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.vertex.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.vertex.in_vec2(2, false, Some("in_uv"));

        let out_pos = self.vertex.out_vec3(0, false, Some("out_pos"));
        let out_normal = self.vertex.out_vec3(1, false, Some("out_normal"));
        let out_uv = self.vertex.out_vec2(2, false, Some("out_uv"));

        let camera = self.set_vertex_uniform(&camera, Some("u_camera"));
        let instances = self.set_vertex_storage(&instances, Some("u_instance_buffer"));

        let instance_idx = self.vertex.instance_id();

        let out_position = self.vertex.position();

        self.vertex.main(|b| {
            let projection = b.load_uniform_field::<_, spv::Mat4>(camera, "projection");
            let view = b.load_uniform_field::<_, spv::Mat4>(camera, "view");
            let idx = b.load_in(instance_idx);
            let u_idx = b.convert::<spv::UInt>(idx);
            let model: spv::Mat4 = b.load_storage_element_field(instances, &u_idx, "model");

            let in_pos = b.load_in(in_pos);
            let x = in_pos.x(b);
            let y = in_pos.y(b);
            let z = in_pos.z(b);
            let world_pos = b.mul(model, b.vec4(&x, &y, &z, &1.0));
            let view_pos = b.mul(view, world_pos);
            let screen_pos = b.mul(projection, view_pos);
            b.store_out(out_pos, world_pos.xyz(b));
            b.store_out(out_position, screen_pos);

            let in_norm = b.load_in(in_normal);
            let model_x = b.mat_col(model, 0).xyz(b);
            let model_y = b.mat_col(model, 1).xyz(b);
            let model_z = b.mat_col(model, 2).xyz(b);
            let model3 = b.mat3(&model_x, &model_y, &model_z);
            let out_norm = b.mul(model3, in_norm);
            b.store_out(out_normal, out_norm);

            let uv = b.load_in(in_uv);
            b.store_out(out_uv, uv);
        });

        let in_pos = self.fragment.in_vec3(0, false, Some("in_pos"));
        let in_normal = self.fragment.in_vec3(1, false, Some("in_normal"));
        let in_uv = self.fragment.in_vec2(2, false, Some("in_uv"));

        (in_pos, in_normal, in_uv)
    }

    /// Returns a vertex shader with a single instance
    ///
    /// The vertex builder can't be used after this function
    /// returns (in_position, in_uv, in_t, in_b, in_n) for the fragment shader
    pub fn tbn_vertex(
        &mut self,
        camera: &'a Camera,
        instance: &'a crate::utils::Instance,
    ) -> (
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

        let out_pos = self.vertex.out_vec3(0, false, Some("out_pos"));
        let out_uv = self.vertex.out_vec2(1, false, Some("out_uv"));
        let out_t = self.vertex.out_vec3(2, false, Some("out_t"));
        let out_b = self.vertex.out_vec3(3, false, Some("out_b"));
        let out_n = self.vertex.out_vec3(4, false, Some("out_n"));

        let camera = self.set_vertex_uniform(camera, Some("u_camera"));
        let instance = self.set_vertex_uniform(instance, Some("u_instance"));

        let vk_position = self.vertex.position();

        self.vertex.main(|b| {
            let projection: spv::Mat4 = b.load_uniform_field(camera, "projection");
            let view: spv::Mat4 = b.load_uniform_field(camera, "view");
            let model: spv::Mat4 = b.load_uniform_field(instance, "model");
            let pos = b.load_in(in_pos);
            let (x, y, z) = (pos.x(b), pos.y(b), pos.z(b));
            let world_pos = b.mul(model, b.vec4(&x, &y, &z, &1.0));
            let view_pos = b.mul(view, world_pos);
            let screen_pos = b.mul(projection, view_pos);
            b.store_out(vk_position, screen_pos);
            b.store_out(out_pos, world_pos.xyz(b));

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

        let in_pos = self.fragment.in_vec3(0, false, Some("in_pos"));
        let in_uv = self.fragment.in_vec2(1, false, Some("in_uv"));
        let in_t = self.fragment.in_vec3(2, false, Some("in_t"));
        let in_b = self.fragment.in_vec3(3, false, Some("in_b"));
        let in_n = self.fragment.in_vec3(4, false, Some("in_n"));

        (in_pos, in_uv, in_t, in_b, in_n)
    }

    /// Set the outputs to sample from the textures
    ///
    /// The fragment builder can't be used after this function
    pub fn textured_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
        normal: spv::Input<spv::Vec3>,
        uv: spv::Input<spv::Vec2>,
        albedo: &'a gfx::Texture2D,
        roughness: &'a gfx::Texture2D,
        metallic: Option<&'a gfx::Texture2D>,
        subsurface: Option<&'a gfx::Texture2D>,
        sampler: &'a gpu::Sampler,
    ) {
        self.textured_or_default_fragment(
            world_pos,
            Left(normal),
            uv,
            Some(albedo),
            Some(roughness),
            metallic,
            subsurface,
            sampler,
            &MaterialData::default(),
        )
    }

    /// Set the outputs to read from the uniform buffer
    ///
    /// The fragment builder can't be used after this function
    pub fn uniform_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
        normal: spv::Input<spv::Vec3>,
        uniform: &'a super::MaterialParams,
    ) {
        let params = self.set_fragment_uniform(&uniform, Some("u_params"));

        self.fragment.main(|b| {
            let albedo: spv::Vec4 = b.load_uniform_field(params, "albedo");
            let roughness: spv::Float = b.load_uniform_field(params, "metallic");
            let metallic: spv::Float = b.load_uniform_field(params, "metallic");
            let mut subsurface: spv::Vec4 = b.load_uniform_field(params, "subsurface");
            let mut tmp = subsurface.xyz(b);
            tmp = b.div(-1.0f32, tmp).exp(b);
            subsurface = b.vec4(&tmp.x(b), &tmp.y(b), &tmp.z(b), &subsurface.z(b));

            b.store_out(self.position, b.load_in(world_pos));
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
        normal: spv::Input<spv::Vec3>,
        constants: &MaterialData,
    ) {
        let mut tmp = constants.subsurface.xyz();
        tmp = (-1.0 / tmp).exp();
        let subsurface = glam::vec4(tmp.x, tmp.y, tmp.z, constants.subsurface.w);
        self.fragment.main(|b| {
            b.store_out(self.position, b.load_in(world_pos));
            b.store_out(self.normal, b.load_in(normal));
            b.store_out(self.albedo, constants.albedo);
            b.store_out(self.roughness, constants.roughness);
            b.store_out(self.metallic, constants.metallic);
            b.store_out(self.subsurface, subsurface);
            b.store_out(self.uv, b.vec2(&0.0, &0.0));
        });
    }

    pub fn textured_or_default_fragment(
        &mut self,
        world_pos: spv::Input<spv::Vec3>,
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
            b.store_out(self.position, b.load_in(world_pos));
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
                b.store_out(
                    self.albedo,
                    b.sample_texture(b.combine_texture_sampler(albedo, sampler), uv),
                );
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
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(uniform);
        self.vertex.uniform(0, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_vertex_storage<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<spv::Struct<U>> {
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(storage);
        self.vertex.storage(
            spv::StorageAccessDesc {
                read: true,
                write: false,
                atomic: false,
            },
            1,
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
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(&texture.0);
        self.vertex
            .texture(0, binding, name)
    }

    pub fn set_vertex_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(&texture.0);
        self.vertex
            .d_texture(0, binding, name)
    }

    pub fn set_vertex_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(&texture.0);
        self.vertex
            .i_texture(0, binding, name)
    }

    pub fn set_vertex_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(&texture.0);
        self.vertex
            .u_texture(0, binding, name)
    }

    /// Set a sampler in the vertex shader
    pub fn set_vertex_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let binding = self.vertex_resources.len() as u32;
        self.vertex_resources.push(sampler);
        self.vertex.sampler(0, binding, name)
    }

    /// Set a uniform buffer in the fragment shader
    pub fn set_fragment_uniform<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        uniform: &'a gfx::Uniform<U>,
        name: Option<&'static str>,
    ) -> spv::Uniform<spv::Struct<U>> {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(uniform);
        self.fragment.uniform(1, binding, name)
    }

    /// Set a storage buffer in the fragment shader
    pub fn set_fragment_storage<U: spv::AsSpvStruct + bytemuck::Pod>(
        &mut self,
        storage: &'a gfx::Storage<U>,
        name: Option<&'static str>,
    ) -> spv::Storage<spv::Struct<U>> {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(storage);
        self.fragment.storage(
            spv::StorageAccessDesc {
                read: true,
                write: false,
                atomic: false,
            },
            1,
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
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(&texture.0);
        self.fragment
            .texture(1, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_d_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::DTexture<D>,
        name: Option<&'static str>,
    ) -> spv::DTexture<D::Spirv> {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(&texture.0);
        self.fragment
            .d_texture(1, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_i_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::ITexture<D>,
        name: Option<&'static str>,
    ) -> spv::ITexture<D::Spirv> {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(&texture.0);
        self.fragment
            .i_texture(1, binding, name)
    }

    /// Set a texture in the fragment shader
    pub fn set_fragment_u_texture<D: gfx::AsDimension>(
        &mut self,
        texture: &'a gfx::UTexture<D>,
        name: Option<&'static str>,
    ) -> spv::UTexture<D::Spirv> {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(&texture.0);
        self.fragment
            .u_texture(1, binding, name)
    }

    /// Set a sampler in the fragment shader
    pub fn set_fragment_sampler(
        &mut self,
        sampler: &'a gpu::Sampler,
        name: Option<&'static str>,
    ) -> spv::Sampler {
        let binding = self.fragment_resources.len() as u32;
        self.fragment_resources.push(sampler);
        self.fragment.sampler(1, binding, name)
    }

    /// Build a material from defalt graphics pipeline parameters
    pub fn build(self, device: &gpu::Device) -> Result<Material, gfx::error::ReflectedError> {
        self.build_from_info(
            device,
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE; 7],
            Some(gpu::DepthState::default()),
        )
    }

    /// Build a material from custom graphics pipeline parameters
    ///
    /// If there are less than the number of blend states required then this function will panic
    /// 5 states are required by default
    /// If using subsurface then another is required
    pub fn build_from_info(
        self,
        device: &gpu::Device,
        rasterizer: gpu::Rasterizer,
        blend_states: &[gpu::BlendState],
        depth_state: Option<gpu::DepthState>,
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
            None,
        )?;
        let bundle = if let Some(mut bundle) = graphics.bundle() {
            for (i, v) in self.vertex_resources.into_iter().enumerate() {
                // should be ok to unwrap result as any resources set should match the binding in spv
                bundle = bundle.set_resource_by_location(0, i as _, v).unwrap();
            }

            for (i, f) in self.fragment_resources.into_iter().enumerate() {
                bundle = bundle.set_resource_by_location(1, i as _, f).unwrap()
            }

            Some(bundle.build(device)?)
        } else {
            None
        };

        Ok(Material { graphics, bundle })
    }
}

/// contains fragment module resources for module
pub struct Material {
    pub graphics: gfx::ReflectedGraphics,
    pub bundle: Option<gfx::Bundle>,
}

impl Material {
    /// Create a default material with a single instance
    pub fn textured(
        device: &gpu::Device,
        camera: &Camera,
        instance: &crate::utils::Instance,
        albedo: &gfx::Texture2D,
        roughness: &gfx::Texture2D,
        metallic: Option<&gfx::Texture2D>,
        normal: Option<&gfx::Texture2D>,
        sampler: &gpu::Sampler,
    ) -> Result<Self, gfx::error::ReflectedError> {
        if let Some(normal) = normal {
            let mut builder = MaterialBuilder::new();
            let (world_pos, uv, t, b, n) = builder.tbn_vertex(camera, instance);
            builder.textured_or_default_fragment(
                world_pos,
                Right((t, b, n, normal)),
                uv,
                Some(albedo),
                Some(roughness),
                metallic,
                None,
                sampler,
                &Default::default(),
            );
            builder.build(device)
        } else {
            let mut builder = MaterialBuilder::new();
            let (world_pos, normal, uv) = builder.default_vertex(camera, instance);
            builder.textured_fragment(
                world_pos, normal, uv, albedo, roughness, metallic, None, sampler,
            );
            builder.build(device)
        }
    }

    /// Create a default material with multiple instances
    pub fn instanced_textured(
        device: &gpu::Device,
        camera: &Camera,
        instances: &crate::utils::Instances,
        albedo: &gfx::Texture2D,
        roughness: &gfx::Texture2D,
        metallic: Option<&gfx::Texture2D>,
        sampler: &gpu::Sampler,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, normal, uv) = builder.instanced_vertex(camera, instances);
        builder.textured_fragment(
            world_pos, normal, uv, albedo, roughness, metallic, None, sampler,
        );
        builder.build(device)
    }

    /// Create a material with uniform parameters with a single instance
    pub fn uniform(
        device: &gpu::Device,
        camera: &Camera,
        instance: &crate::utils::Instance,
        uniform: &super::MaterialParams,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, normal, _) = builder.default_vertex(camera, instance);
        builder.uniform_fragment(world_pos, normal, uniform);
        builder.build(device)
    }

    /// Create a material with uniform parameters with multiple instances
    pub fn uniform_instanced(
        device: &gpu::Device,
        camera: &Camera,
        instances: &crate::utils::Instances,
        uniform: &super::MaterialParams,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, normal, _) = builder.instanced_vertex(camera, instances);
        builder.uniform_fragment(world_pos, normal, uniform);
        builder.build(device)
    }

    /// Create a material with constant parameters with a single instance
    pub fn constant(
        device: &gpu::Device,
        camera: &Camera,
        instance: &crate::utils::Instance,
        constants: &super::MaterialData,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, normal, _) = builder.default_vertex(camera, instance);
        builder.constant_fragment(world_pos, normal, constants);
        builder.build(device)
    }

    /// Create a material with constant parameters with multiple instances
    pub fn constant_instanced(
        device: &gpu::Device,
        camera: &Camera,
        instances: &crate::utils::Instances,
        constants: &super::MaterialData,
    ) -> Result<Self, gfx::error::ReflectedError> {
        let mut builder = MaterialBuilder::new();
        let (world_pos, normal, _) = builder.instanced_vertex(camera, instances);
        builder.constant_fragment(world_pos, normal, constants);
        builder.build(device)
    }

    /// Draw all the meshes with the material into self
    pub fn pass<'a, 'b, V: gfx::Vertex>(
        &'a self,
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        buffer: &'a super::GeometryBuffer,
        meshes: impl IntoIterator<Item = &'b dyn gfx::Mesh<V>>,
        clear: bool,
    ) -> Result<(), gfx::error::ReflectedError> {
        let load = if clear {
            gpu::LoadOp::Clear
        } else {
            gpu::LoadOp::Load
        };
        let clear_color = gpu::ClearValue::ColorFloat([0.0; 4]);
        let (color_attachments, resolve_attachments) = if buffer.ms() {
            let color = vec![
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("position").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("normal").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("albedo").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("roughness").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("metallic").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("subsurface").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get_ms("uv").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::DontCare,
                },
            ];
            let resolve = vec![
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("position").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("normal").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("albedo").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("roughness").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("metallic").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("subsurface").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("uv").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                }
            ];
            (color, resolve)
        } else {
            let color = vec![
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("position").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("normal").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("albedo").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("roughness").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("metallic").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("subsurface").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Borrowed(&buffer.get("uv").unwrap().view),
                        clear_color,
                    ),
                    load,
                    store: gpu::StoreOp::Store,
                },
            ];

            (color, vec![])
        };
        let mut pass = encoder.graphics_pass_reflected(
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

        if let Some(bundle) = &self.bundle {
            pass.set_bundle_ref(bundle);
        }

        for mesh in meshes {
            pass.draw_mesh_owned(mesh);
        }

        Ok(())
    }
}
