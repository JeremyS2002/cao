
use std::marker::PhantomData;

use crate::builder::RawBuilder;

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

use super::PrimitiveType;

macro_rules! gen_types {
    ($($name:ident,)*) => {
        $(
            #[derive(Copy, Clone)]
            pub struct $name {
                pub(crate) id: usize,
            }
        )*
    };
}

gen_types!(
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
);

pub trait SpvRustEq<T: AsPrimitive>: AsPrimitive { }

pub trait AsPrimitiveType {
    const TY: crate::data::PrimitiveType;
}

pub trait AsPrimitive {
    fn id(&self, b: &dyn RawBuilder) -> usize;

    fn ty(&self) -> crate::data::PrimitiveType;
}

pub trait IsPrimitiveType {
    const TY: crate::data::PrimitiveType;
}

pub trait AsDataType {
    const TY: crate::data::DataType;
}

pub trait AsData {
    fn id(&self, b: &dyn RawBuilder) -> usize;

    fn ty(&self) -> crate::data::DataType;
}

pub trait IsDataType {
    const TY: crate::data::DataType;
}

pub trait FromId {
    fn from_id(id: usize) -> Self;
}

pub trait SpvStore<Rhs: AsPrimitiveType>: AsPrimitiveType + AsPrimitive { 
    fn val(rhs: Rhs) -> crate::data::PrimitiveVal;
}

macro_rules! gen_as_data {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl SpvRustEq<$name> for $name { }

            impl SpvRustEq<$rust> for $name { }

            impl SpvRustEq<$name> for $rust { }

            impl SpvStore<$rust> for $name { 
                fn val(rhs: $rust) -> crate::data::PrimitiveVal {
                    crate::data::PrimitiveVal::$name(rhs)
                }
            }

            impl AsPrimitiveType for $name {
                const TY: crate::data::PrimitiveType = crate::data::PrimitiveType::$name;
            }

            impl IsPrimitiveType for $name {
                const TY: crate::data::PrimitiveType = crate::data::PrimitiveType::$name;
            }

            impl AsPrimitive for $name {
                fn id(&self, _: &dyn RawBuilder) -> usize {
                    self.id
                }

                fn ty(&self) -> crate::data::PrimitiveType {
                    <Self as AsPrimitiveType>::TY
                }
            }

            impl AsPrimitiveType for $rust {
                const TY: crate::data::PrimitiveType = crate::data::PrimitiveType::$name;
            }

            impl AsPrimitive for $rust {
                fn id(&self, b: &dyn RawBuilder) -> usize {
                    let id = b.get_new_id();
                    b.push_instruction(crate::builder::Instruction::Store {
                        val: crate::data::PrimitiveVal::$name(*self),
                        store: id,
                    });
                    id
                }

                fn ty(&self) -> crate::data::PrimitiveType {
                    <Self as AsPrimitiveType>::TY
                }
            }

            impl AsDataType for $name {
                const TY: crate::data::DataType = crate::data::DataType::Primitive(crate::data::PrimitiveType::$name);
            }

            impl IsDataType for $name {
                const TY: crate::data::DataType = crate::data::DataType::Primitive(crate::data::PrimitiveType::$name);
            }

            impl AsData for $name {
                fn id(&self, _: &dyn RawBuilder) -> usize {
                    self.id
                }

                fn ty(&self) -> crate::data::DataType {
                    <Self as AsDataType>::TY
                }
            }

            impl AsDataType for $rust {
                const TY: crate::data::DataType = crate::data::DataType::Primitive(crate::data::PrimitiveType::$name);
            }

            impl AsData for $rust {
                fn id(&self, b: &dyn RawBuilder) -> usize {
                    let id = b.get_new_id();
                    b.push_instruction(crate::builder::Instruction::Store {
                        val: crate::data::PrimitiveVal::$name(*self),
                        store: id,
                    });
                    id
                }

                fn ty(&self) -> crate::data::DataType {
                    <Self as AsDataType>::TY
                }
            }

            impl FromId for $name {
                fn from_id(id: usize) -> Self {
                    Self { id }
                }
            }
        )*
    };
}

