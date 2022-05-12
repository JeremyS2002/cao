
pub mod as_ty;
pub mod ty_structs;
pub mod ops;

pub use as_ty::*;
pub use ty_structs::*;
pub use ops::*;

pub trait DataRef {
    fn ty(&self) -> DataType;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DataType {
    Bool,
    Int,
    UInt,
    Float,
    Double,
    BVec2,
    BVec3,
    BVec4,
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DataVal {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    BVec2(glam::BVec2),
    BVec3(glam::BVec3),
    BVec4(glam::BVec4),
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
