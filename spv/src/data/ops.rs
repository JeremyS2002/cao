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

macro_rules! impl_assign_op {
    ($name:ident, $rust:ident, $op:ident) => {
        impl $op<$name> for $name {}
        impl $op<$rust> for $name {}
    };
}

macro_rules! impl_math {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl_op!($name, $rust, SpvAdd);
            impl_op!($name, $rust, SpvSub);
            impl_op!($name, $rust, SpvDiv);
            impl_op!($name, $rust, SpvMul);
            impl_assign_op!($name, $rust, SpvAddAssign);
            impl_assign_op!($name, $rust, SpvSubAssign);
            impl_assign_op!($name, $rust, SpvDivAssign);
            impl_assign_op!($name, $rust, SpvMulAssign);
        )*
    };
}

impl_math!(
    Int, i32, UInt, u32, Float, f32, Double, f64, IVec2, RustIVec2, IVec3, RustIVec3, IVec4,
    RustIVec4, UVec2, RustUVec2, UVec3, RustUVec3, UVec4, RustUVec4, Vec2, RustVec2, Vec3,
    RustVec3, Vec4, RustVec4, DVec2, RustDVec2, DVec3, RustDVec3, DVec4, RustDVec4, Mat2, RustMat2,
    Mat3, RustMat3, Mat4, RustMat4, DMat2, RustDMat2, DMat3, RustDMat3, DMat4, RustDMat4,
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
    ($scalar:ident, $rust_scalar:ident, $ty:ident, $rust_ty:ident, $op:ident) => {
        impl $op<$scalar> for $ty {
            type Output = $ty;
        }

        impl $op<$rust_scalar> for $ty {
            type Output = $ty;
        }

        impl $op<$scalar> for $rust_ty {
            type Output = $ty;
        }

        impl $op<$rust_scalar> for $rust_ty {
            type Output = $ty;
        }

        impl $op<$ty> for $scalar {
            type Output = $ty;
        }

        impl $op<$rust_ty> for $scalar {
            type Output = $ty;
        }

        impl $op<$ty> for $rust_scalar {
            type Output = $ty;
        }

        impl $op<$rust_ty> for $rust_scalar {
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
    ($($scalar:ident, $rust_scalar:ident, $ty:ident, $rust_ty:ident,)*) => {
        $(
            impl_scalar_ty_op!($scalar, $rust_scalar, $ty, $rust_ty, SpvMul);
            impl_scalar_ty_op!($scalar, $rust_scalar, $ty, $rust_ty, SpvDiv);
            impl_scalar_ty_assign_ops!($scalar, $rust_scalar, $ty, SpvMulAssign);
            impl_scalar_ty_assign_ops!($scalar, $rust_scalar, $ty, SpvDivAssign);
        )*
    };
}

impl_scalar_ty_ops!(
    Int, i32, IVec2, RustIVec2, Int, i32, IVec3, RustIVec3, Int, i32, IVec4, RustIVec4, UInt, u32,
    UVec2, RustUVec2, UInt, u32, UVec3, RustUVec3, UInt, u32, UVec4, RustUVec4, Float, f32, Vec2,
    RustVec2, Float, f32, Vec3, RustVec3, Float, f32, Vec4, RustVec4, Float, f32, Mat2, RustMat2,
    Float, f32, Mat3, RustMat3, Float, f32, Mat4, RustMat4, Double, f64, DVec2, RustDVec2, Double,
    f64, DVec3, RustDVec3, Double, f64, DVec4, RustDVec4, Double, f64, DMat2, RustDMat2, Double,
    f64, DMat3, RustDMat3, Double, f64, DMat4, RustDMat4,
);

macro_rules! impl_bitwise {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl_op!($name, $rust, SpvBitAnd);
            impl_op!($name, $rust, SpvBitOr);
            impl_op!($name, $rust, SpvBitXor);
            impl_assign_op!($name, $rust, SpvBitAndAssign);
            impl_assign_op!($name, $rust, SpvBitOrAssign);
            impl_assign_op!($name, $rust, SpvBitXorAssign);
        )*
    };
}

impl_bitwise!(Int, i32, UInt, u32,);
