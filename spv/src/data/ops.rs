#![allow(unused_imports)]

// If I knew how to write macros properly this wouldn't be here but this is easier than learning proper macros
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

#[cfg(feature = "glam")]
use glam::{
    DMat2 as GlamDMat2, DMat3 as GlamDMat3, DMat4 as GlamDMat4, DVec2 as GlamDVec2,
    DVec3 as GlamDVec3, DVec4 as GlamDVec4, IVec2 as GlamIVec2, IVec3 as GlamIVec3,
    IVec4 as GlamIVec4, Mat2 as GlamMat2, Mat3 as GlamMat3, Mat4 as GlamMat4, UVec2 as GlamUVec2,
    UVec3 as GlamUVec3, UVec4 as GlamUVec4, Vec2 as GlamVec2, Vec3 as GlamVec3, Vec4 as GlamVec4,
};

use super::ty_structs::*;

pub trait SpvAdd<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvSub<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvMul<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvDiv<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}

pub trait SpvAddAssign<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {}
pub trait SpvSubAssign<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {}
pub trait SpvMulAssign<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {}
pub trait SpvDivAssign<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {}

pub trait SpvBitAnd<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvBitOr<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvBitXor<Rhs: AsPrimitiveType + AsPrimitive>: AsPrimitiveType + AsPrimitive {
    type Output: FromId + AsPrimitiveType;
}
pub trait SpvBitAndAssign<Rhs: AsPrimitiveType + AsPrimitive>:
    AsPrimitiveType + AsPrimitive
{
}
pub trait SpvBitOrAssign<Rhs: AsPrimitiveType + AsPrimitive>:
    AsPrimitiveType + AsPrimitive
{
}
pub trait SpvBitXorAssign<Rhs: AsPrimitiveType + AsPrimitive>:
    AsPrimitiveType + AsPrimitive
{
}

macro_rules! impl_op_primitive {
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

macro_rules! impl_op_vec_mat {
    ($name:ident, $rust:ident, $glm:ident, $op:ident) => {
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

        impl $op<$glm> for $name {
            type Output = $name;
        }

        impl $op<$name> for $glm {
            type Output = $name;
        }

        impl $op<$glm> for $glm {
            type Output = $name;
        }
    };
}

macro_rules! impl_assign_op_primitive {
    ($name:ident, $rust:ident, $op:ident) => {
        impl $op<$name> for $name {}
        impl $op<$rust> for $name {}
    };
}

macro_rules! impl_assign_op_vec_mat {
    ($name:ident, $rust:ident, $glm:ident, $op:ident) => {
        impl $op<$name> for $name {}
        impl $op<$rust> for $name {}
        impl $op<$glm> for $name {}
    };
}

macro_rules! impl_math_primitive {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl_op_primitive!($name, $rust, SpvAdd);
            impl_op_primitive!($name, $rust, SpvSub);
            impl_op_primitive!($name, $rust, SpvDiv);
            impl_op_primitive!($name, $rust, SpvMul);
            impl_assign_op_primitive!($name, $rust, SpvAddAssign);
            impl_assign_op_primitive!($name, $rust, SpvSubAssign);
            impl_assign_op_primitive!($name, $rust, SpvDivAssign);
            impl_assign_op_primitive!($name, $rust, SpvMulAssign);
        )*
    };
}

macro_rules! impl_math_vec_mat {
    ($($name:ident, $rust:ident, $glm:ident,)*) => {
        $(
            impl_op_vec_mat!($name, $rust, $glm, SpvAdd);
            impl_op_vec_mat!($name, $rust, $glm, SpvSub);
            impl_op_vec_mat!($name, $rust, $glm, SpvDiv);
            impl_op_vec_mat!($name, $rust, $glm, SpvMul);
            impl_assign_op_vec_mat!($name, $rust, $glm, SpvAddAssign);
            impl_assign_op_vec_mat!($name, $rust, $glm, SpvSubAssign);
            impl_assign_op_vec_mat!($name, $rust, $glm, SpvDivAssign);
            impl_assign_op_vec_mat!($name, $rust, $glm, SpvMulAssign);
        )*
    };
}

