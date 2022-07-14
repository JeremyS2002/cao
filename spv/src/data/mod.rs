use std::{any::TypeId, collections::HashMap};

pub mod as_ty;
pub mod ops;
pub mod spv_array;
pub mod spv_struct;
pub mod ty_structs;

pub use as_ty::*;
pub use ops::*;
pub use spv_array::*;
pub use spv_struct::*;
pub use ty_structs::*;

// if I could write better macros then this wouldn't be necissary
type RustDMat2 = [[f64; 2]; 2];
type RustDMat3 = [[f64; 3]; 3];
type RustDMat4 = [[f64; 4]; 4];
type RustDVec2 = [f64; 2];
type RustDVec3 = [f64; 3];
type RustDVec4 = [f64; 4];
type RustIVec2 = [i32; 2];
type RustIVec3 = [i32; 3];
type RustIVec4 = [i32; 4];
type RustMat2 = [[f32; 2]; 2];
type RustMat3 = [[f32; 3]; 3];
type RustMat4 = [[f32; 4]; 4];
type RustUVec2 = [u32; 2];
type RustUVec3 = [u32; 3];
type RustUVec4 = [u32; 4];
type RustVec2 = [f32; 2];
type RustVec3 = [f32; 3];
type RustVec4 = [f32; 4];

pub trait DataRef {
    fn ty(&self) -> PrimitiveType;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PrimitiveType {
    Bool,
    Int,
    UInt,
    Float,
    Double,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
    DVec2,
    DVec3,
    DVec4,
    Mat2,
    Mat3,
    Mat4,
    DMat2,
    DMat3,
    DMat4,
}

impl PrimitiveType {
    pub fn component(&self) -> Option<PrimitiveType> {
        match self {
            PrimitiveType::IVec2 => Some(PrimitiveType::Int),
            PrimitiveType::IVec3 => Some(PrimitiveType::Int),
            PrimitiveType::IVec4 => Some(PrimitiveType::Int),
            PrimitiveType::UVec2 => Some(PrimitiveType::UInt),
            PrimitiveType::UVec3 => Some(PrimitiveType::UInt),
            PrimitiveType::UVec4 => Some(PrimitiveType::UInt),
            PrimitiveType::Vec2 => Some(PrimitiveType::Float),
            PrimitiveType::Vec3 => Some(PrimitiveType::Float),
            PrimitiveType::Vec4 => Some(PrimitiveType::Float),
            PrimitiveType::DVec2 => Some(PrimitiveType::Double),
            PrimitiveType::DVec3 => Some(PrimitiveType::Double),
            PrimitiveType::DVec4 => Some(PrimitiveType::Double),
            PrimitiveType::Mat2 => Some(PrimitiveType::Float),
            PrimitiveType::Mat3 => Some(PrimitiveType::Float),
            PrimitiveType::Mat4 => Some(PrimitiveType::Float),
            PrimitiveType::DMat2 => Some(PrimitiveType::Double),
            PrimitiveType::DMat3 => Some(PrimitiveType::Double),
            PrimitiveType::DMat4 => Some(PrimitiveType::Double),
            _ => None,
        }
    }

