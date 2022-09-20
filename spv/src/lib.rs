
pub use either;
use either::*;

use std::sync::Arc;
use std::sync::Mutex;

pub mod data;
pub mod instruction;
pub mod io;
pub mod builder;
pub mod func;
pub mod scope;
pub mod bindings;

pub use data::*;
pub use instruction::*;
pub use io::*;
pub use builder::*;
pub use func::*;
pub use scope::*;
pub use bindings::*;

pub use glam::IVec2 as GlamIVec2;
pub use glam::IVec3 as GlamIVec3;
pub use glam::IVec4 as GlamIVec4;
pub use glam::UVec2 as GlamUVec2;
pub use glam::UVec3 as GlamUVec3;
pub use glam::UVec4 as GlamUVec4;
pub use glam::Vec2 as GlamVec2;
pub use glam::Vec3 as GlamVec3;
pub use glam::Vec4 as GlamVec4;
pub use glam::DVec2 as GlamDVec2;
pub use glam::DVec3 as GlamDVec3;
pub use glam::DVec4 as GlamDVec4;
pub use glam::Mat2 as GlamMat2;
pub use glam::Mat3 as GlamMat3;
pub use glam::Mat4 as GlamMat4;
pub use glam::DMat2 as GlamDMat2;
pub use glam::DMat3 as GlamDMat3;
pub use glam::DMat4 as GlamDMat4;

pub use spv_derive::AsStructType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    TessellationEval,
    TessellationControl,
    Geometry,
    Fragment,
    Compute,
}

impl ShaderStage {
    pub(crate) fn specialize(&self, b: &mut RSpirvBuilder, spv_fn: u32) {
        match self {
            ShaderStage::Fragment => {
                b.execution_mode(spv_fn, rspirv::spirv::ExecutionMode::OriginUpperLeft, &[]);
            },
            _ => (),
        }
    }

    pub(crate) fn rspirv(&self) -> rspirv::spirv::ExecutionModel {
        match self {
            ShaderStage::Vertex => rspirv::spirv::ExecutionModel::Vertex,
            ShaderStage::TessellationEval => rspirv::spirv::ExecutionModel::TessellationEvaluation,
            ShaderStage::TessellationControl => rspirv::spirv::ExecutionModel::TessellationControl,
            ShaderStage::Geometry => rspirv::spirv::ExecutionModel::Geometry,
            ShaderStage::Fragment => rspirv::spirv::ExecutionModel::Fragment,
            ShaderStage::Compute => rspirv::spirv::ExecutionModel::GLCompute,
        }
    }
}

pub struct Builder {
    inner: Arc<Mutex<BuilderInner>>,
}

impl Builder {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(BuilderInner::new())) }
    }

    pub fn compile(&self) -> Vec<u32> {
        self.inner.lock().unwrap().compile()
    }

    pub fn __inner<'a>(&'a self) -> &'a Arc<Mutex<BuilderInner>> {
        &self.inner
    }

    pub fn get_entry_name(&self, entry: ShaderStage) -> Option<&'static str> {
        let inner = self.inner.lock().unwrap();
        let f = inner.entry_points.get(&entry)?;
        inner.functions.get(f)?.name
    }

    pub fn get_inputs(&self) -> Vec<IOData> {
        let inner = self.inner.lock().unwrap();
        inner.inputs.clone()
    }

    pub fn get_outputs(&self) -> Vec<IOData> {
        let inner = self.inner.lock().unwrap();
        inner.outputs.clone()
    }

    pub fn get_uniforms(&self) -> Vec<UniformData> {
        let inner = self.inner.lock().unwrap();
        inner.uniforms.clone()
    }

    pub fn get_storages(&self) -> Vec<StorageData> {
        let inner = self.inner.lock().unwrap();
        inner.storages.clone()
    }

    pub fn get_textures(&self) -> Vec<TextureData> {
        let inner = self.inner.lock().unwrap();
        inner.textures.clone()
    }

    pub fn get_sampled_textures(&self) -> Vec<SampledTextureData> {
        let inner = self.inner.lock().unwrap();
        inner.sampled_textures.clone()
    }

    pub fn get_samplers(&self) -> Vec<SamplerData> {
        let inner = self.inner.lock().unwrap();
        inner.samplers.clone()
    }

    pub fn get_push_constants(&self) -> Option<PushData> {
        let inner = self.inner.lock().unwrap();
        inner.push_constants.clone()
    }
}

