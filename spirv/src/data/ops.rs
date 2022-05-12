
#![allow(unused_imports)]

// If I knew how to write macros properly this wouldn't be here but this is easier than learning proper macros
use glam::BVec2 as GlamBVec2;
use glam::BVec3 as GlamBVec3;
use glam::BVec4 as GlamBVec4;
use glam::IVec2 as GlamIVec2;
use glam::IVec3 as GlamIVec3;
use glam::IVec4 as GlamIVec4;
use glam::UVec2 as GlamUVec2;
use glam::UVec3 as GlamUVec3;
use glam::UVec4 as GlamUVec4;
use glam::Vec2 as GlamVec2;
use glam::Vec3 as GlamVec3;
use glam::Vec4 as GlamVec4;
use glam::DVec2 as GlamDVec2;
use glam::DVec3 as GlamDVec3;
use glam::DVec4 as GlamDVec4;
use glam::Mat2 as GlamMat2;
use glam::Mat3 as GlamMat3;
use glam::Mat4 as GlamMat4;
use glam::DMat2 as GlamDMat2;
use glam::DMat3 as GlamDMat3;
use glam::DMat4 as GlamDMat4;

use super::ty_structs::*;

pub trait SpvAdd<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}
pub trait SpvSub<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}
pub trait SpvMul<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}
pub trait SpvDiv<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}
pub trait SpvBitAnd<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}
pub trait SpvBitOr<Rhs: AsDataType + AsData>: AsDataType + AsData {
    type Output: FromId + AsDataType;
}
pub trait SpvBitXor<Rhs: AsDataType + AsData>: AsDataType + AsData { 
    type Output: FromId + AsDataType;
}

macro_rules! impl_op {
    ($name:ident, $rust:ident, $op:ident) => {
        impl $op<$name> for $name {
            type Output = $name;
        }

        impl $op<$rust> for $name {
            type Output = $name;
        }

        impl $op<$name> for $rust {
            type Output = $name;
        }

        impl $op<$rust> for $rust {
            type Output = $name;
        }
    };
}

macro_rules! impl_math {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl_op!($name, $rust, SpvAdd);
            impl_op!($name, $rust, SpvSub);
            impl_op!($name, $rust, SpvDiv);
            impl_op!($name, $rust, SpvMul);
        )*
    };
}

impl_math!(
    Int, i32,
    UInt, u32,
    Float, f32,
    Double, f64,
    IVec2, GlamIVec2,
    IVec3, GlamIVec3,
    IVec4, GlamIVec4,
    UVec2, GlamUVec2,
    UVec3, GlamUVec3,
    UVec4, GlamUVec4,
    Vec2, GlamVec2,
    Vec3, GlamVec3,
    Vec4, GlamVec4,
    DVec2, GlamDVec2,
    DVec3, GlamDVec3,
    DVec4, GlamDVec4,
    Mat2, GlamMat2,
    Mat3, GlamMat3,
    Mat4, GlamMat4,
    DMat2, GlamDMat2,
    DMat3, GlamDMat3,
    DMat4, GlamDMat4,
);

macro_rules! impl_mat_vec_op {
    ($vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident) => {
        impl SpvMul<$vec> for $mat {
            type Output = $vec;
        }

        impl SpvMul<$rust_vec> for $mat {
            type Output = $vec;
        }

        impl SpvMul<$vec> for $rust_mat {
            type Output = $vec;
        }

        impl SpvMul<$rust_vec> for $rust_mat {
            type Output = $vec;
        }
    };
}

macro_rules! impl_mat_vec {
    ($($vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident,)*) => {
        $(
            impl_mat_vec_op!($vec, $rust_vec, $mat, $rust_mat);
        )*
    };
}

impl_mat_vec!(
    Vec2, GlamVec2, Mat2, GlamMat2,
    Vec3, GlamVec3, Mat3, GlamMat3,
    Vec4, GlamVec4, Mat4, GlamMat4,
    DVec2, GlamDVec2, DMat2, GlamDMat2,
    DVec3, GlamDVec3, DMat3, GlamDMat3,
    DVec4, GlamDVec4, DMat4, GlamDMat4,
);