    pub fn components(&self) -> u32 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::Int => 1,
            PrimitiveType::UInt => 1,
            PrimitiveType::Float => 1,
            PrimitiveType::Double => 1,
            PrimitiveType::IVec2 => 2,
            PrimitiveType::IVec3 => 3,
            PrimitiveType::IVec4 => 4,
            PrimitiveType::UVec2 => 2,
            PrimitiveType::UVec3 => 3,
            PrimitiveType::UVec4 => 4,
            PrimitiveType::Vec2 => 2,
            PrimitiveType::Vec3 => 3,
            PrimitiveType::Vec4 => 4,
            PrimitiveType::DVec2 => 2,
            PrimitiveType::DVec3 => 3,
            PrimitiveType::DVec4 => 4,
            PrimitiveType::Mat2 => 4,
            PrimitiveType::Mat3 => 9,
            PrimitiveType::Mat4 => 16,
            PrimitiveType::DMat2 => 4,
            PrimitiveType::DMat3 => 9,
            PrimitiveType::DMat4 => 16,
        }
    }

    pub fn vector_components(&self) -> Option<u32> {
        match self {
            PrimitiveType::Mat2 => Some(2),
            PrimitiveType::Mat3 => Some(3),
            PrimitiveType::Mat4 => Some(4),
            PrimitiveType::DMat2 => Some(2),
            PrimitiveType::DMat3 => Some(3),
            PrimitiveType::DMat4 => Some(4),
            _ => None,
        }
    }

    pub fn vector_type(&self) -> Option<PrimitiveType> {
        match self {
            PrimitiveType::Mat2 => Some(PrimitiveType::Vec2),
            PrimitiveType::Mat3 => Some(PrimitiveType::Vec3),
            PrimitiveType::Mat4 => Some(PrimitiveType::Vec4),
            PrimitiveType::DMat2 => Some(PrimitiveType::DVec2),
            PrimitiveType::DMat3 => Some(PrimitiveType::DVec3),
            PrimitiveType::DMat4 => Some(PrimitiveType::DVec4),
            _ => None,
        }
    }

    /// Returns the number of bytes between rows of a matrix
    /// TODO test this matches with spirv and rust data.
    /// shaderc compiling glsl marks all matrices as having a stride of 16
    pub fn matrix_stride(&self) -> Option<u32> {
        match self {
            PrimitiveType::Mat2 => Some(2 * 4),
            PrimitiveType::Mat3 => Some(3 * 4),
            PrimitiveType::Mat4 => Some(4 * 4),
            PrimitiveType::DMat2 => Some(2 * 8),
            PrimitiveType::DMat3 => Some(3 * 8),
            PrimitiveType::DMat4 => Some(4 * 8),
            _ => None,
        }
    }

    /// Returns the size of the Primitive in bytes
    pub fn size(&self) -> u32 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::Int => 4,
            PrimitiveType::UInt => 4,
            PrimitiveType::Float => 4,
            PrimitiveType::Double => 8,
            PrimitiveType::IVec2 => 2 * 4,
            PrimitiveType::IVec3 => 3 * 4,
            PrimitiveType::IVec4 => 4 * 4,
            PrimitiveType::UVec2 => 2 * 4,
            PrimitiveType::UVec3 => 3 * 4,
            PrimitiveType::UVec4 => 4 * 4,
            PrimitiveType::Vec2 => 2 * 4,
            PrimitiveType::Vec3 => 3 * 4,
            PrimitiveType::Vec4 => 4 * 4,
            PrimitiveType::DVec2 => 2 * 8,
            PrimitiveType::DVec3 => 3 * 8,
            PrimitiveType::DVec4 => 4 * 8,
            PrimitiveType::Mat2 => 4 * 4,
            PrimitiveType::Mat3 => 9 * 4,
            PrimitiveType::Mat4 => 16 * 4,
            PrimitiveType::DMat2 => 4 * 8,
            PrimitiveType::DMat3 => 9 * 8,
            PrimitiveType::DMat4 => 16 * 8,
        }
    }

    pub fn is_matrix(&self) -> bool {
        match self {
            Self::Mat2 | Self::Mat3 | Self::Mat4 | Self::DMat2 | Self::DMat3 | Self::DMat4 => true,
            _ => false,
        }
    }

    pub fn is_vector(&self) -> bool {
        match self {
            Self::IVec2
            | Self::IVec3
            | Self::IVec4
            | Self::UVec2
            | Self::UVec3
            | Self::UVec4
            | Self::Vec2
            | Self::Vec3
            | Self::Vec4
            | Self::DVec2
            | Self::DVec3
            | Self::DVec4 => true,
            _ => false,
        }
    }

    pub fn is_scalar(&self) -> bool {
        match self {
            Self::Bool | Self::Int | Self::UInt | Self::Float | Self::Double => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Self::Int | Self::IVec2 | Self::IVec3 | Self::IVec4 => true,
            _ => false,
        }
    }

    pub fn is_uint(&self) -> bool {
        match self {
            Self::UInt | Self::UVec2 | Self::UVec3 | Self::UVec4 => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::Float
            | Self::Vec2
            | Self::Vec3
            | Self::Vec4
            | Self::Mat2
            | Self::Mat3
            | Self::Mat4 => true,
            _ => false,
        }
    }

    pub fn is_double(&self) -> bool {
        match self {
            Self::Double
            | Self::DVec2
            | Self::DVec3
            | Self::DVec4
            | Self::DMat2
            | Self::DMat3
            | Self::DMat4 => true,
            _ => false,
        }
    }

    pub fn base_type(&self, b: &mut rspirv::dr::Builder) -> u32 {
        match self {
            PrimitiveType::Bool => b.type_bool(),
            PrimitiveType::Int => b.type_int(32, 1),
            PrimitiveType::UInt => b.type_int(32, 0),
            PrimitiveType::Float => b.type_float(32),
            PrimitiveType::Double => b.type_float(64),
            PrimitiveType::IVec2 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 2)
            }
            PrimitiveType::IVec3 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 3)
            }
            PrimitiveType::IVec4 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 4)
            }
            PrimitiveType::UVec2 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 2)
            }
            PrimitiveType::UVec3 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 3)
            }
            PrimitiveType::UVec4 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 4)
            }
            PrimitiveType::Vec2 => {
                let r = b.type_float(32);
                b.type_vector(r, 2)
            }
            PrimitiveType::Vec3 => {
                let r = b.type_float(32);
                b.type_vector(r, 3)
            }
            PrimitiveType::Vec4 => {
                let r = b.type_float(32);
                b.type_vector(r, 4)
            }
            PrimitiveType::DVec2 => {
                let r = b.type_float(64);
                b.type_vector(r, 2)
            }
            PrimitiveType::DVec3 => {
                let r = b.type_float(64);
                b.type_vector(r, 3)
            }
            PrimitiveType::DVec4 => {
                let r = b.type_float(64);
                b.type_vector(r, 4)
            }
            PrimitiveType::Mat2 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 2);
                b.type_matrix(v, 2)
            }
            PrimitiveType::Mat3 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 3);
                b.type_matrix(v, 3)
            }
            PrimitiveType::Mat4 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 4);
                b.type_matrix(v, 4)
            }
            PrimitiveType::DMat2 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 2);
                b.type_matrix(v, 2)
            }
            PrimitiveType::DMat3 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 3);
                b.type_matrix(v, 3)
            }
            PrimitiveType::DMat4 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 4);
                b.type_matrix(v, 4)
            }
        }
    }

    pub(crate) fn pointer_type(&self, b: &mut rspirv::dr::Builder) -> u32 {
        let b_ty = self.base_type(b);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, b_ty)
    }

    pub(crate) fn variable(&self, b: &mut rspirv::dr::Builder, var_block: usize) -> u32 {
        let p_ty = self.pointer_type(b);
        let current_block = b.selected_block().unwrap();
        b.select_block(Some(var_block)).unwrap();
        let id = b.id();
        b.insert_into_block(
            rspirv::dr::InsertPoint::Begin,
            rspirv::dr::Instruction::new(
                rspirv::spirv::Op::Variable,
                Some(p_ty),
                Some(id),
                vec![rspirv::dr::Operand::StorageClass(
                    rspirv::spirv::StorageClass::Function,
                )],
            ),
        )
        .unwrap();
        b.select_block(Some(current_block)).unwrap();
        id
        // b.variable(
        //     p_ty,
        //     None,
        //     rspirv::spirv::StorageClass::Function,
        //     None,
        // )
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PrimitiveVal {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    IVec2(RustIVec2),
    IVec3(RustIVec3),
    IVec4(RustIVec4),
    UVec2(RustUVec2),
    UVec3(RustUVec3),
    UVec4(RustUVec4),
    Vec2(RustVec2),
    Vec3(RustVec3),
    Vec4(RustVec4),
    DVec2(RustDVec2),
    DVec3(RustDVec3),
    DVec4(RustDVec4),
    Mat2(RustMat2),
    Mat3(RustMat3),
    Mat4(RustMat4),
    DMat2(RustDMat2),
    DMat3(RustDMat3),
    DMat4(RustDMat4),
}

// impl From<DataVal> for DataType {
//     fn from(v: DataVal) -> Self {
//         v.into()
//     }
// }

impl From<PrimitiveVal> for PrimitiveType {
    fn from(v: PrimitiveVal) -> Self {
        Self::from(&v)
    }
}

impl From<&'_ PrimitiveVal> for PrimitiveType {
    fn from(v: &'_ PrimitiveVal) -> Self {
        match v {
            PrimitiveVal::Bool(_) => PrimitiveType::Bool,
            PrimitiveVal::Int(_) => PrimitiveType::Int,
            PrimitiveVal::UInt(_) => PrimitiveType::UInt,
            PrimitiveVal::Float(_) => PrimitiveType::Float,
            PrimitiveVal::Double(_) => PrimitiveType::Double,
            PrimitiveVal::IVec3(_) => PrimitiveType::IVec2,
            PrimitiveVal::IVec4(_) => PrimitiveType::IVec3,
            PrimitiveVal::IVec2(_) => PrimitiveType::IVec4,
            PrimitiveVal::UVec3(_) => PrimitiveType::UVec2,
            PrimitiveVal::UVec2(_) => PrimitiveType::UVec3,
            PrimitiveVal::UVec4(_) => PrimitiveType::UVec4,
            PrimitiveVal::Vec3(_) => PrimitiveType::Vec2,
            PrimitiveVal::Vec2(_) => PrimitiveType::Vec3,
            PrimitiveVal::Vec4(_) => PrimitiveType::Vec4,
            PrimitiveVal::DVec3(_) => PrimitiveType::DVec2,
            PrimitiveVal::DVec2(_) => PrimitiveType::DVec3,
            PrimitiveVal::DVec4(_) => PrimitiveType::DVec4,
            PrimitiveVal::Mat3(_) => PrimitiveType::Mat2,
            PrimitiveVal::Mat2(_) => PrimitiveType::Mat3,
            PrimitiveVal::Mat4(_) => PrimitiveType::Mat4,
            PrimitiveVal::DMat3(_) => PrimitiveType::DMat2,
            PrimitiveVal::DMat2(_) => PrimitiveType::DMat3,
            PrimitiveVal::DMat4(_) => PrimitiveType::DMat4,
        }
    }
}