// io
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    pub fn input<T: AsIOTypeConst>(&self, location: u32, flat: bool, name: Option<&'static str>) -> Input<T> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_none(), "Error cannot declare input: {{ location: {}, flat: {}, name: {:?} }} when builder is in a function", location, flat, name);
        let id = inner.inputs.len();
        inner.inputs.push(IOData {
            ty: T::IO_TY,
            location: Left(location),
            flat,
            name,
        });
        drop(inner);
        Input { 
            id, 
            inner: Arc::clone(&self.inner), 
            marker: std::marker::PhantomData,
        }
    }

    pub fn output<T: AsIOTypeConst>(&self, location: u32, flat: bool, name: Option<&'static str>) -> Output<T> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_none(), "Error cannot declare output: {{ location: {}, flat: {}, name: {:?} }} when builder is in a function", location, flat, name);
        let id = inner.outputs.len();
        inner.outputs.push(IOData {
            ty: T::IO_TY,
            location: Left(location),
            flat,
            name,
        });
        drop(inner);
        Output {
            id,
            inner: Arc::clone(&self.inner),
            marker: std::marker::PhantomData,
        }
    }
    
    fn built_in_input<T: AsIOTypeConst>(&self, built_in: rspirv::spirv::BuiltIn, name: &'static str) -> Input<T> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_none(), "Error cannot declare input: {:?} when builder is in a function", built_in);
        let id = inner.inputs.len();
        inner.inputs.push(IOData {
            ty: T::IO_TY,
            location: Right(built_in),
            flat: false,
            name: Some(name),
        });
        drop(inner);
        Input { 
            id, 
            inner: Arc::clone(&self.inner), 
            marker: std::marker::PhantomData,
        }
    }

    fn built_in_output<T: AsIOTypeConst>(&self, built_in: rspirv::spirv::BuiltIn, name: &'static str) -> Output<T> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_none(), "Error cannot declare built in output: {:?} when builder is in a function", built_in);
        let id = inner.outputs.len();
        inner.outputs.push(IOData {
            ty: T::IO_TY,
            location: Right(built_in),
            flat: false,
            name: Some(name),
        });
        drop(inner);
        Output {
            id,
            inner: Arc::clone(&self.inner),
            marker: std::marker::PhantomData,
        }
    }
}

macro_rules! impl_io {
    ($($name:ident, $f_in:ident, $f_flat_in:ident, $f_out:ident, $f_flat_out:ident,)*) => {
        $(
            pub fn $f_in(&self, location: u32, name: &'static str) -> Input<$name> {
                self.input(location, false, Some(name))
            }
    
            pub fn $f_flat_in(&self, location: u32, name: &'static str) -> Input<$name> {
                self.input(location, true, Some(name))
            }
    
            pub fn $f_out(&self, location: u32, name: &'static str) -> Output<$name> {
                self.output(location, false, Some(name))
            }
    
            pub fn $f_flat_out(&self, location: u32, name: &'static str) -> Output<$name> {
                self.output(location, true, Some(name))
            }
        )*
    };
}

macro_rules! impl_built_in_input {
    ($($f:ident, $ty:ident, $built_in:ident,)*) => {
        $(
            pub fn $f(&self) -> Input<$ty> {
                self.built_in_input(rspirv::spirv::BuiltIn::$built_in, stringify!($built_in))
            }
        )*
    };
}

macro_rules! impl_built_in_output {
    ($($f:ident, $ty:ident, $built_in:ident,)*) => {
        $(
            pub fn $f(&self) -> Output<$ty> {
                self.built_in_output(rspirv::spirv::BuiltIn::$built_in, stringify!($built_in))
            }
        )*
    };
}

impl Builder {
    #[rustfmt::skip]
    impl_io!(
        IOFloat, in_float, in_flat_float, out_float, out_flat_float,
        IOVec2, in_vec2, in_flat_vec2, out_vec2, out_flat_vec2,
        IOVec3, in_vec3, in_flat_vec3, out_vec3, out_flat_vec3,
        IOVec4, in_vec4, in_flat_vec4, out_vec4, out_flat_vec4,
    );