#[rustfmt::skip]
impl_math_primitive!(
    Int, i32,
    UInt, u32,
    Float, f32,
    Double, f64,
);

#[rustfmt::skip]
impl_math_vec_mat!(
    IVec2, RustIVec2, GlamIVec2,
    IVec3, RustIVec3, GlamIVec3,
    IVec4, RustIVec4, GlamIVec4,
    UVec2, RustUVec2, GlamUVec2,
    UVec3, RustUVec3, GlamUVec3,
    UVec4, RustUVec4, GlamUVec4,
    Vec2, RustVec2, GlamVec2,
    Vec3, RustVec3, GlamVec3,
    Vec4, RustVec4, GlamVec4,
    DVec2, RustDVec2, GlamDVec2,
    DVec3, RustDVec3, GlamDVec3,
    DVec4, RustDVec4, GlamDVec4,
    Mat2, RustMat2, GlamMat2,
    Mat3, RustMat3, GlamMat3,
    Mat4, RustMat4, GlamMat4,
    DMat2, RustDMat2, GlamDMat2,
    DMat3, RustDMat3, GlamDMat3,
    DMat4, RustDMat4, GlamDMat4,
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

macro_rules! impl_mat_vec_assign_op {
    ($vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident) => {
        impl SpvMulAssign<$mat> for $vec {}

        impl SpvMulAssign<$rust_mat> for $vec {}
    };
}

macro_rules! impl_mat_vec {
    ($($vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident,)*) => {
        $(
            impl_mat_vec_op!($vec, $rust_vec, $mat, $rust_mat);
            impl_mat_vec_assign_op!($vec, $rust_vec, $mat, $rust_mat);
        )*
    };
}

impl_mat_vec!(
    Vec2, RustVec2, Mat2, RustMat2, Vec3, RustVec3, Mat3, RustMat3, Vec4, RustVec4, Mat4, RustMat4,
    DVec2, RustDVec2, DMat2, RustDMat2, DVec3, RustDVec3, DMat3, RustDMat3, DVec4, RustDVec4,
    DMat4, RustDMat4,
);

macro_rules! impl_scalar_ty_op {
    ($scalar:ident, $rust_scalar:ident, $ty:ident, $rust_ty:ident, $glm:ident, $op:ident) => {
        impl $op<$scalar> for $ty {
            type Output = $ty;
        }

        impl $op<$rust_scalar> for $ty {
            type Output = $ty;
        }

        impl $op<$scalar> for $rust_ty {
            type Output = $ty;
        }

        impl $op<$scalar> for $glm {
            type Output = $ty;
        }

        impl $op<$rust_scalar> for $rust_ty {
            type Output = $ty;
        }

        impl $op<$rust_scalar> for $glm {
            type Output = $ty;
        }

        impl $op<$ty> for $scalar {
            type Output = $ty;
        }

        impl $op<$rust_ty> for $scalar {
            type Output = $ty;
        }

        impl $op<$glm> for $scalar {
            type Output = $ty;
        }

        impl $op<$ty> for $rust_scalar {
            type Output = $ty;
        }

        impl $op<$rust_ty> for $rust_scalar {
            type Output = $ty;
        }

        impl $op<$glm> for $rust_scalar {
            type Output = $ty;
        }
    };
}

macro_rules! impl_scalar_ty_assign_ops {
    ($scalar:ident, $rust_scalar:ident, $ty:ident, $op:ident) => {
        impl $op<$scalar> for $ty {}

        impl $op<$rust_scalar> for $ty {}
    };
}

macro_rules! impl_scalar_ty_ops {
    ($($scalar:ident, $rust_scalar:ident, $ty:ident, $rust_ty:ident, $glm:ident,)*) => {
        $(
            impl_scalar_ty_op!($scalar, $rust_scalar, $ty, $rust_ty, $glm, SpvMul);
            impl_scalar_ty_op!($scalar, $rust_scalar, $ty, $rust_ty, $glm, SpvDiv);
            impl_scalar_ty_assign_ops!($scalar, $rust_scalar, $ty, SpvMulAssign);
            impl_scalar_ty_assign_ops!($scalar, $rust_scalar, $ty, SpvDivAssign);
        )*
    };
}

#[rustfmt::skip]
impl_scalar_ty_ops!(
    Int, i32, IVec2, RustIVec2, GlamIVec2,
    Int, i32, IVec3, RustIVec3, GlamIVec3,
    Int, i32, IVec4, RustIVec4, GlamIVec4,
    UInt, u32, UVec2, RustUVec2, GlamUVec2,
    UInt, u32, UVec3, RustUVec3, GlamUVec3,
    UInt, u32, UVec4, RustUVec4, GlamUVec4,
    Float, f32, Vec2, RustVec2, GlamVec2,
    Float, f32, Vec3, RustVec3, GlamVec3,
    Float, f32, Vec4, RustVec4, GlamVec4,
    Float, f32, Mat2, RustMat2, GlamMat2,
    Float, f32, Mat3, RustMat3, GlamMat3,
    Float, f32, Mat4, RustMat4, GlamMat4,
    Double, f64, DVec2, RustDVec2, GlamDVec2,
    Double, f64, DVec3, RustDVec3, GlamDVec3,
    Double, f64, DVec4, RustDVec4, GlamDVec4,
    Double, f64, DMat2, RustDMat2, GlamDMat2,
    Double, f64, DMat3, RustDMat3, GlamDMat3,
    Double, f64, DMat4, RustDMat4, GlamDMat4,
);

macro_rules! impl_bitwise {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl_op_primitive!($name, $rust, SpvBitAnd);
            impl_op_primitive!($name, $rust, SpvBitOr);
            impl_op_primitive!($name, $rust, SpvBitXor);
            impl_assign_op_primitive!($name, $rust, SpvBitAndAssign);
            impl_assign_op_primitive!($name, $rust, SpvBitOrAssign);
            impl_assign_op_primitive!($name, $rust, SpvBitXorAssign);
        )*
    };
}