gen_as_data!(
    Bool, bool,
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

pub struct VectorShuffle<T: AsPrimitiveType> {
    pub(crate) src: usize,
    pub(crate) src_ty: PrimitiveType,
    pub(crate) components: [u32; 4],
    pub(crate) _marker: PhantomData<T>,
}

macro_rules! impl_unit_shuffle {
    ($base:ident, $unit:ident : [ $($f:ident, $component:literal,)* ] ) => {
        $(
            pub fn $f(&self) -> VectorShuffle<$unit> {
                VectorShuffle {
                    src: self.id,
                    src_ty: PrimitiveType::$base,
                    components: [$component, 0, 0, 0,],
                    _marker: PhantomData,
                }
            }
        )*
    };
}

macro_rules! impl_vec2_shuffle {
    ($base:ident, $vec2:ident : [ $($f:ident, $a:literal, $b:literal,)* ]) => {
        $(
            pub fn $f(&self) -> VectorShuffle<$vec2> {
                VectorShuffle {
                    src: self.id,
                    src_ty: PrimitiveType::$base,
                    components: [$a, $b, 0, 0,],
                    _marker: PhantomData,
                }
            }
        )*
    };
}

macro_rules! impl_vec3_shuffle {
    ($base:ident, $vec3:ident : [ $($f:ident, $a:literal, $b:literal, $c:literal,)* ]) => {
        $(
            pub fn $f(&self) -> VectorShuffle<$vec3> {
                VectorShuffle {
                    src: self.id,
                    src_ty: PrimitiveType::$base,
                    components: [$a, $b, $c, 0,],
                    _marker: PhantomData,
                }
            }
        )*
    };
}

macro_rules! impl_vec4_shuffle {
    ($base:ident, $vec4:ident : [ $($f:ident, $a:literal, $b:literal, $c:literal, $d:literal,)* ]) => {
        $(
            pub fn $f(&self) -> VectorShuffle<$vec4> {
                VectorShuffle {
                    src: self.id,
                    src_ty: PrimitiveType::$base,
                    components: [$a, $b, $c, $d,],
                    _marker: PhantomData,
                }
            }
        )*
    };
}

// TODO: automatic loops, the shuffles were written by a python
// script and pasted into the macro invocations, I don't know how
// to get macros to dynamically name functions like this
macro_rules! impl_vector_shuffle {
    ($($unit:ident, $vec2:ident, $vec3:ident, $vec4:ident,)*) => {
        $(
            impl $vec2 {
                impl_unit_shuffle!(
                    $vec2, $unit : [
                        x, 0,
                        y, 1,
                    ]
                );

                impl_vec2_shuffle!(
                    $vec2, $vec2 : [
                        xx, 0, 0,
                        xy, 0, 1,
                        yx, 1, 0,
                        yy, 1, 1, 
                    ]
                );
                
                impl_vec3_shuffle!(
                    $vec2, $vec3 : [
                        xxx, 0, 0, 0,
                        xxy, 0, 0, 1,
                        xyx, 0, 1, 0,
                        xyy, 0, 1, 1,
                        yxx, 1, 0, 0,
                        yxy, 1, 0, 1,
                        yyx, 1, 1, 0,
                        yyy, 1, 1, 1,
                    ]
                );

                impl_vec4_shuffle!(
                    $vec2, $vec4 : [
                        xxxx, 0, 0, 0, 0,
                        xxxy, 0, 0, 0, 1,
                        xxyx, 0, 0, 1, 0,
                        xxyy, 0, 0, 1, 1,
                        xyxx, 0, 1, 0, 0,
                        xyxy, 0, 1, 0, 1,
                        xyyx, 0, 1, 1, 0,
                        xyyy, 0, 1, 1, 1,
                        yxxx, 1, 0, 0, 0,
                        yxxy, 1, 0, 0, 1,
                        yxyx, 1, 0, 1, 0,
                        yxyy, 1, 0, 1, 1,
                        yyxx, 1, 1, 0, 0,
                        yyxy, 1, 1, 0, 1,
                        yyyx, 1, 1, 1, 0,
                        yyyy, 1, 1, 1, 1,
                    ]
                );
            }

            impl $vec3 {
                impl_unit_shuffle!(
                    $vec3, $unit : [
                        x, 0, 
                        y, 1,
                        z, 2,
                    ]
                );

                impl_vec2_shuffle!(
                    $vec3, $vec2 : [
                        xx, 0, 0,
                        xy, 0, 1,
                        xz, 0, 2,
                        yx, 1, 0,
                        yy, 1, 1,
                        yz, 1, 2,
                        zx, 2, 0,
                        zy, 2, 1,
                        zz, 2, 2,
                    ]
                );

                impl_vec3_shuffle!(
                    $vec2, $vec3 : [
                        xxx, 0, 0, 0,
                        xxy, 0, 0, 1,
                        xxz, 0, 0, 2,
                        xyx, 0, 1, 0,
                        xyy, 0, 1, 1,
                        xyz, 0, 1, 2,
                        xzx, 0, 2, 0,
                        xzy, 0, 2, 1,
                        xzz, 0, 2, 2,
                        yxx, 1, 0, 0,
                        yxy, 1, 0, 1,
                        yxz, 1, 0, 2,
                        yyx, 1, 1, 0,
                        yyy, 1, 1, 1,
                        yyz, 1, 1, 2,
                        yzx, 1, 2, 0,
                        yzy, 1, 2, 1,
                        yzz, 1, 2, 2,
                        zxx, 2, 0, 0,
                        zxy, 2, 0, 1,
                        zxz, 2, 0, 2,
                        zyx, 2, 1, 0,
                        zyy, 2, 1, 1,
                        zyz, 2, 1, 2,
                        zzx, 2, 2, 0,
                        zzy, 2, 2, 1,
                        zzz, 2, 2, 2,               
                    ]
                );

                impl_vec4_shuffle!(
                    $vec3, $vec4 : [
                        xxxx, 0, 0, 0, 0,
                        xxxy, 0, 0, 0, 1,
                        xxxz, 0, 0, 0, 2,
                        xxyx, 0, 0, 1, 0,
                        xxyy, 0, 0, 1, 1,
                        xxyz, 0, 0, 1, 2,
                        xxzx, 0, 0, 2, 0,
                        xxzy, 0, 0, 2, 1,
                        xxzz, 0, 0, 2, 2,
                        xyxx, 0, 1, 0, 0,
                        xyxy, 0, 1, 0, 1,
                        xyxz, 0, 1, 0, 2,
                        xyyx, 0, 1, 1, 0,
                        xyyy, 0, 1, 1, 1,
                        xyyz, 0, 1, 1, 2,
                        xyzx, 0, 1, 2, 0,
                        xyzy, 0, 1, 2, 1,
                        xyzz, 0, 1, 2, 2,
                        xzxx, 0, 2, 0, 0,
                        xzxy, 0, 2, 0, 1,
                        xzxz, 0, 2, 0, 2,
                        xzyx, 0, 2, 1, 0,
                        xzyy, 0, 2, 1, 1,
                        xzyz, 0, 2, 1, 2,
                        xzzx, 0, 2, 2, 0,
                        xzzy, 0, 2, 2, 1,
                        xzzz, 0, 2, 2, 2,
                        yxxx, 1, 0, 0, 0,
                        yxxy, 1, 0, 0, 1,
                        yxxz, 1, 0, 0, 2,
                        yxyx, 1, 0, 1, 0,
                        yxyy, 1, 0, 1, 1,
                        yxyz, 1, 0, 1, 2,
                        yxzx, 1, 0, 2, 0,
                        yxzy, 1, 0, 2, 1,
                        yxzz, 1, 0, 2, 2,
                        yyxx, 1, 1, 0, 0,
                        yyxy, 1, 1, 0, 1,
                        yyxz, 1, 1, 0, 2,
                        yyyx, 1, 1, 1, 0,
                        yyyy, 1, 1, 1, 1,
                        yyyz, 1, 1, 1, 2,
                        yyzx, 1, 1, 2, 0,
                        yyzy, 1, 1, 2, 1,
                        yyzz, 1, 1, 2, 2,
                        yzxx, 1, 2, 0, 0,
                        yzxy, 1, 2, 0, 1,
                        yzxz, 1, 2, 0, 2,
                        yzyx, 1, 2, 1, 0,
                        yzyy, 1, 2, 1, 1,
                        yzyz, 1, 2, 1, 2,
                        yzzx, 1, 2, 2, 0,
                        yzzy, 1, 2, 2, 1,
                        yzzz, 1, 2, 2, 2,
                        zxxx, 2, 0, 0, 0,
                        zxxy, 2, 0, 0, 1,
                        zxxz, 2, 0, 0, 2,
                        zxyx, 2, 0, 1, 0,
                        zxyy, 2, 0, 1, 1,
                        zxyz, 2, 0, 1, 2,
                        zxzx, 2, 0, 2, 0,
                        zxzy, 2, 0, 2, 1,
                        zxzz, 2, 0, 2, 2,
                        zyxx, 2, 1, 0, 0,
                        zyxy, 2, 1, 0, 1,
                        zyxz, 2, 1, 0, 2,
                        zyyx, 2, 1, 1, 0,
                        zyyy, 2, 1, 1, 1,
                        zyyz, 2, 1, 1, 2,
                        zyzx, 2, 1, 2, 0,
                        zyzy, 2, 1, 2, 1,
                        zyzz, 2, 1, 2, 2,
                        zzxx, 2, 2, 0, 0,
                        zzxy, 2, 2, 0, 1,
                        zzxz, 2, 2, 0, 2,
                        zzyx, 2, 2, 1, 0,
                        zzyy, 2, 2, 1, 1,
                        zzyz, 2, 2, 1, 2,
                        zzzx, 2, 2, 2, 0,
                        zzzy, 2, 2, 2, 1,
                        zzzz, 2, 2, 2, 2,
                    ]
                );
            }

            impl $vec4 {
                impl_unit_shuffle!(
                    $vec4, $unit : [
                        x, 0,
                        y, 1,
                        z, 2,
                        w, 3,
                    ]
                );

                impl_vec2_shuffle!(
                    $vec4, $vec2 : [
                        xx, 0, 0,
                        xy, 0, 1,
                        xz, 0, 2,
                        xw, 0, 3,
                        yx, 1, 0,
                        yy, 1, 1,
                        yz, 1, 2,
                        yw, 1, 3,
                        zx, 2, 0,
                        zy, 2, 1,
                        zz, 2, 2,
                        zw, 2, 3,
                        wx, 3, 0,
                        wy, 3, 1,
                        wz, 3, 2,
                        ww, 3, 3,
                    ]
                );

                impl_vec3_shuffle!(
                    $vec2, $vec3 : [
                        xxx, 0, 0, 0,
                        xxy, 0, 0, 1,
                        xxz, 0, 0, 2,
                        xxw, 0, 0, 3,
                        xyx, 0, 1, 0,
                        xyy, 0, 1, 1,
                        xyz, 0, 1, 2,
                        xyw, 0, 1, 3,
                        xzx, 0, 2, 0,
                        xzy, 0, 2, 1,
                        xzz, 0, 2, 2,
                        xzw, 0, 2, 3,
                        xwx, 0, 3, 0,
                        xwy, 0, 3, 1,
                        xwz, 0, 3, 2,
                        xww, 0, 3, 3,
                        yxx, 1, 0, 0,
                        yxy, 1, 0, 1,
                        yxz, 1, 0, 2,
                        yxw, 1, 0, 3,
                        yyx, 1, 1, 0,
                        yyy, 1, 1, 1,
                        yyz, 1, 1, 2,
                        yyw, 1, 1, 3,
                        yzx, 1, 2, 0,
                        yzy, 1, 2, 1,
                        yzz, 1, 2, 2,
                        yzw, 1, 2, 3,
                        ywx, 1, 3, 0,
                        ywy, 1, 3, 1,
                        ywz, 1, 3, 2,
                        yww, 1, 3, 3,
                        zxx, 2, 0, 0,
                        zxy, 2, 0, 1,
                        zxz, 2, 0, 2,
                        zxw, 2, 0, 3,
                        zyx, 2, 1, 0,
                        zyy, 2, 1, 1,
                        zyz, 2, 1, 2,
                        zyw, 2, 1, 3,
                        zzx, 2, 2, 0,
                        zzy, 2, 2, 1,
                        zzz, 2, 2, 2,
                        zzw, 2, 2, 3,
                        zwx, 2, 3, 0,
                        zwy, 2, 3, 1,
                        zwz, 2, 3, 2,
                        zww, 2, 3, 3,
                        wxx, 3, 0, 0,
                        wxy, 3, 0, 1,
                        wxz, 3, 0, 2,
                        wxw, 3, 0, 3,
                        wyx, 3, 1, 0,
                        wyy, 3, 1, 1,
                        wyz, 3, 1, 2,
                        wyw, 3, 1, 3,
                        wzx, 3, 2, 0,
                        wzy, 3, 2, 1,
                        wzz, 3, 2, 2,
                        wzw, 3, 2, 3,
                        wwx, 3, 3, 0,
                        wwy, 3, 3, 1,
                        wwz, 3, 3, 2,
                        www, 3, 3, 3,
                    ]
                );

                impl_vec4_shuffle!(
                    $vec4, $vec4 : [
                        xxxx, 0, 0, 0, 0,
                        xxxy, 0, 0, 0, 1,
                        xxxz, 0, 0, 0, 2,
                        xxxw, 0, 0, 0, 3,
                        xxyx, 0, 0, 1, 0,
                        xxyy, 0, 0, 1, 1,
                        xxyz, 0, 0, 1, 2,
                        xxyw, 0, 0, 1, 3,
                        xxzx, 0, 0, 2, 0,
                        xxzy, 0, 0, 2, 1,
                        xxzz, 0, 0, 2, 2,
                        xxzw, 0, 0, 2, 3,
                        xxwx, 0, 0, 3, 0,
                        xxwy, 0, 0, 3, 1,
                        xxwz, 0, 0, 3, 2,
                        xxww, 0, 0, 3, 3,
                        xyxx, 0, 1, 0, 0,
                        xyxy, 0, 1, 0, 1,
                        xyxz, 0, 1, 0, 2,
                        xyxw, 0, 1, 0, 3,
                        xyyx, 0, 1, 1, 0,
                        xyyy, 0, 1, 1, 1,
                        xyyz, 0, 1, 1, 2,
                        xyyw, 0, 1, 1, 3,
                        xyzx, 0, 1, 2, 0,
                        xyzy, 0, 1, 2, 1,
                        xyzz, 0, 1, 2, 2,
                        xyzw, 0, 1, 2, 3,
                        xywx, 0, 1, 3, 0,
                        xywy, 0, 1, 3, 1,
                        xywz, 0, 1, 3, 2,
                        xyww, 0, 1, 3, 3,
                        xzxx, 0, 2, 0, 0,
                        xzxy, 0, 2, 0, 1,
                        xzxz, 0, 2, 0, 2,
                        xzxw, 0, 2, 0, 3,
                        xzyx, 0, 2, 1, 0,
                        xzyy, 0, 2, 1, 1,
                        xzyz, 0, 2, 1, 2,
                        xzyw, 0, 2, 1, 3,
                        xzzx, 0, 2, 2, 0,
                        xzzy, 0, 2, 2, 1,
                        xzzz, 0, 2, 2, 2,
                        xzzw, 0, 2, 2, 3,
                        xzwx, 0, 2, 3, 0,
                        xzwy, 0, 2, 3, 1,
                        xzwz, 0, 2, 3, 2,
                        xzww, 0, 2, 3, 3,
                        xwxx, 0, 3, 0, 0,
                        xwxy, 0, 3, 0, 1,
                        xwxz, 0, 3, 0, 2,
                        xwxw, 0, 3, 0, 3,
                        xwyx, 0, 3, 1, 0,
                        xwyy, 0, 3, 1, 1,
                        xwyz, 0, 3, 1, 2,
                        xwyw, 0, 3, 1, 3,
                        xwzx, 0, 3, 2, 0,
                        xwzy, 0, 3, 2, 1,
                        xwzz, 0, 3, 2, 2,
                        xwzw, 0, 3, 2, 3,
                        xwwx, 0, 3, 3, 0,
                        xwwy, 0, 3, 3, 1,
                        xwwz, 0, 3, 3, 2,
                        xwww, 0, 3, 3, 3,
                        yxxx, 1, 0, 0, 0,
                        yxxy, 1, 0, 0, 1,
                        yxxz, 1, 0, 0, 2,
                        yxxw, 1, 0, 0, 3,
                        yxyx, 1, 0, 1, 0,
                        yxyy, 1, 0, 1, 1,
                        yxyz, 1, 0, 1, 2,
                        yxyw, 1, 0, 1, 3,
                        yxzx, 1, 0, 2, 0,
                        yxzy, 1, 0, 2, 1,
                        yxzz, 1, 0, 2, 2,
                        yxzw, 1, 0, 2, 3,
                        yxwx, 1, 0, 3, 0,
                        yxwy, 1, 0, 3, 1,
                        yxwz, 1, 0, 3, 2,
                        yxww, 1, 0, 3, 3,
                        yyxx, 1, 1, 0, 0,
                        yyxy, 1, 1, 0, 1,
                        yyxz, 1, 1, 0, 2,
                        yyxw, 1, 1, 0, 3,
                        yyyx, 1, 1, 1, 0,
                        yyyy, 1, 1, 1, 1,
                        yyyz, 1, 1, 1, 2,
                        yyyw, 1, 1, 1, 3,
                        yyzx, 1, 1, 2, 0,
                        yyzy, 1, 1, 2, 1,
                        yyzz, 1, 1, 2, 2,
                        yyzw, 1, 1, 2, 3,
                        yywx, 1, 1, 3, 0,
                        yywy, 1, 1, 3, 1,
                        yywz, 1, 1, 3, 2,
                        yyww, 1, 1, 3, 3,
                        yzxx, 1, 2, 0, 0,
                        yzxy, 1, 2, 0, 1,
                        yzxz, 1, 2, 0, 2,
                        yzxw, 1, 2, 0, 3,
                        yzyx, 1, 2, 1, 0,
                        yzyy, 1, 2, 1, 1,
                        yzyz, 1, 2, 1, 2,
                        yzyw, 1, 2, 1, 3,
                        yzzx, 1, 2, 2, 0,
                        yzzy, 1, 2, 2, 1,
                        yzzz, 1, 2, 2, 2,
                        yzzw, 1, 2, 2, 3,
                        yzwx, 1, 2, 3, 0,
                        yzwy, 1, 2, 3, 1,
                        yzwz, 1, 2, 3, 2,
                        yzww, 1, 2, 3, 3,
                        ywxx, 1, 3, 0, 0,
                        ywxy, 1, 3, 0, 1,
                        ywxz, 1, 3, 0, 2,
                        ywxw, 1, 3, 0, 3,
                        ywyx, 1, 3, 1, 0,
                        ywyy, 1, 3, 1, 1,
                        ywyz, 1, 3, 1, 2,
                        ywyw, 1, 3, 1, 3,
                        ywzx, 1, 3, 2, 0,
                        ywzy, 1, 3, 2, 1,
                        ywzz, 1, 3, 2, 2,
                        ywzw, 1, 3, 2, 3,
                        ywwx, 1, 3, 3, 0,
                        ywwy, 1, 3, 3, 1,
                        ywwz, 1, 3, 3, 2,
                        ywww, 1, 3, 3, 3,
                        zxxx, 2, 0, 0, 0,
                        zxxy, 2, 0, 0, 1,
                        zxxz, 2, 0, 0, 2,
                        zxxw, 2, 0, 0, 3,
                        zxyx, 2, 0, 1, 0,
                        zxyy, 2, 0, 1, 1,
                        zxyz, 2, 0, 1, 2,
                        zxyw, 2, 0, 1, 3,
                        zxzx, 2, 0, 2, 0,
                        zxzy, 2, 0, 2, 1,
                        zxzz, 2, 0, 2, 2,
                        zxzw, 2, 0, 2, 3,
                        zxwx, 2, 0, 3, 0,
                        zxwy, 2, 0, 3, 1,
                        zxwz, 2, 0, 3, 2,
                        zxww, 2, 0, 3, 3,
                        zyxx, 2, 1, 0, 0,
                        zyxy, 2, 1, 0, 1,
                        zyxz, 2, 1, 0, 2,
                        zyxw, 2, 1, 0, 3,
                        zyyx, 2, 1, 1, 0,
                        zyyy, 2, 1, 1, 1,
                        zyyz, 2, 1, 1, 2,
                        zyyw, 2, 1, 1, 3,
                        zyzx, 2, 1, 2, 0,
                        zyzy, 2, 1, 2, 1,
                        zyzz, 2, 1, 2, 2,
                        zyzw, 2, 1, 2, 3,
                        zywx, 2, 1, 3, 0,
                        zywy, 2, 1, 3, 1,
                        zywz, 2, 1, 3, 2,
                        zyww, 2, 1, 3, 3,
                        zzxx, 2, 2, 0, 0,
                        zzxy, 2, 2, 0, 1,
                        zzxz, 2, 2, 0, 2,
                        zzxw, 2, 2, 0, 3,
                        zzyx, 2, 2, 1, 0,
                        zzyy, 2, 2, 1, 1,
                        zzyz, 2, 2, 1, 2,
                        zzyw, 2, 2, 1, 3,
                        zzzx, 2, 2, 2, 0,
                        zzzy, 2, 2, 2, 1,
                        zzzz, 2, 2, 2, 2,
                        zzzw, 2, 2, 2, 3,
                        zzwx, 2, 2, 3, 0,
                        zzwy, 2, 2, 3, 1,
                        zzwz, 2, 2, 3, 2,
                        zzww, 2, 2, 3, 3,
                        zwxx, 2, 3, 0, 0,
                        zwxy, 2, 3, 0, 1,
                        zwxz, 2, 3, 0, 2,
                        zwxw, 2, 3, 0, 3,
                        zwyx, 2, 3, 1, 0,
                        zwyy, 2, 3, 1, 1,
                        zwyz, 2, 3, 1, 2,
                        zwyw, 2, 3, 1, 3,
                        zwzx, 2, 3, 2, 0,
                        zwzy, 2, 3, 2, 1,
                        zwzz, 2, 3, 2, 2,
                        zwzw, 2, 3, 2, 3,
                        zwwx, 2, 3, 3, 0,
                        zwwy, 2, 3, 3, 1,
                        zwwz, 2, 3, 3, 2,
                        zwww, 2, 3, 3, 3,
                        wxxx, 3, 0, 0, 0,
                        wxxy, 3, 0, 0, 1,
                        wxxz, 3, 0, 0, 2,
                        wxxw, 3, 0, 0, 3,
                        wxyx, 3, 0, 1, 0,
                        wxyy, 3, 0, 1, 1,
                        wxyz, 3, 0, 1, 2,
                        wxyw, 3, 0, 1, 3,
                        wxzx, 3, 0, 2, 0,
                        wxzy, 3, 0, 2, 1,
                        wxzz, 3, 0, 2, 2,
                        wxzw, 3, 0, 2, 3,
                        wxwx, 3, 0, 3, 0,
                        wxwy, 3, 0, 3, 1,
                        wxwz, 3, 0, 3, 2,
                        wxww, 3, 0, 3, 3,
                        wyxx, 3, 1, 0, 0,
                        wyxy, 3, 1, 0, 1,
                        wyxz, 3, 1, 0, 2,
                        wyxw, 3, 1, 0, 3,
                        wyyx, 3, 1, 1, 0,
                        wyyy, 3, 1, 1, 1,
                        wyyz, 3, 1, 1, 2,
                        wyyw, 3, 1, 1, 3,
                        wyzx, 3, 1, 2, 0,
                        wyzy, 3, 1, 2, 1,
                        wyzz, 3, 1, 2, 2,
                        wyzw, 3, 1, 2, 3,
                        wywx, 3, 1, 3, 0,
                        wywy, 3, 1, 3, 1,
                        wywz, 3, 1, 3, 2,
                        wyww, 3, 1, 3, 3,
                        wzxx, 3, 2, 0, 0,
                        wzxy, 3, 2, 0, 1,
                        wzxz, 3, 2, 0, 2,
                        wzxw, 3, 2, 0, 3,
                        wzyx, 3, 2, 1, 0,
                        wzyy, 3, 2, 1, 1,
                        wzyz, 3, 2, 1, 2,
                        wzyw, 3, 2, 1, 3,
                        wzzx, 3, 2, 2, 0,
                        wzzy, 3, 2, 2, 1,
                        wzzz, 3, 2, 2, 2,
                        wzzw, 3, 2, 2, 3,
                        wzwx, 3, 2, 3, 0,
                        wzwy, 3, 2, 3, 1,
                        wzwz, 3, 2, 3, 2,
                        wzww, 3, 2, 3, 3,
                        wwxx, 3, 3, 0, 0,
                        wwxy, 3, 3, 0, 1,
                        wwxz, 3, 3, 0, 2,
                        wwxw, 3, 3, 0, 3,
                        wwyx, 3, 3, 1, 0,
                        wwyy, 3, 3, 1, 1,
                        wwyz, 3, 3, 1, 2,
                        wwyw, 3, 3, 1, 3,
                        wwzx, 3, 3, 2, 0,
                        wwzy, 3, 3, 2, 1,
                        wwzz, 3, 3, 2, 2,
                        wwzw, 3, 3, 2, 3,
                        wwwx, 3, 3, 3, 0,
                        wwwy, 3, 3, 3, 1,
                        wwwz, 3, 3, 3, 2,
                        wwww, 3, 3, 3, 3,
                    ]
                );
            }
        )*
    };
}

impl_vector_shuffle!(
    Float, Vec2, Vec3, Vec4,
    Int, IVec2, IVec3, IVec4,
    UInt, UVec2, UVec3, UVec4,
    Double, DVec2, DVec3, DVec4,
);

// macro_rules! impl_vector_shuffle {
//     ($(($name:ident, $unit:ident, $vec2:ident, $vec3:ident, $vec4:ident): [ $($f:ident, $dst:ident, [$($a:literal,)*],)* ],)*) => {
//         $(
//             impl $name {
//                 $(
//                     pub fn $f(&self) -> VectorShuffle<$dst> {
//                         VectorShuffle {
//                             src: self.id,
//                             src_ty: PrimitiveType::$name,
//                             components: [$($a,)*],
//                             _marker: PhantomData,
//                         }
//                     }
//                 )*
//             }
//         )*
//     };
// }

// impl_vector_shuffle!(
//     (Vec2, Float, Vec2, Vec3, Vec4) : [
//         x, Float, [0, 0, 0, 0,],
//         y, Float, [1, 0, 0, 0,],
//         xx, Vec2, [0, 0, 0, 0,],
//         xy, Vec2, [0, 1, 0, 0,],
//         yx, Vec2, [1, 0, 0, 0,],
//         yy, Vec2, [1, 1, 0, 0,],
//     ],
//     (Vec3, Float, Vec2, Vec3, Vec4) : [
//         x, Float, [0, 0, 0, 0,],
//         y, Float, [1, 0, 0, 0,],
//         z, Float, [2, 0, 0, 0,],
//         xx, Vec2, [0, 0, 0, 0,],
//         xy, Vec2, [0, 1, 0, 0,],
//         yx, Vec2, [1, 0, 0, 0,],
//         yy, Vec2, [1, 1, 0, 0,],
//         xz, Vec2, [0, 2, 0, 0,],
//         zx, Vec2, [2, 0, 0, 0,],
//         zz, Vec2, [2, 2, 0, 0,],
//         yz, Vec2, [1, 2, 0, 0,],
//         zy, Vec2, [2, 1, 0, 0,],
//         xxx, Vec3, [0, 0, 0, 0,],
//         xxy, Vec3, [0, 0, 1, 0,],
//         xxz, Vec3, [0, 0, 2, 0,],
//         xyx, Vec3, [0, 1, 0, 0,],
//         xyy, Vec3, [0, 1, 1, 0,],
//         xyz, Vec3, [0, 1, 2, 0,],
//         xzx, Vec3, [0, 2, 0, 0,],
//         xzy, Vec3, [0, 2, 1, 0,],
//         xzz, Vec3, [0, 2, 2, 0,],
//         yxx, Vec3, [1, 0, 0, 0,],
//         yxy, Vec3, [1, 0, 1, 0,],
//         yxz, Vec3, [1, 0, 2, 0,],
//         yyx, Vec3, [1, 1, 0, 0,],
//         yyy, Vec3, [1, 1, 1, 0,],
//         yyz, Vec3, [1, 1, 2, 0,],
//         yzx, Vec3, [1, 2, 0, 0,],
//         yzy, Vec3, [1, 2, 1, 0,],
//         yzz, Vec3, [1, 2, 2, 0,],
//         zxx, Vec3, [2, 0, 0, 0,],
//         zxy, Vec3, [2, 0, 1, 0,],
//         zxz, Vec3, [2, 0, 2, 0,],
//         zyx, Vec3, [2, 1, 0, 0,],
//         zyy, Vec3, [2, 1, 1, 0,],
//         zyz, Vec3, [2, 1, 2, 0,],
//         zzx, Vec3, [2, 2, 0, 0,],
//         zzy, Vec3, [2, 2, 1, 0,],
//         zzz, Vec3, [2, 2, 2, 0,],
//     ],
// );

// macro_rules! gen_op {
//     ($op:ident, $f:ident, $name:ident, $rust:ident) => {
//         // name name
//         impl std::ops::$op<$name> for $name {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // name &name
//         impl std::ops::$op<$name> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name name
//         impl std::ops::$op<&'_ $name> for $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name &name
//         impl std::ops::$op<&'_ $name> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // name rust
//         impl std::ops::$op<$rust> for $name {
//             type Output = $name;

//             fn $f(self, rhs: $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }
        
//         // name &rust
//         impl std::ops::$op<&'_ $rust> for $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(*rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name rust
//         impl std::ops::$op<$rust> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name &rust
//         impl std::ops::$op<&'_ $rust> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(*rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // rust name
//         impl std::ops::$op<$name> for $rust {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // rust &name
//         impl std::ops::$op<&'_ $name> for $rust {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &rust name
//         impl std::ops::$op<$name> for &'_ $rust {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(*self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &rust &name
//         impl std::ops::$op<&'_ $name> for &'_ $rust {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::PrimitiveVal::$name(*self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }
//     };
// }

// macro_rules! gen_bool_ops {
//     ($($name:ident, $rust:ident,)*) => {
//         $(
//             gen_op!(BitAnd, bitand, $name, $rust);
//             gen_op!(BitOr, bitor, $name, $rust);
//             gen_op!(BitXor, bitxor, $name, $rust);
//         )*
//     };
// }

// gen_bool_ops!(
//     Bool, bool,
//     BVec2, GlamBVec2,
//     BVec3, GlamBVec3,
//     BVec4, GlamBVec4,
// );

// macro_rules! gen_ops {
//     ($($name:ident, $rust:ident,)*) => {
//         $(
//             gen_op!(Add, add, $name, $rust);
//             gen_op!(Sub, sub, $name, $rust);
//             gen_op!(Mul, mul, $name, $rust);
//             gen_op!(Div, div, $name, $rust);
//         )*
//     };
// }

// gen_ops!(
//     Int, i32,
//     UInt, u32,
//     Float, f32,
//     Double, f64,
//     IVec2, GlamIVec2,
//     IVec3, GlamIVec3,
//     IVec4, GlamIVec4,
//     UVec2, GlamUVec2,
//     UVec3, GlamUVec3,
//     UVec4, GlamUVec4,
//     Vec2, GlamVec2,
//     Vec3, GlamVec3,
//     Vec4, GlamVec4,
//     DVec2, GlamDVec2,
//     DVec3, GlamDVec3,
//     DVec4, GlamDVec4,
//     Mat2, GlamMat2,
//     Mat3, GlamMat3,
//     Mat4, GlamMat4,
//     DMat2, GlamDMat2,
//     DMat3, GlamDMat3,
//     DMat4, GlamDMat4,
// );


// macro_rules! gen_vec_mat_ops {
//     ($($mat:ident, $rust_mat:ident, $vec:ident, $rust_vec:ident,)*) => {
//         $(
//             // mat vec
//             impl std::ops::Mul<$vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // mat &vec
//             impl std::ops::Mul<&'_ $vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat vec
//             impl std::ops::Mul<$vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat &vec
//             impl std::ops::Mul<&'_ $vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }



//             // mat rust_vec
//             impl std::ops::Mul<$rust_vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$vec(rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // mat &rust_vec
//             impl std::ops::Mul<&'_ $rust_vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$vec(*rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat rust_vec
//             impl std::ops::Mul<$rust_vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$vec(rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat &rust_vec
//             impl std::ops::Mul<&'_ $rust_vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$vec(*rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }



//             // rust_mat vec
//             impl std::ops::Mul<$vec> for $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$mat(self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // rust_mat &vec
//             impl std::ops::Mul<&'_ $vec> for $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$mat(self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &rust_mat vec
//             impl std::ops::Mul<$vec> for &'_ $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$mat(*self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &rust_mat &vec
//             impl std::ops::Mul<&'_ $vec> for &'_ $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::PrimitiveVal::$mat(*self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::PrimitiveType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }
//         )*
        
//     };
// }

// gen_vec_mat_ops!(
//     Mat2, GlamMat2, Vec2, GlamVec2,
//     Mat3, GlamMat3, Vec3, GlamVec3,
//     Mat4, GlamMat4, Vec4, GlamVec4,
//     DMat2, GlamDMat2, DVec2, GlamDVec2,
//     DMat3, GlamDMat3, DVec3, GlamDVec3,
//     DMat4, GlamDMat4, DVec4, GlamDVec4,
// );