    #[rustfmt::skip]
    impl_built_in_input!(
        vertex_id, IOInt, VertexId,
        instance_index, IOInt, InstanceIndex,
        draw_index, IOInt, DrawIndex,
        base_vertex, IOInt, BaseVertex,

        patch_vertices, IOInt, PatchVertices,
        primitive_id, IOInt, PrimitiveId,
        invocation_id, IOInt, InvocationId,

        tess_coord, IOVec3, TessCoord,
        
        frag_coord, IOVec4, FragCoord,
        point_coord, IOVec2, PointCoord,
        layer, IOInt, Layer,

        num_work_groups, IOUVec3, NumWorkgroups,
        work_group_id, IOUVec3, WorkgroupId,
        local_invocation_id, IOUVec3, LocalInvocationId,
        global_invocation_id, IOUVec3, GlobalInvocationId,
        local_invocation_index, IOUInt, LocalInvocationIndex,
    );

    #[rustfmt::skip]
    impl_built_in_output!(
        vk_position, IOVec4, Position,
        point_size, IOFloat, PointSize,

        frag_depth, IOFloat, FragDepth,
    );
}

// functions
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    pub fn func<T: IsTypeConst, F: FnOnce()>(&self, name: Option<&'static str>, f: F) -> Func<T> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_none(), "Error cannot declare function: {{ name: {:?} }} when builder is in a function", name);
        let func_id = inner.functions.len();
        inner.functions.insert(func_id, FuncData { 
            ret: T::TY, 
            arguments: Vec::new(),
            instructions: Vec::new(), 
            name,
        });

        let scope = FuncScope::new();

        inner.scope = Some(Box::new(scope));

        drop(inner);

        f();

        let mut inner = self.inner.lock().unwrap();

        let instructions = match inner.scope.take().unwrap().downcast::<FuncScope>() {
            Ok(scope) => scope.instructions,
            Err(_) => unreachable!(),
        };
        
        let func_data = inner.functions.get_mut(&func_id).unwrap();
        func_data.instructions = instructions;

        drop(inner);

        Func {
            id: func_id,
            inner: Arc::clone(&self.inner),
            marker: std::marker::PhantomData,
        }
    }

    pub fn entry<F: FnOnce()>(&self, stage: ShaderStage, name: &'static str, f: F) {
        let main = self.func::<Void, _>(Some(name), f);

        let mut inner = self.inner.lock().unwrap();

        inner.entry_points.insert(stage, main.id);
    }
}

// set const
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    pub fn const_struct<'a, T: RustStructType>(&'a self, val: T) -> T::Spv<'a> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let id = val.struct_id(&mut **scope);
            drop(scope);
            drop(inner);
            T::Spv::from_id(id, &self.inner)
        } else {
            panic!("Cannot declare const struct when not in function");
        }
    }
}

impl Builder {
    fn set_const(&self, val: Val) -> usize {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.scope.is_some(), "Cannot declare new variable {:?} when not in function", val);

        let scope = inner.scope.as_mut().unwrap();

        let store = scope.get_new_id();

        scope.push_instruction(Instruction::SetConst(OpSetConst {
            val: val,
            store,
        }));

        drop(scope);
        drop(inner);

        store
    }
}

macro_rules! impl_set {
    ($($name:ident, $f:ident, $rust:ident, $enum:ident, $stct:ident,)*) => {
        $(
            pub fn $f(&self, val: $rust) -> $name {
                let id = self.set_const(Val::$enum($stct::$name(val)));
                $name {
                    id,
                    b: &self.inner
                }
            }
        )*
    };
}

impl Builder {
    #[rustfmt::skip]
    impl_set!(
        Int, const_int, i32, Scalar, ScalarVal,
        UInt, const_uint, u32, Scalar, ScalarVal,
        Float, const_float, f32, Scalar, ScalarVal,
        Double, const_double, f64, Scalar, ScalarVal,
        IVec2, const_ivec2, GlamIVec2, Vector, VectorVal,
        IVec3, const_ivec3, GlamIVec3, Vector, VectorVal,
        IVec4, const_ivec4, GlamIVec4, Vector, VectorVal,
        UVec2, const_uvec2, GlamUVec2, Vector, VectorVal,
        UVec3, const_uvec3, GlamUVec3, Vector, VectorVal,
        UVec4, const_uvec4, GlamUVec4, Vector, VectorVal,
        Vec2, const_vec2, GlamVec2, Vector, VectorVal,
        Vec3, const_vec3, GlamVec3, Vector, VectorVal,
        Vec4, const_vec4, GlamVec4, Vector, VectorVal,
        DVec2, const_dvec2, GlamDVec2, Vector, VectorVal,
        DVec3, const_dvec3, GlamDVec3, Vector, VectorVal,
        DVec4, const_dvec4, GlamDVec4, Vector, VectorVal,
        Mat2, const_mat2, GlamMat2, Matrix, MatrixVal,
        Mat3, const_mat3, GlamMat3, Matrix, MatrixVal,
        Mat4, const_mat4, GlamMat4, Matrix, MatrixVal,
        DMat2, const_dmat2, GlamDMat2, Matrix, MatrixVal,
        DMat3, const_dmat3, GlamDMat3, Matrix, MatrixVal,
        DMat4, const_dmat4, GlamDMat4, Matrix, MatrixVal,
    );
}