impl PrimitiveVal {
    pub(crate) fn set_constant(&self, b: &mut rspirv::dr::Builder) -> (u32, u32) {
        match self {
            PrimitiveVal::Bool(v) => {
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = if *v {
                    b.constant_true(ty)
                } else {
                    b.constant_false(ty)
                };
                (c, ty)
            }
            PrimitiveVal::Int(v) => {
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_u32(ty, unsafe { std::mem::transmute(*v) });
                (c, ty)
            }
            PrimitiveVal::UInt(v) => {
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_u32(ty, *v);
                (c, ty)
            }
            PrimitiveVal::Float(v) => {
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_f32(ty, *v);
                (c, ty)
            }
            PrimitiveVal::Double(v) => {
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_f64(ty, *v);
                (c, ty)
            }
            PrimitiveVal::IVec2(v) => {
                let x = Self::Int(v[0]).set_constant(b).0;
                let y = Self::Int(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::IVec3(v) => {
                let x = Self::Int(v[0]).set_constant(b).0;
                let y = Self::Int(v[1]).set_constant(b).0;
                let z = Self::Int(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::IVec4(v) => {
                let x = Self::Int(v[0]).set_constant(b).0;
                let y = Self::Int(v[1]).set_constant(b).0;
                let z = Self::Int(v[2]).set_constant(b).0;
                let w = Self::Int(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
            PrimitiveVal::UVec2(v) => {
                let x = Self::UInt(v[0]).set_constant(b).0;
                let y = Self::UInt(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::UVec3(v) => {
                let x = Self::UInt(v[0]).set_constant(b).0;
                let y = Self::UInt(v[1]).set_constant(b).0;
                let z = Self::UInt(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::UVec4(v) => {
                let x = Self::UInt(v[0]).set_constant(b).0;
                let y = Self::UInt(v[1]).set_constant(b).0;
                let z = Self::UInt(v[2]).set_constant(b).0;
                let w = Self::UInt(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
            PrimitiveVal::Vec2(v) => {
                let x = Self::Float(v[0]).set_constant(b).0;
                let y = Self::Float(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::Vec3(v) => {
                let x = Self::Float(v[0]).set_constant(b).0;
                let y = Self::Float(v[1]).set_constant(b).0;
                let z = Self::Float(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::Vec4(v) => {
                let x = Self::Float(v[0]).set_constant(b).0;
                let y = Self::Float(v[1]).set_constant(b).0;
                let z = Self::Float(v[2]).set_constant(b).0;
                let w = Self::Float(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
            PrimitiveVal::DVec2(v) => {
                let x = Self::Double(v[0]).set_constant(b).0;
                let y = Self::Double(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::DVec3(v) => {
                let x = Self::Double(v[0]).set_constant(b).0;
                let y = Self::Double(v[1]).set_constant(b).0;
                let z = Self::Double(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::DVec4(v) => {
                let x = Self::Double(v[0]).set_constant(b).0;
                let y = Self::Double(v[1]).set_constant(b).0;
                let z = Self::Double(v[2]).set_constant(b).0;
                let w = Self::Double(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
            PrimitiveVal::Mat2(v) => {
                let x = Self::Vec2(v[0]).set_constant(b).0;
                let y = Self::Vec2(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::Mat3(v) => {
                let x = Self::Vec3(v[0]).set_constant(b).0;
                let y = Self::Vec3(v[1]).set_constant(b).0;
                let z = Self::Vec3(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::Mat4(v) => {
                let x = Self::Vec4(v[0]).set_constant(b).0;
                let y = Self::Vec4(v[1]).set_constant(b).0;
                let z = Self::Vec4(v[2]).set_constant(b).0;
                let w = Self::Vec4(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
            PrimitiveVal::DMat2(v) => {
                let x = Self::DVec2(v[0]).set_constant(b).0;
                let y = Self::DVec2(v[1]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            }
            PrimitiveVal::DMat3(v) => {
                let x = Self::DVec3(v[0]).set_constant(b).0;
                let y = Self::DVec3(v[1]).set_constant(b).0;
                let z = Self::DVec3(v[2]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            }
            PrimitiveVal::DMat4(v) => {
                let x = Self::DVec4(v[0]).set_constant(b).0;
                let y = Self::DVec4(v[1]).set_constant(b).0;
                let z = Self::DVec4(v[2]).set_constant(b).0;
                let w = Self::DVec4(v[3]).set_constant(b).0;
                let ty = PrimitiveType::from(*self).base_type(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            }
        }
    }

    pub fn set(&self, b: &mut rspirv::dr::Builder, var_block: usize) -> u32 {
        let c = self.set_constant(b).0;
        let var = PrimitiveType::from(self).variable(b, var_block);
        b.store(var, c, None, None).unwrap();
        var
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataType {
    Primitive(PrimitiveType),
    Array(PrimitiveType, usize),
    Struct(
        TypeId,
        &'static str,
        &'static [&'static str],
        &'static [DataType],
    ),
}

impl DataType {
    /// Returns the size of the data type in bytes
    pub fn size(&self) -> u32 {
        match self {
            DataType::Primitive(p) => p.size(),
            DataType::Array(p, n) => p.size() * (*n) as u32,
            DataType::Struct(_, _, _, d) => (*d).iter().fold(0, |a, b| a + b.size()),
        }
    }

    pub fn base_type(
        &self,
        b: &mut rspirv::dr::Builder,
        struct_map: &mut HashMap<TypeId, u32>,
    ) -> u32 {
        match self {
            Self::Primitive(ty) => ty.base_type(b),
            Self::Array(ty, n) => {
                let p = ty.base_type(b);
                let l = PrimitiveVal::UInt(*n as _).set_constant(b).0;
                b.type_array(p, l)
            }
            Self::Struct(id, name, names, types) => {
                if let Some(spv_type_object) = struct_map.get(id) {
                    *spv_type_object
                } else {
                    let spv_types = types
                        .iter()
                        .map(|t| t.base_type(b, struct_map))
                        .collect::<Vec<_>>();

                    let spv_ty_object = b.type_struct(spv_types);

                    struct_map.insert(*id, spv_ty_object);

                    b.name(spv_ty_object, *name);

                    let mut offset = 0u32;
                    let mut index = 0u32;
                    for (ty, &name) in types.iter().zip(*names) {
                        b.member_name(spv_ty_object, index, name);
                        b.member_decorate(
                            spv_ty_object,
                            index,
                            rspirv::spirv::Decoration::Offset,
                            [rspirv::dr::Operand::LiteralInt32(offset)],
                        );
                        if let DataType::Primitive(p) = ty {
                            if p.is_matrix() {
                                b.member_decorate(
                                    spv_ty_object,
                                    index,
                                    rspirv::spirv::Decoration::MatrixStride,
                                    [rspirv::dr::Operand::LiteralInt32(
                                        p.matrix_stride().unwrap(),
                                    )],
                                );
                                b.member_decorate(
                                    spv_ty_object,
                                    index,
                                    rspirv::spirv::Decoration::ColMajor,
                                    [],
                                );
                            }
                        }
                        offset += ty.size();
                        index += 1;
                    }

                    spv_ty_object
                }
            }
        }
    }

    pub(crate) fn pointer_type(
        &self,
        b: &mut rspirv::dr::Builder,
        struct_map: &mut HashMap<TypeId, u32>,
    ) -> u32 {
        let b_ty = self.base_type(b, struct_map);
        b.type_pointer(None, rspirv::spirv::StorageClass::Function, b_ty)
    }

    pub(crate) fn variable(
        &self,
        b: &mut rspirv::dr::Builder,
        struct_map: &mut HashMap<TypeId, u32>,
        var_block: usize,
    ) -> u32 {
        let p_ty = self.pointer_type(b, struct_map);
        let current_block = b.selected_block().unwrap();
        b.select_block(Some(var_block)).unwrap();
        let id = b.id();
        b.insert_into_block(
            rspirv::dr::InsertPoint::Begin,
            rspirv::dr::Instruction::new(
                rspirv::spirv::Op::Variable,
                Some(p_ty),
                Some(id),
                vec![rspirv::dr::Operand::StorageClass(
                    rspirv::spirv::StorageClass::Function,
                )],
            ),
        )
        .unwrap();
        b.select_block(Some(current_block)).unwrap();
        id
    }
}

impl From<DataVal> for DataType {
    fn from(v: DataVal) -> Self {
        Self::from(&v)
    }
}

impl From<&'_ DataVal> for DataType {
    fn from(v: &'_ DataVal) -> Self {
        match v {
            DataVal::Primitive(p) => Self::Primitive(PrimitiveType::from(*p)),
            DataVal::Array(a) => Self::Array(PrimitiveType::from(a[0]), a.len()),
            DataVal::Struct(_, _) => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataVal {
    Primitive(PrimitiveVal),
    Array(Vec<PrimitiveVal>),
    Struct(&'static [&'static str], Vec<DataVal>),
}

impl DataVal {
    pub(crate) fn set_constant(
        &self,
        b: &mut rspirv::dr::Builder,
        struct_map: &mut HashMap<TypeId, u32>,
    ) -> (u32, u32) {
        match self {
            DataVal::Primitive(p) => p.set_constant(b),
            DataVal::Array(v) => {
                let ty = DataType::from(self).base_type(b, struct_map);
                let components = v.iter().map(|c| c.set_constant(b).0).collect::<Vec<_>>();
                (b.constant_composite(ty, components), ty)
            }
            DataVal::Struct(_, _) => todo!(),
        }
    }

    pub fn set(&self, b: &mut rspirv::dr::Builder, struct_map: &mut HashMap<TypeId, u32>) -> u32 {
        let (c, ty) = self.set_constant(b, struct_map);
        let p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Function, ty);
        let var = b.variable(p_ty, None, rspirv::spirv::StorageClass::Function, None);
        b.store(var, c, None, None).unwrap();
        var
    }
}
