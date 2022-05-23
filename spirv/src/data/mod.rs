
use std::collections::HashMap;

pub mod as_ty;
pub mod ty_structs;
pub mod ops;
pub mod spv_array;
pub mod spv_struct;

pub use as_ty::*;
pub use ty_structs::*;
pub use ops::*;
pub use spv_array::*;
pub use spv_struct::*;

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
    pub fn is_matrix(&self) -> bool {
        match self {
            Self::Mat2
            | Self::Mat3
            | Self::Mat4
            | Self::DMat2
            | Self::DMat3
            | Self::DMat4 => true,
            _ => false
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
            Self::Bool 
            | Self::Int
            | Self::UInt
            | Self::Float
            | Self::Double => true,
            _ => false
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Self::Int
            | Self::IVec2
            | Self::IVec3
            | Self::IVec4 => true,
            _ => false,
        }
    }

    pub fn is_uint(&self) -> bool {
        match self {
            Self::UInt
            | Self::UVec2
            | Self::UVec3
            | Self::UVec4 => true,
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

    pub fn raw_ty(&self, b: &mut rspirv::dr::Builder) -> u32 {
        match self {
            PrimitiveType::Bool => b.type_bool(),
            PrimitiveType::Int => b.type_int(32, 1),
            PrimitiveType::UInt => b.type_int(32, 0),
            PrimitiveType::Float => b.type_float(32),
            PrimitiveType::Double => b.type_float(64),
            PrimitiveType::IVec2 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 2)
            },
            PrimitiveType::IVec3 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 3)
            },
            PrimitiveType::IVec4 => {
                let r = b.type_int(32, 1);
                b.type_vector(r, 4)
            },
            PrimitiveType::UVec2 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 2)
            },
            PrimitiveType::UVec3 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 3)
            },
            PrimitiveType::UVec4 => {
                let r = b.type_int(32, 0);
                b.type_vector(r, 4)
            },
            PrimitiveType::Vec2 => {
                let r = b.type_float(32);
                b.type_vector(r, 2)
            },
            PrimitiveType::Vec3 => {
                let r = b.type_float(32);
                b.type_vector(r, 3)
            },
            PrimitiveType::Vec4 => {
                let r = b.type_float(32);
                b.type_vector(r, 4)
            },
            PrimitiveType::DVec2 => {
                let r = b.type_float(64);
                b.type_vector(r, 2)
            },
            PrimitiveType::DVec3 => {
                let r = b.type_float(64);
                b.type_vector(r, 3)
            },
            PrimitiveType::DVec4 => {
                let r = b.type_float(64);
                b.type_vector(r, 4)
            },
            PrimitiveType::Mat2 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 2);
                b.type_matrix(v, 2)
            },
            PrimitiveType::Mat3 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 3);
                b.type_matrix(v, 3)
            },
            PrimitiveType::Mat4 => {
                let r = b.type_float(32);
                let v = b.type_vector(r, 4);
                b.type_matrix(v, 4)
            },
            PrimitiveType::DMat2 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 2);
                b.type_matrix(v, 2)
            },
            PrimitiveType::DMat3 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 3);
                b.type_matrix(v, 3)
            },
            PrimitiveType::DMat4 => {
                let r = b.type_float(64);
                let v = b.type_vector(r, 4);
                b.type_matrix(v, 4)
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PrimitiveVal {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    IVec2(glam::IVec2),
    IVec3(glam::IVec3),
    IVec4(glam::IVec4),
    UVec2(glam::UVec2),
    UVec3(glam::UVec3),
    UVec4(glam::UVec4),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    DVec2(glam::DVec2),
    DVec3(glam::DVec3),
    DVec4(glam::DVec4),
    Mat2(glam::Mat2),
    Mat3(glam::Mat3),
    Mat4(glam::Mat4),
    DMat2(glam::DMat2),
    DMat3(glam::DMat3),
    DMat4(glam::DMat4),
}

// impl From<DataVal> for DataType {
//     fn from(v: DataVal) -> Self {
//         v.into()
//     }
// }

impl From<PrimitiveVal> for PrimitiveType {
    fn from(v: PrimitiveVal) -> Self {
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
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = if *v {
                    b.constant_true(ty)
                } else {
                    b.constant_false(ty)
                };
                (c, ty)
            },
            PrimitiveVal::Int(v) => {
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_u32(ty, unsafe { std::mem::transmute(*v) });
                (c, ty)
            },
            PrimitiveVal::UInt(v) => {
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_u32(ty, *v);
                (c, ty)
            },
            PrimitiveVal::Float(v) => {
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_f32(ty, *v);
                (c, ty)
            },
            PrimitiveVal::Double(v) => {
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_f64(ty, *v);
                (c, ty)
            },
            PrimitiveVal::IVec2(v) => {
                let x = Self::Int(v.x).set_constant(b).0;
                let y = Self::Int(v.y).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::IVec3(v) => {
                let x = Self::Int(v.x).set_constant(b).0;
                let y = Self::Int(v.y).set_constant(b).0;
                let z = Self::Int(v.z).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::IVec4(v) => {
                let x = Self::Int(v.x).set_constant(b).0;
                let y = Self::Int(v.y).set_constant(b).0;
                let z = Self::Int(v.z).set_constant(b).0;
                let w = Self::Int(v.w).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
            PrimitiveVal::UVec2(v) => {
                let x = Self::UInt(v.x).set_constant(b).0;
                let y = Self::UInt(v.y).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::UVec3(v) => {
                let x = Self::UInt(v.x).set_constant(b).0;
                let y = Self::UInt(v.y).set_constant(b).0;
                let z = Self::UInt(v.z).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::UVec4(v) => {
                let x = Self::UInt(v.x).set_constant(b).0;
                let y = Self::UInt(v.y).set_constant(b).0;
                let z = Self::UInt(v.z).set_constant(b).0;
                let w = Self::UInt(v.w).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
            PrimitiveVal::Vec2(v) => {
                let x = Self::Float(v.x).set_constant(b).0;
                let y = Self::Float(v.y).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::Vec3(v) => {
                let x = Self::Float(v.x).set_constant(b).0;
                let y = Self::Float(v.y).set_constant(b).0;
                let z = Self::Float(v.z).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::Vec4(v) => {
                let x = Self::Float(v.x).set_constant(b).0;
                let y = Self::Float(v.y).set_constant(b).0;
                let z = Self::Float(v.z).set_constant(b).0;
                let w = Self::Float(v.w).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
            PrimitiveVal::DVec2(v) => {
                let x = Self::Double(v.x).set_constant(b).0;
                let y = Self::Double(v.y).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::DVec3(v) => {
                let x = Self::Double(v.x).set_constant(b).0;
                let y = Self::Double(v.y).set_constant(b).0;
                let z = Self::Double(v.z).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::DVec4(v) => {
                let x = Self::Double(v.x).set_constant(b).0;
                let y = Self::Double(v.y).set_constant(b).0;
                let z = Self::Double(v.z).set_constant(b).0;
                let w = Self::Double(v.w).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
            PrimitiveVal::Mat2(v) => {
                let x = Self::Vec2(v.col(0)).set_constant(b).0;
                let y = Self::Vec2(v.col(1)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::Mat3(v) => {
                let x = Self::Vec3(v.col(0)).set_constant(b).0;
                let y = Self::Vec3(v.col(1)).set_constant(b).0;
                let z = Self::Vec3(v.col(2)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::Mat4(v) => {
                let x = Self::Vec4(v.col(0)).set_constant(b).0;
                let y = Self::Vec4(v.col(1)).set_constant(b).0;
                let z = Self::Vec4(v.col(2)).set_constant(b).0;
                let w = Self::Vec4(v.col(3)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
            PrimitiveVal::DMat2(v) => {
                let x = Self::DVec2(v.col(0)).set_constant(b).0;
                let y = Self::DVec2(v.col(1)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y]);
                (c, ty)
            },
            PrimitiveVal::DMat3(v) => {
                let x = Self::DVec3(v.col(0)).set_constant(b).0;
                let y = Self::DVec3(v.col(1)).set_constant(b).0;
                let z = Self::DVec3(v.col(2)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z]);
                (c, ty)
            },
            PrimitiveVal::DMat4(v) => {
                let x = Self::DVec4(v.col(0)).set_constant(b).0;
                let y = Self::DVec4(v.col(1)).set_constant(b).0;
                let z = Self::DVec4(v.col(2)).set_constant(b).0;
                let w = Self::DVec4(v.col(3)).set_constant(b).0;
                let ty = PrimitiveType::from(*self).raw_ty(b);
                let c = b.constant_composite(ty, [x, y, z, w]);
                (c, ty)
            },
        }
    }

    pub fn set(&self, b: &mut rspirv::dr::Builder) -> u32 {
        let (c, ty) = self.set_constant(b);
        let p_ty = b.type_pointer(
            None, 
            rspirv::spirv::StorageClass::Function,
            ty,
        );
        let var = b.variable(
            p_ty, 
            None, 
            rspirv::spirv::StorageClass::Function,
            None,
        );
        b.store(var, c, None, None).unwrap();
        var
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataType {
    Primitive(PrimitiveType),
    Array(PrimitiveType, usize),
    Struct(&'static [&'static str], &'static [DataType]),
}

impl DataType {
    pub fn raw_ty(&self, b: &mut rspirv::dr::Builder, struct_map: &mut HashMap<(usize, usize), u32>) -> u32 {
        match self {
            Self::Primitive(ty) => ty.raw_ty(b),
            Self::Array(ty, n) => {
                let p = ty.raw_ty(b);
                b.type_array(p, *n as u32)
            },
            Self::Struct(names, types) => {
                // Future me, probably don't change this unless you know what you're doing
                let names_p = (*names).as_ptr() as usize;
                let types_p = (&types).as_ptr() as usize;
                if let Some(spv_type_object) = struct_map.get(&(names_p, types_p)) {
                    *spv_type_object
                } else {
                    let spv_types = types
                        .iter()
                        .map(|t| {
                            t.raw_ty(b, struct_map)
                        })
                        .collect::<Vec<_>>();
            
                    let spv_ty_object = b.type_struct(spv_types);

                    spv_ty_object
                }
            }
        }
    }
}

impl From<DataVal> for DataType {
    fn from(v: DataVal) -> Self {
        match v {
            DataVal::Primitive(p) => Self::Primitive(PrimitiveType::from(p)),
            DataVal::Array(a) => Self::Array(PrimitiveType::from(a[0]), a.len()),
            DataVal::Struct(_, _) => todo!(),
        }
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
    pub(crate) fn set_constant(&self, b: &mut rspirv::dr::Builder, struct_map: &mut HashMap<(usize, usize), u32>) -> (u32, u32) {
        match self {
            DataVal::Primitive(p) => p.set_constant(b),
            DataVal::Array(v) => {
                let ty = DataType::from(self).raw_ty(b, struct_map);
                let components = v
                    .iter()
                    .map(|c| {
                        c.set_constant(b).0
                    })
                    .collect::<Vec<_>>();
                (b.constant_composite(ty, components), ty)
            },
            DataVal::Struct(_, _) => todo!(),
        }
    }

    pub fn set(&self, b: &mut rspirv::dr::Builder, struct_map: &mut HashMap<(usize, usize), u32>) -> u32 {
        let (c, ty) = self.set_constant(b, struct_map);
        let p_ty = b.type_pointer(
            None, 
            rspirv::spirv::StorageClass::Function,
            ty,
        );
        let var = b.variable(
            p_ty, 
            None, 
            rspirv::spirv::StorageClass::Function,
            None,
        );
        b.store(var, c, None, None).unwrap();
        var
    }
}