// construct
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    fn composite<'a>(&self, ty: Type, constituents: impl IntoIterator<Item=&'a dyn AsType>) -> usize {
        let mut inner = self.inner.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            let constituents = constituents.into_iter()
                .map(|c| {
                    (c.id(&mut **scope), c.ty())
                })
                .collect();

            scope.push_instruction(Instruction::Composite(OpComposite {
                ty,
                id: new_id,
                constituents,
            }));

            new_id
        } else {
            panic!("Cannot make construct vector when in scope");
        }
    }
}

macro_rules! make2 {
    ($($name:ident, $f:ident, $c:ident, $elem:ident,)*) => {
        $(
            pub fn $f<'a>(&'a self, x: &dyn SpvRustEq<$elem<'a>>, y: &dyn SpvRustEq<$elem<'a>>) -> $name<'a> {
                let id = self.composite(Type::$c, [x.as_ty_ref(), y.as_ty_ref()]);
                $name {
                    id,
                    b: &self.inner
                }
            }
        )*
    };
}

macro_rules! make3 {
    ($($name:ident, $f:ident, $c:ident, $elem:ident,)*) => {
        $(
            pub fn $f<'a>(&'a self, x: &dyn SpvRustEq<$elem<'a>>, y: &dyn SpvRustEq<$elem<'a>>, z: &dyn SpvRustEq<$elem<'a>>) -> $name<'a> {
                let id = self.composite(Type::$c, [x.as_ty_ref(), y.as_ty_ref(), z.as_ty_ref()]);
                $name {
                    id,
                    b: &self.inner
                }
            }
        )*
    };
}

macro_rules! make4 {
    ($($name:ident, $f:ident, $c:ident, $elem:ident,)*) => {
        $(
            pub fn $f<'a>(&'a self, x: &dyn SpvRustEq<$elem<'a>>, y: &dyn SpvRustEq<$elem<'a>>, z: &dyn SpvRustEq<$elem<'a>>, w: &dyn SpvRustEq<$elem<'a>>) -> $name<'a> {
                let id = self.composite(Type::$c, [x.as_ty_ref(), y.as_ty_ref(), z.as_ty_ref(), w.as_ty_ref()]);
                $name {
                    id,
                    b: &self.inner
                }
            }
        )*
    };
}

impl Builder {
    #[rustfmt::skip]
    make2!(
        IVec2, ivec2, IVEC2, Int,
        UVec2, uvec2, UVEC2, UInt,
        Vec2, vec2, VEC2, Float,
        DVec2, dvec2, DVEC2, Double,
        Mat2, mat2, MAT2, Vec2,
        DMat2, dmat2, DMAT2, DVec2,
    );

    #[rustfmt::skip]
    make3!(
        IVec3, ivec3, IVEC3, Int,
        UVec3, uvec3, UVEC3, UInt,
        Vec3, vec3, VEC3, Float,
        DVec3, dvec3, DVEC3, Double,
        Mat3, mat3, MAT3, Vec3,
        DMat3, dmat3, DMAT3, DVec3,
    );

    #[rustfmt::skip]
    make4!(
        IVec4, ivec4, IVEC4, Int,
        UVec4, uvec4, UVEC4, UInt,
        Vec4, vec4, VEC4, Float,
        DVec4, dvec4, DVEC4, Double,
        Mat4, mat4, MAT4, Vec4,
        DMat4, dmat4, DMAT4, DVec4,
    );
}

