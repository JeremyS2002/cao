
use std::borrow::Cow;

use either::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalarType {
    Bool,
    Signed(u32),
    Unsigned(u32),
    Float(u32),
}

impl ScalarType {
    pub const BOOL: Self = Self::Bool;
    pub const INT: Self = Self::Signed(32);
    pub const UINT: Self = Self::Unsigned(32);
    pub const FLOAT: Self = Self::Float(32);
    pub const DOUBLE: Self = Self::Float(64);

    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        match self {
            ScalarType::Bool => b.type_bool(),
            ScalarType::Signed(w) => b.type_int(*w, 1),
            ScalarType::Unsigned(w) => b.type_int(*w, 0),
            ScalarType::Float(w) => b.type_float(*w)
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
    }

    pub fn size(&self) -> u32 {
        match self {
            ScalarType::Bool => 1,
            ScalarType::Signed(w) => w / 8,
            ScalarType::Unsigned(w) => w / 8,
            ScalarType::Float(w) => w / 8,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            ScalarType::Bool => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            ScalarType::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            ScalarType::Signed(_) => true,
            _ => false,
        }
    }

    pub fn is_uint(&self) -> bool {
        match self {
            ScalarType::Unsigned(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VectorType {
    pub scalar_ty: ScalarType,
    pub n_scalar: u32,
}

impl VectorType {
    pub const IVEC2: Self = Self {
        scalar_ty: ScalarType::INT,
        n_scalar: 2,
    };
    pub const IVEC3: Self = Self {
        scalar_ty: ScalarType::INT,
        n_scalar: 3,
    };
    pub const IVEC4: Self = Self {
        scalar_ty: ScalarType::INT,
        n_scalar: 4,
    };

    pub const UVEC2: Self = Self {
        scalar_ty: ScalarType::UINT,
        n_scalar: 2,
    };
    pub const UVEC3: Self = Self {
        scalar_ty: ScalarType::UINT,
        n_scalar: 3,
    };
    pub const UVEC4: Self = Self {
        scalar_ty: ScalarType::UINT,
        n_scalar: 4,
    };

    pub const VEC2: Self = Self {
        scalar_ty: ScalarType::FLOAT,
        n_scalar: 2,
    };
    pub const VEC3: Self = Self {
        scalar_ty: ScalarType::FLOAT,
        n_scalar: 3,
    };
    pub const VEC4: Self = Self {
        scalar_ty: ScalarType::FLOAT,
        n_scalar: 4,
    };

    pub const DVEC2: Self = Self {
        scalar_ty: ScalarType::DOUBLE,
        n_scalar: 2,
    };
    pub const DVEC3: Self = Self {
        scalar_ty: ScalarType::DOUBLE,
        n_scalar: 3,
    };
    pub const DVEC4: Self = Self {
        scalar_ty: ScalarType::DOUBLE,
        n_scalar: 4,
    };

    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let scalar = self.scalar_ty.rspirv(b);
        b.type_vector(scalar, self.n_scalar)
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
    }

    pub fn size(&self) -> u32 {
        self.n_scalar * self.scalar_ty.size()
    }

    pub fn is_float(&self) -> bool {
        self.scalar_ty.is_float()
    }

    pub fn is_int(&self) -> bool {
        self.scalar_ty.is_int()
    }

    pub fn is_uint(&self) -> bool {
        self.scalar_ty.is_uint()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MatrixType {
    pub vec_ty: VectorType,
    pub n_vec: u32
}

impl MatrixType {
    pub const MAT2: Self = Self {
        vec_ty: VectorType::VEC2,
        n_vec: 2,
    };
    pub const MAT3: Self = Self {
        vec_ty: VectorType::VEC3,
        n_vec: 3,
    };
    pub const MAT4: Self = Self {
        vec_ty: VectorType::VEC4,
        n_vec: 4,
    };

    pub const DMAT2: Self = Self {
        vec_ty: VectorType::DVEC2,
        n_vec: 2,
    };
    pub const DMAT3: Self = Self {
        vec_ty: VectorType::DVEC3,
        n_vec: 3,
    };
    pub const DMAT4: Self = Self {
        vec_ty: VectorType::DVEC4,
        n_vec: 4,
    };

    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let vec = self.vec_ty.rspirv(b);
        b.type_matrix(vec, self.n_vec)
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
    }

    pub fn size(&self) -> u32 {
        self.n_vec * self.vec_ty.size()
    }

    pub fn stride(&self) -> u32 {
        self.vec_ty.size()
    }

    pub fn is_float(&self) -> bool {
        self.vec_ty.scalar_ty.is_float()
    }

    pub fn is_int(&self) -> bool {
        self.vec_ty.scalar_ty.is_int()
    }

    pub fn is_uint(&self) -> bool {
        self.vec_ty.scalar_ty.is_uint()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub element_ty: Either<&'static Type, Box<Type>>,
    pub length: Option<usize>,
}

impl ArrayType {
    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let element = self.element_ty.rspirv(b);
        if let Some(length) = self.length {
            b.type_array(element, length as u32)
        } else {
            b.type_runtime_array(element)
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
    }

    pub fn size(&self) -> Option<u32> {
        if self.length.is_some() && self.element_ty.size().is_some() {
            Some(self.length.unwrap() as u32 * self.element_ty.size().unwrap())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructMember {
    pub name: Option<Either<&'static str, String>>,
    pub ty: Type,
    pub offset: u32,
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub name: Option<Either<&'static str, String>>,
    pub members: Cow<'static, [StructMember]>,
}

impl std::cmp::PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && (*self.members) == (*other.members)
    }
}

impl std::cmp::Eq for StructType { }

impl std::hash::Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        (*self.members).hash(state);
    }
}

impl StructType {
    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        if let Some(id) = b.struct_map.get(self) {
            *id
        } else {
            let members = self.members.iter().map(|m| m.ty.rspirv(b)).collect::<Vec<_>>();

            let id = b.type_struct(members);

            let mut idx = 0u32;
            for member in &*self.members {
                if let Some(name) = &member.name {
                    let name = match name {
                        Left(n) => *n,
                        Right(n) => &**n,
                    };
                    b.member_name(id, idx, name);
                }

                b.member_decorate(
                    id, 
                    idx, 
                    rspirv::spirv::Decoration::Offset, 
                    [rspirv::dr::Operand::LiteralInt32(member.offset)]
                );

                match member.ty {
                    Type::Matrix(m) => {
                        b.member_decorate(
                            id,
                            idx,
                            rspirv::spirv::Decoration::MatrixStride,
                            [
                                rspirv::dr::Operand::LiteralInt32(m.stride()),
                            ]
                        );

                        b.member_decorate(
                            id,
                            idx,
                            rspirv::spirv::Decoration::ColMajor,
                            []
                        );
                    },
                    _ => (),
                }

                idx += 1;
            }

            b.struct_map.insert(self.clone(), id);
            id
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
    }

    pub fn size(&self) -> Option<u32> {
        let mut size = 0;
        
        for member in &*self.members {
            if let Some(s) = member.ty.size() {
                size += s;
            } else {
                return None
            }
        }

        Some(size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    R8Unorm,
    Rg8Unorm,
    Rg8Snorm,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    // Rg64Uint,
    // Rg64Sint,
    // Rg64Float,

    // Rgb8Unorm,
    // Rgb8Snorm,
    // Rgb8Srgb,
    // Rgb16Unorm,
    // Rgb16Float,
    // Rgb16Snorm,
    // Rgb32Uint,
    // Rgb32Sint,
    // Rgb32Float,
    // Rgb64Uint,
    // Rgb64Sint,
    // Rgb64Float,

    Rgba8Unorm,
    Rgba8Snorm,
    Rgba8Srgb,
    Rgba16Unorm,
    Rgba16Float,
    Rgba16Snorm,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    // Rgba64Uint,
    // Rgba64Sint,
    // Rgba64Float,

    // Bgr8Unorm,
    // Bgr8Snorm,
    // Bgr8Srgb,
    // Bgra8Unorm,
    // Bgra8Snorm,
    // Bgra8Srgb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureSpvFormat {
    Color(TextureFormat),
    Sampled,
    Depth,
}

impl TextureSpvFormat {
    pub(crate) fn rspirv(&self) -> rspirv::spirv::ImageFormat {
        match self {
            Self::Color(c) => match c {
                TextureFormat::R8Unorm => rspirv::spirv::ImageFormat::R8,
                TextureFormat::Rg8Unorm => rspirv::spirv::ImageFormat::Rg8,
                TextureFormat::Rg8Snorm => rspirv::spirv::ImageFormat::Rg8Snorm,
                TextureFormat::Rg16Unorm => rspirv::spirv::ImageFormat::Rg16,
                TextureFormat::Rg16Snorm => rspirv::spirv::ImageFormat::Rg16Snorm,
                TextureFormat::Rg16Float => rspirv::spirv::ImageFormat::Rg16f,
                TextureFormat::Rg32Uint => rspirv::spirv::ImageFormat::Rg32ui,
                TextureFormat::Rg32Sint => rspirv::spirv::ImageFormat::Rg32i,
                TextureFormat::Rg32Float => rspirv::spirv::ImageFormat::Rg32f,
                // TextureFormat::Rg64Uint => rspirv::spirv::ImageFormat::Rg64ui,
                // TextureFormat::Rg64Sint => rspirv::spirv::ImageFormat::Rg64i,
                // TextureFormat::Rg64Float => rspirv::spirv::ImageFormat::Rg64f,
                // TextureFormat::Rgb8Unorm => rspirv::spirv::ImageFormat::Rgba8,
                // TextureFormat::Rgb8Snorm => rspirv::spirv::ImageFormat::Rgba8,
                // TextureFormat::Rgb8Srgb => rspirv::spirv::ImageFormat::Rgba8,
                // TextureFormat::Rgb16Unorm => rspirv::spirv::ImageFormat::Rgb16Unorm,
                // TextureFormat::Rgb16Float => rspirv::spirv::ImageFormat::Rgb16f,
                // TextureFormat::Rgb16Snorm => rspirv::spirv::ImageFormat::Rgb16Snorm,
                // TextureFormat::Rgb32Uint => rspirv::spirv::ImageFormat::Rgb32Uint,
                // TextureFormat::Rgb32Sint => rspirv::spirv::ImageFormat::Rgb32Sint,
                // TextureFormat::Rgb32Float => rspirv::spirv::ImageFormat::Rgb32f,
                // TextureFormat::Rgb64Uint => rspirv::spirv::ImageFormat::Rgb64Uint,
                // TextureFormat::Rgb64Sint => rspirv::spirv::ImageFormat::Rgb64Sint,
                // TextureFormat::Rgb64Float => rspirv::spirv::ImageFormat::Rgb64f,
                TextureFormat::Rgba8Unorm => rspirv::spirv::ImageFormat::Rgba8,
                TextureFormat::Rgba8Snorm => rspirv::spirv::ImageFormat::Rgba8Snorm,
                TextureFormat::Rgba8Srgb => rspirv::spirv::ImageFormat::Rgba8Snorm,
                TextureFormat::Rgba16Unorm => rspirv::spirv::ImageFormat::Rgba16,
                TextureFormat::Rgba16Float => rspirv::spirv::ImageFormat::Rgba16f,
                TextureFormat::Rgba16Snorm => rspirv::spirv::ImageFormat::Rgba16Snorm,
                TextureFormat::Rgba32Uint => rspirv::spirv::ImageFormat::Rgba32ui,
                TextureFormat::Rgba32Sint => rspirv::spirv::ImageFormat::Rgba32i,
                TextureFormat::Rgba32Float => rspirv::spirv::ImageFormat::Rgba32f,
                // TextureFormat::Rgba64Uint => rspirv::spirv::ImageFormat::Rgba64Uint,
                // TextureFormat::Rgba64Sint => rspirv::spirv::ImageFormat::Rgba64Sint,
                // TextureFormat::Rgba64Float => rspirv::spirv::ImageFormat::Rgba64f,
                // TextureFormat::Bgr8Unorm => rspirv::spirv::ImageFormat::Bgr8Unorm,
                // TextureFormat::Bgr8Snorm => rspirv::spirv::ImageFormat::Bgr8Snorm,
                // TextureFormat::Bgr8Srgb => rspirv::spirv::ImageFormat::Bgr8Srgb,
                // TextureFormat::Bgra8Unorm => rspirv::spirv::ImageFormat::Bgra8Unorm,
                // TextureFormat::Bgra8Snorm => rspirv::spirv::ImageFormat::Bgra8Snorm,
                // TextureFormat::Bgra8Srgb => rspirv::spirv::ImageFormat::Bgra8Srgb,
            },
            _ => rspirv::spirv::ImageFormat::Unknown
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureDimension {
    D1,
    D1Array,
    D2,
    D2Ms,
    D2Array,
    D2MsArray,
    Cube,
    CubeArray,
    D3,
}

impl TextureDimension {
    pub fn arrayed(&self) -> bool {
        match self {
            TextureDimension::D1Array => true,
            TextureDimension::D2Array => true,
            TextureDimension::D2MsArray => true,
            TextureDimension::CubeArray => true,
            _ => false,
        }
    }

    pub fn ms(&self) -> bool {
        match self {
            TextureDimension::D2Ms => true,
            TextureDimension::D2MsArray => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureType {
    pub scalar_ty: ScalarType,
    pub dimension: TextureDimension,
    pub format: TextureSpvFormat,
}

impl TextureType {
    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let component_spv_ty = self.scalar_ty.rspirv(b);
        b.type_image(
            component_spv_ty,
            match self.dimension {
                TextureDimension::D1 => rspirv::spirv::Dim::Dim1D,
                TextureDimension::D1Array => rspirv::spirv::Dim::Dim1D,
                TextureDimension::D2 => rspirv::spirv::Dim::Dim2D,
                TextureDimension::D2Ms => rspirv::spirv::Dim::Dim2D,
                TextureDimension::D2Array => rspirv::spirv::Dim::Dim2D,
                TextureDimension::D2MsArray => rspirv::spirv::Dim::Dim2D,
                TextureDimension::Cube => rspirv::spirv::Dim::DimCube,
                TextureDimension::CubeArray => rspirv::spirv::Dim::DimCube,
                TextureDimension::D3 => rspirv::spirv::Dim::Dim3D,
            },
            if let TextureSpvFormat::Depth = self.format {
                1
            } else {
                0
            },
            if self.dimension.arrayed() {
                1
            } else {
                0
            },
            if self.dimension.ms() {
                1
            } else {
                0
            },
            if let TextureSpvFormat::Sampled = self.format {
                1
            } else {
                0
            },
            self.format.rspirv(),
            None,
        )
    }

    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let spv_tex_ty = self.rspirv(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, spv_tex_ty)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
    Array(ArrayType),
    Struct(StructType),
    Texture(TextureType),
}

impl Type {
    pub const BOOL: Self = Self::Scalar(ScalarType::BOOL);
    pub const INT: Self = Self::Scalar(ScalarType::INT);
    pub const UINT: Self = Self::Scalar(ScalarType::UINT);
    pub const FLOAT: Self = Self::Scalar(ScalarType::FLOAT);
    pub const DOUBLE: Self = Self::Scalar(ScalarType::DOUBLE);

    pub const IVEC2: Self = Self::Vector(VectorType::IVEC2);
    pub const IVEC3: Self = Self::Vector(VectorType::IVEC3);
    pub const IVEC4: Self = Self::Vector(VectorType::IVEC4);
    pub const UVEC2: Self = Self::Vector(VectorType::UVEC2);
    pub const UVEC3: Self = Self::Vector(VectorType::UVEC3);
    pub const UVEC4: Self = Self::Vector(VectorType::UVEC4);
    pub const VEC2: Self = Self::Vector(VectorType::VEC2);
    pub const VEC3: Self = Self::Vector(VectorType::VEC3);
    pub const VEC4: Self = Self::Vector(VectorType::VEC4);
    pub const DVEC2: Self = Self::Vector(VectorType::DVEC2);
    pub const DVEC3: Self = Self::Vector(VectorType::DVEC3);
    pub const DVEC4: Self = Self::Vector(VectorType::DVEC4);

    pub const MAT2: Self = Self::Matrix(MatrixType::MAT2);
    pub const MAT3: Self = Self::Matrix(MatrixType::MAT3);
    pub const MAT4: Self = Self::Matrix(MatrixType::MAT4);
    pub const DMAT2: Self = Self::Matrix(MatrixType::DMAT2);
    pub const DMAT3: Self = Self::Matrix(MatrixType::DMAT3);
    pub const DMAT4: Self = Self::Matrix(MatrixType::DMAT4);

    pub(crate) fn rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        match self {
            Type::Void => b.type_void(),
            Type::Scalar(s) => s.rspirv(b),
            Type::Vector(v) => v.rspirv(b),
            Type::Matrix(m) => m.rspirv(b),
            Type::Array(a) => a.rspirv(b),
            Type::Struct(s) => s.rspirv(b),
            Type::Texture(t) => t.rspirv(b),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pointer(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        // let spv_ty = self.rspirv(b);
        // b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_ty)
        match self {
            Type::Void => {
                let ty = b.type_void();
                b.type_pointer(None, rspirv::spirv::StorageClass::Function, ty)
            },
            Type::Scalar(s) => s.pointer(b),
            Type::Vector(v) => v.pointer(b),
            Type::Matrix(m) => m.pointer(b),
            Type::Array(a) => a.pointer(b),
            Type::Struct(s) => s.pointer(b),
            Type::Texture(t) => t.pointer(b),
        }
    }

    pub fn size(&self) -> Option<u32> {
        match self {
            Type::Void => Some(0),
            Type::Scalar(s) => Some(s.size()),
            Type::Vector(v) => Some(v.size()),
            Type::Matrix(m) => Some(m.size()),
            Type::Array(a) => a.size(),
            Type::Struct(s) => s.size(),
            Type::Texture(_) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScalarVal {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
}

impl ScalarVal {
    pub fn scalar_ty(&self) -> ScalarType {
        match self {
            ScalarVal::Bool(_) => ScalarType::Bool,
            ScalarVal::Int(_) => ScalarType::Signed(32),
            ScalarVal::UInt(_) => ScalarType::Unsigned(32),
            ScalarVal::Float(_) => ScalarType::Float(32),
            ScalarVal::Double(_) => ScalarType::Float(64),
        }
    }

    pub(crate) fn set_rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let ty = self.scalar_ty().rspirv(b);
        match self {
            ScalarVal::Bool(bl) => if *bl  {
                b.constant_true(ty)
            } else {
                b.constant_false(ty)
            },
            ScalarVal::Int(i) => b.constant_u32(ty, unsafe { std::mem::transmute(*i) }),
            ScalarVal::UInt(u) => b.constant_u32(ty, *u),
            ScalarVal::Float(f) => b.constant_f32(ty, *f),
            ScalarVal::Double(d) => b.constant_f64(ty, *d),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VectorVal {
    IVec2(crate::GlamIVec2),
    IVec3(crate::GlamIVec3),
    IVec4(crate::GlamIVec4),
    UVec2(crate::GlamUVec2),
    UVec3(crate::GlamUVec3),
    UVec4(crate::GlamUVec4),
    Vec2(crate::GlamVec2),
    Vec3(crate::GlamVec3),
    Vec4(crate::GlamVec4),
    DVec2(crate::GlamDVec2),
    DVec3(crate::GlamDVec3),
    DVec4(crate::GlamDVec4),
}

impl VectorVal {
    pub fn vector_ty(&self) -> VectorType {
        match self {
            VectorVal::IVec2(_) => VectorType { 
                scalar_ty: ScalarType::Signed(32), 
                n_scalar: 2 
            },
            VectorVal::IVec3(_) => VectorType { 
                scalar_ty: ScalarType::Signed(32), 
                n_scalar: 3 
            },
            VectorVal::IVec4(_) => VectorType { 
                scalar_ty: ScalarType::Signed(32), 
                n_scalar: 4
            },
            VectorVal::UVec2(_) => VectorType { 
                scalar_ty: ScalarType::Unsigned(32), 
                n_scalar: 2 
            },
            VectorVal::UVec3(_) => VectorType { 
                scalar_ty: ScalarType::Unsigned(32), 
                n_scalar: 3
            },
            VectorVal::UVec4(_) => VectorType { 
                scalar_ty: ScalarType::Unsigned(32), 
                n_scalar: 4 
            },
            VectorVal::Vec2(_) => VectorType { 
                scalar_ty: ScalarType::Float(32), 
                n_scalar: 2 
            },
            VectorVal::Vec3(_) => VectorType { 
                scalar_ty: ScalarType::Float(32), 
                n_scalar: 3 
            },
            VectorVal::Vec4(_) => VectorType { 
                scalar_ty: ScalarType::Float(32), 
                n_scalar: 4 
            },
            VectorVal::DVec2(_) => VectorType { 
                scalar_ty: ScalarType::Float(64), 
                n_scalar: 2 
            },
            VectorVal::DVec3(_) => VectorType { 
                scalar_ty: ScalarType::Float(64), 
                n_scalar: 3 
            },
            VectorVal::DVec4(_) => VectorType { 
                scalar_ty: ScalarType::Float(64), 
                n_scalar: 4 
            },
        }
    }

    pub(crate) fn set_rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let ty = self.vector_ty().rspirv(b);
        match self {
            VectorVal::IVec2(v) => {
                let x = ScalarVal::Int(v.x).set_rspirv(b);
                let y = ScalarVal::Int(v.y).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            VectorVal::IVec3(v) => {
                let x = ScalarVal::Int(v.x).set_rspirv(b);
                let y = ScalarVal::Int(v.y).set_rspirv(b);
                let z = ScalarVal::Int(v.z).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            VectorVal::IVec4(v) => {
                let x = ScalarVal::Int(v.x).set_rspirv(b);
                let y = ScalarVal::Int(v.y).set_rspirv(b);
                let z = ScalarVal::Int(v.z).set_rspirv(b);
                let w = ScalarVal::Int(v.w).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
            VectorVal::UVec2(v) => {
                let x = ScalarVal::UInt(v.x).set_rspirv(b);
                let y = ScalarVal::UInt(v.y).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            VectorVal::UVec3(v) => {
                let x = ScalarVal::UInt(v.x).set_rspirv(b);
                let y = ScalarVal::UInt(v.y).set_rspirv(b);
                let z = ScalarVal::UInt(v.z).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            VectorVal::UVec4(v) => {
                let x = ScalarVal::UInt(v.x).set_rspirv(b);
                let y = ScalarVal::UInt(v.y).set_rspirv(b);
                let z = ScalarVal::UInt(v.z).set_rspirv(b);
                let w = ScalarVal::UInt(v.w).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
            VectorVal::Vec2(v) => {
                let x = ScalarVal::Float(v.x).set_rspirv(b);
                let y = ScalarVal::Float(v.y).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            VectorVal::Vec3(v) => {
                let x = ScalarVal::Float(v.x).set_rspirv(b);
                let y = ScalarVal::Float(v.y).set_rspirv(b);
                let z = ScalarVal::Float(v.z).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            VectorVal::Vec4(v) => {
                let x = ScalarVal::Float(v.x).set_rspirv(b);
                let y = ScalarVal::Float(v.y).set_rspirv(b);
                let z = ScalarVal::Float(v.z).set_rspirv(b);
                let w = ScalarVal::Float(v.w).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
            VectorVal::DVec2(v) => {
                let x = ScalarVal::Double(v.x).set_rspirv(b);
                let y = ScalarVal::Double(v.y).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            VectorVal::DVec3(v) => {
                let x = ScalarVal::Double(v.x).set_rspirv(b);
                let y = ScalarVal::Double(v.y).set_rspirv(b);
                let z = ScalarVal::Double(v.z).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            VectorVal::DVec4(v) => {
                let x = ScalarVal::Double(v.x).set_rspirv(b);
                let y = ScalarVal::Double(v.y).set_rspirv(b);
                let z = ScalarVal::Double(v.z).set_rspirv(b);
                let w = ScalarVal::Double(v.w).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MatrixVal {
    Mat2(crate::GlamMat2),
    Mat3(crate::GlamMat3),
    Mat4(crate::GlamMat4),
    DMat2(crate::GlamDMat2),
    DMat3(crate::GlamDMat3),
    DMat4(crate::GlamDMat4),
}

impl MatrixVal {
    pub fn matrix_ty(&self) -> MatrixType {
        match self {
            MatrixVal::Mat2(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(32), 
                    n_scalar: 2 
                }, 
                n_vec: 2, 
            },
            MatrixVal::Mat3(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(32), 
                    n_scalar: 3 
                }, 
                n_vec: 3, 
            },
            MatrixVal::Mat4(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(32), 
                    n_scalar: 4 
                }, 
                n_vec: 4, 
            },
            MatrixVal::DMat2(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(64), 
                    n_scalar: 2 
                }, 
                n_vec: 2, 
            },
            MatrixVal::DMat3(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(64), 
                    n_scalar: 3 
                }, 
                n_vec: 3, 
            },
            MatrixVal::DMat4(_) => MatrixType { 
                vec_ty: VectorType { 
                    scalar_ty: ScalarType::Float(64), 
                    n_scalar: 4 
                }, 
                n_vec: 4, 
            },
        }
    }

    pub(crate) fn set_rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        let ty = self.matrix_ty().rspirv(b);

        match self {
            MatrixVal::Mat2(m) => {
                let x = VectorVal::Vec2(m.col(0)).set_rspirv(b);
                let y = VectorVal::Vec2(m.col(1)).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            MatrixVal::Mat3(m) => {
                let x = VectorVal::Vec3(m.col(0)).set_rspirv(b);
                let y = VectorVal::Vec3(m.col(1)).set_rspirv(b);
                let z = VectorVal::Vec3(m.col(2)).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            MatrixVal::Mat4(m) => {
                let x = VectorVal::Vec4(m.col(0)).set_rspirv(b);
                let y = VectorVal::Vec4(m.col(1)).set_rspirv(b);
                let z = VectorVal::Vec4(m.col(2)).set_rspirv(b);
                let w = VectorVal::Vec4(m.col(3)).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
            MatrixVal::DMat2(m) => {
                let x = VectorVal::DVec2(m.col(0)).set_rspirv(b);
                let y = VectorVal::DVec2(m.col(1)).set_rspirv(b);
                b.constant_composite(ty, [x, y])
            },
            MatrixVal::DMat3(m) => {
                let x = VectorVal::DVec3(m.col(0)).set_rspirv(b);
                let y = VectorVal::DVec3(m.col(1)).set_rspirv(b);
                let z = VectorVal::DVec3(m.col(2)).set_rspirv(b);
                b.constant_composite(ty, [x, y, z])
            },
            MatrixVal::DMat4(m) => {
                let x = VectorVal::DVec4(m.col(0)).set_rspirv(b);
                let y = VectorVal::DVec4(m.col(1)).set_rspirv(b);
                let z = VectorVal::DVec4(m.col(2)).set_rspirv(b);
                let w = VectorVal::DVec4(m.col(3)).set_rspirv(b);
                b.constant_composite(ty, [x, y, z, w])
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Val {
    Scalar(ScalarVal),
    Vector(VectorVal),
    Matrix(MatrixVal),
}

impl Val {
    pub fn ty(&self) -> Type {
        match self {
            Val::Scalar(s) => Type::Scalar(s.scalar_ty()),
            Val::Vector(v) => Type::Vector(v.vector_ty()),
            Val::Matrix(m) => Type::Matrix(m.matrix_ty()),
        }
    }

    pub(crate) fn set_rspirv(&self, b: &mut crate::RSpirvBuilder) -> u32 {
        match self {
            Val::Scalar(v) => v.set_rspirv(b),
            Val::Vector(v) => v.set_rspirv(b),
            Val::Matrix(v) => v.set_rspirv(b),
        }
    }
}