impl_bitwise!(Int, i32, UInt, u32,);

pub trait SpvInto<T: IsPrimitiveType>: IsPrimitiveType + AsPrimitive {}

macro_rules! impl_spv_into {
    ($($a:ident, $b:ident,)*) => {
        $(
            impl SpvInto<$b> for $a { }
        )*
    };
}

#[rustfmt::skip]
impl_spv_into!(
    Int, UInt,
    Int, Float,
    Int, Double,
    UInt, Int,
    UInt, Float,
    UInt, Double,
    Float, Int,
    Float, UInt,
    Float, Double,
    Double, Int,
    Double, UInt,
    Double, Float,
);

pub trait SpvEq<T: AsPrimitive>: AsPrimitive {}
pub trait SpvNEq<T: AsPrimitive>: AsPrimitive {}
pub trait SpvLt<T: AsPrimitive>: AsPrimitive {}
pub trait SpvGt<T: AsPrimitive>: AsPrimitive {}
pub trait SpvLe<T: AsPrimitive>: AsPrimitive {}
pub trait SpvGe<T: AsPrimitive>: AsPrimitive {}

macro_rules! impl_cmp {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl SpvEq<$name> for $name { }
            impl SpvEq<$rust> for $name { }
            impl SpvEq<$name> for $rust { }
            impl SpvNEq<$name> for $name { }
            impl SpvNEq<$rust> for $name { }
            impl SpvNEq<$name> for $rust { }
            impl SpvLt<$name> for $name { }
            impl SpvLt<$rust> for $name { }
            impl SpvLt<$name> for $rust { }
            impl SpvGt<$name> for $name { }
            impl SpvGt<$rust> for $name { }
            impl SpvGt<$name> for $rust { }
            impl SpvLe<$name> for $name { }
            impl SpvLe<$rust> for $name { }
            impl SpvLe<$name> for $rust { }
            impl SpvGe<$name> for $name { }
            impl SpvGe<$rust> for $name { }
            impl SpvGe<$name> for $rust { }
        )*
    };
}

impl_cmp!(Int, i32, UInt, u32, Float, f32, Double, f64,);