// bindings
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    pub fn push_constants<T: IsTypeConst>(&self, name: Option<&'static str>) -> PushConstants<T> {
        let mut inner = self.inner.lock().unwrap();

        inner.push_constants = Some(PushData { 
            ty: T::TY, 
            name 
        });

        drop(inner);
        PushConstants { 
            b: Arc::clone(&self.inner), 
            marker: std::marker::PhantomData 
        }
    }

    pub fn uniform<T: IsTypeConst>(&self, set: u32, binding: u32, name: Option<&'static str>) -> Uniform<T> {
        let mut inner = self.inner.lock().unwrap();

        let id = inner.uniforms.len();
        inner.uniforms.push(UniformData {
            ty: T::TY,
            set,
            binding,
            name,
        });

        drop(inner);
        Uniform { 
            id, 
            b: Arc::clone(&self.inner), 
            marker: std::marker::PhantomData 
        }
    }

    fn raw_storage<T: IsTypeConst>(&self, set: u32, binding: u32, read: bool, write: bool, name: Option<&'static str>) -> Storage<T> {
        let mut inner = self.inner.lock().unwrap();

        let id = inner.storages.len();
        inner.storages.push(StorageData { 
            ty: T::TY, 
            read, 
            write, 
            set, 
            binding, 
            name, 
        });

        drop(inner);
        Storage {
            id,
            b: Arc::clone(&self.inner),
            marker: std::marker::PhantomData,
        }
    }

    pub fn storage<T: IsTypeConst>(&self, set: u32, binding: u32, name: Option<&'static str>) -> Storage<T> {
        self.raw_storage(set, binding, true, true, name)
    }

    pub fn readonly_storage<T: IsTypeConst>(&self, set: u32, binding: u32, name: Option<&'static str>) -> Storage<T> {
        self.raw_storage(set, binding, true, false, name)
    }

    pub fn writeonly_storage<T: IsTypeConst>(&self, set: u32, binding: u32, name: Option<&'static str>) -> Storage<T> {
        self.raw_storage(set, binding, false, true, name)
    }
}

// texture
// ================================================================================
// ================================================================================
// ================================================================================

impl Builder {
    pub fn sampler(&self, set: u32, binding: u32, name: Option<&'static str>) -> Sampler {
        let mut inner = self.inner.lock().unwrap();

        let id = inner.samplers.len();
        inner.samplers.push(SamplerData { 
            set, 
            binding, 
            name 
        });

        Sampler {
            id,
        }
    }

    fn raw_texture<D: AsDimension, T: GTexture<D>>(&self, set: u32, binding: u32, name: Option<&'static str>) -> T {
        let mut inner = self.inner.lock().unwrap();

        let id = inner.textures.len();
        inner.textures.push(TextureData {
            ty: T::TEXTURE_TY,
            set,
            binding,
            name,
        });

        drop(inner);
        T::new(id, Arc::clone(&self.inner))
    }

    fn raw_sampled_texture<D: AsDimension, T: SampledGTexture<D>>(&self, set: u32, binding: u32, name: Option<&'static str>) -> T {
        let mut inner = self.inner.lock().unwrap();

        let id = inner.sampled_textures.len();
        inner.sampled_textures.push(SampledTextureData {
            ty: T::Texture::TEXTURE_TY,
            set,
            binding,
            name,
        });

        drop(inner);
        T::from_uniform(id, Arc::clone(&self.inner))
    }

    pub fn itexture<D: AsDimension>(&self, set: u32, binding: u32, name: Option<&'static str>) -> ITexture<D> {
        self.raw_texture(set, binding, name)
    }

    pub fn utexture<D: AsDimension>(&self, set: u32, binding: u32, name: Option<&'static str>) -> UTexture<D> {
        self.raw_texture(set, binding, name)
    }

    pub fn texture<D: AsDimension>(&self, set: u32, binding: u32, name: Option<&'static str>) -> Texture<D> {
        self.raw_texture(set, binding, name)
    }

    pub fn dtexture<D: AsDimension>(&self, set: u32, binding: u32, name: Option<&'static str>) -> DTexture<D> {
        self.raw_texture(set, binding, name)
    }
}

macro_rules! impl_texture {
    ($($name:ident, $f:ident,)*) => {
        $(
            pub fn $f(&self, set: u32, binding: u32, name: Option<&'static str>) -> $name {
                self.raw_texture(set, binding, name)
            }
        )*
    };
}

impl Builder {
    #[rustfmt::skip]
    impl_texture!(
        ITexture1D, itexture1d,
        ITexture2D, itexture2d,
        ITexture2DMs, itexture2d_ms,
        ITexture2DArray, itexture2d_array,
        ITexture2DMsArray, itexture2d_ms_array,
        ITextureCube, itexture_cube,
        ITextureCubeArray, itexture_cube_array,

        UTexture1D, utexture1d,
        UTexture1DArray, utexture1d_array,
        UTexture2D, utexture2d,
        UTexture2DMs, utexture2d_ms,
        UTexture2DArray, utexture2d_array,
        UTexture2DMsArray, utexture2d_ms_array,
        UTextureCube, utexture_cube,
        UTextureCubeArray, utexture_cube_array,

        Texture1D, texture1d,
        Texture1DArray, texture1d_array,
        Texture2D, texture2d,
        Texture2DMs, texture2d_ms,
        Texture2DArray, texture2d_array,
        Texture2DMsArray, texture2d_ms_array,
        TextureCube, texture_cube,
        TextureCubeArray, texture_cube_array,

        DTexture1D, dtexture1d,
        DTexture1DArray, dtexture1d_array,
        DTexture2D, dtexture2d,
        DTexture2DMs, dtexture2d_ms,
        DTexture2DArray, dtexture2d_array,
        DTexture2DMsArray, dtexture2d_ms_array,
        DTextureCube, dtexture_cube,
        DTextureCubeArray, dtexture_cube_array,
    );
}

macro_rules! impl_sampled_texture {
    ($($name:ident, $f:ident,)*) => {
        $(
            pub fn $f(&self, set: u32, binding: u32, name: Option<&'static str>) -> $name {
                self.raw_sampled_texture(set, binding, name)
            }
        )*
    };
}

impl Builder {
    #[rustfmt::skip]
    impl_sampled_texture!(
        SampledITexture1D, sampled_itexture1d,
        SampledITexture2D, sampled_itexture2d,
        SampledITexture2DMs, sampled_itexture2d_ms,
        SampledITexture2DArray, sampled_itexture2d_array,
        SampledITexture2DMsArray, sampled_itexture2d_ms_array,
        SampledITextureCube, sampled_itexture_cube,
        SampledITextureCubeArray, sampled_itexture_cube_array,

        SampledUTexture1D, sampled_utexture1d,
        SampledUTexture1DArray, sampled_utexture1d_array,
        SampledUTexture2D, sampled_utexture2d,
        SampledUTexture2DMs, sampled_utexture2d_ms,
        SampledUTexture2DArray, sampled_utexture2d_array,
        SampledUTexture2DMsArray, sampled_utexture2d_ms_array,
        SampledUTextureCube, sampled_utexture_cube,
        SampledUTextureCubeArray, sampled_utexture_cube_array,

        SampledTexture1D, sampled_texture1d,
        SampledTexture1DArray, sampled_texture1d_array,
        SampledTexture2D, sampled_texture2d,
        SampledTexture2DMs, sampled_texture2d_ms,
        SampledTexture2DArray, sampled_texture2d_array,
        SampledTexture2DMsArray, sampled_texture2d_ms_array,
        SampledTextureCube, sampled_texture_cube,
        SampledTextureCubeArray, sampled_texture_cube_array,

        SampledDTexture1D, sampled_dtexture1d,
        SampledDTexture1DArray, sampled_dtexture1d_array,
        SampledDTexture2D, sampled_dtexture2d,
        SampledDTexture2DMs, sampled_dtexture2d_ms,
        SampledDTexture2DArray, sampled_dtexture2d_array,
        SampledDTexture2DMsArray, sampled_dtexture2d_ms_array,
        SampledDTextureCube, sampled_dtexture_cube,
        SampledDTextureCubeArray, sampled_dtexture_cube_array,
    );
}

pub fn combine<D: AsDimension, T: GTexture<D>>(texture: &T, sampler: Sampler) -> T::Sampler {
    let mut inner = texture.b().lock().unwrap();
    if let Some(scope) = &mut inner.scope {
        let new_id = scope.get_new_id();

        scope.push_instruction(Instruction::Combine(OpCombine {
            tex_ty: T::TEXTURE_TY,
            texture: texture.texture_id(),
            sampler: sampler.id,
            store: new_id,
        }));
        
        drop(scope);
        drop(inner);
        T::Sampler::from_combine(new_id, Arc::clone(&texture.b()))
    } else {
        panic!("Cannot combine texture and sampler when not in function");
    }
}

pub fn sample<'a, 'b, D: AsDimension, S: SampledGTexture<D>>(sampled_texture: &'a S, coord: D::Coordinate<'b>) -> S::Sample<'a> {
    let mut inner = sampled_texture.b().lock().unwrap();
    if let Some(scope) = &mut inner.scope {
        let new_id = scope.get_new_id();

        let coord_id = coord.id(&mut **scope);

        scope.push_instruction(Instruction::Sample(OpSample {
            tex_ty: S::Texture::TEXTURE_TY,
            sampled_texture: sampled_texture.sampled_texture_id(),
            coordinate: (coord_id, D::Coordinate::TY),
            store: (new_id, S::Sample::TY),
            explict_lod: false,
        }));
        
        drop(scope);
        drop(inner);

        S::Sample::from_id(new_id, sampled_texture.b())
    } else {
        panic!("Cannot combine texture and sampler when not in function");
    }
}

// loop
// ================================================================================
// ================================================================================
// ================================================================================

// pub fn loop_while<F: FnOnce()>(b: Builder, f: F) {

// }

pub struct IfChain<'a> {
    builder: &'a Arc<Mutex<BuilderInner>>,
    then: Arc<Mutex<Option<Either<Box<OpIf>, OpElse>>>>,
}

pub fn spv_if<'a, F: FnOnce()>(b: Bool<'a>, f: F) -> IfChain<'a> {
    let mut inner = b.b.lock().unwrap();

    if let Some(scope) = inner.scope.take() {
        let if_scope = IfScope {
            instructions: Vec::new(),
            outer: scope,
        };

        inner.scope = Some(Box::new(if_scope));

        drop(inner);

        f();
        
        let mut inner = b.b.lock().unwrap();

        let mut if_scope = if let Ok(t) = inner.scope.take().unwrap().downcast::<IfScope>() {
            t
        } else {
            unreachable!()
        };

        let then = Arc::default();

        if_scope.outer.push_instruction(crate::Instruction::If(OpIf {
            condition: b.id,
            instructions: if_scope.instructions,
            then: Arc::clone(&then),
        }));

        inner.scope = Some(if_scope.outer);

        IfChain {
            builder: b.b,
            then,
        }
    } else {
        panic!("Cannot branch if not in function");
    }
}

impl<'a> IfChain<'a> {
    pub fn spv_else_if<'b, F: FnOnce()>(self, b: Bool<'b>, f: F) -> IfChain<'a> {
        let mut inner = b.b.lock().unwrap();

        if let Some(scope) = inner.scope.take() {
            let if_scope = IfScope {
                instructions: Vec::new(),
                outer: scope,
            };

            inner.scope = Some(Box::new(if_scope));

            drop(inner);

            f();

            let mut inner = b.b.lock().unwrap();
            
            let if_scope = if let Ok(t) = inner.scope.take().unwrap().downcast::<IfScope>() {
                t
            } else {
                unreachable!()
            };

            let new_then = Arc::default();

            let mut then = self.then.lock().unwrap();
            *then = Some(Left(Box::new(OpIf {
                condition: b.id,
                instructions: if_scope.instructions,
                then: Arc::clone(&new_then),
            })));

            inner.scope = Some(if_scope.outer);

            IfChain {
                builder: self.builder,
                then: new_then,
            }
        } else {
            panic!("Cannot branch if not in function");
        }
    }

    pub fn spv_else<F: FnOnce()>(self, f: F) {
        let mut inner = self.builder.lock().unwrap();

        if let Some(scope) = inner.scope.take() {
            let if_scope = IfScope {
                instructions: Vec::new(),
                outer: scope,
            };

            inner.scope = Some(Box::new(if_scope));

            drop(inner);

            f();

            let mut inner = self.builder.lock().unwrap();
            
            let if_scope = if let Ok(t) = inner.scope.take().unwrap().downcast::<IfScope>() {
                t
            } else {
                unreachable!()
            };

            let mut then = self.then.lock().unwrap();
            *then = Some(Right(OpElse {
                instructions: if_scope.instructions,
            }));

            inner.scope = Some(if_scope.outer);
        } else {
            panic!("Cannot branch if not in function");
        }
    }
}
