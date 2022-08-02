use crate::builder::Handle;
use crate::builder::RawBuilder;

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

#[cfg(feature = "glam")]
use glam::{
    DMat2 as GlamDMat2, DMat3 as GlamDMat3, DMat4 as GlamDMat4, DVec2 as GlamDVec2,
    DVec3 as GlamDVec3, DVec4 as GlamDVec4, IVec2 as GlamIVec2, IVec3 as GlamIVec3,
    IVec4 as GlamIVec4, Mat2 as GlamMat2, Mat3 as GlamMat3, Mat4 as GlamMat4, UVec2 as GlamUVec2,
    UVec3 as GlamUVec3, UVec4 as GlamUVec4, Vec2 as GlamVec2, Vec3 as GlamVec3, Vec4 as GlamVec4,
};

use super::PrimitiveType;

macro_rules! gen_types {
    ($($name:ident,)*) => {
        $(
            #[derive(Copy, Clone, Debug)]
            pub struct $name {
                pub(crate) id: usize,
            }
        )*
    };
}

gen_types!(
    Bool, Int, UInt, Float, Double, IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
    DVec2, DVec3, DVec4, Mat2, Mat3, Mat4, DMat2, DMat3, DMat4,
);

pub trait SpvRustEq<T: AsPrimitive>: AsPrimitive {}

pub trait AsPrimitiveType {
    const TY: crate::data::PrimitiveType;
}

pub trait AsPrimitive {
    fn id(&self, b: &dyn RawBuilder) -> usize;

    fn ty(&self) -> crate::data::PrimitiveType;
}

pub trait IsPrimitiveType: AsPrimitiveType {}

pub trait AsDataType {
    const TY: crate::data::DataType;
}

pub trait AsData {
    fn id(&self, b: &dyn RawBuilder) -> usize;

    fn ty(&self) -> crate::data::DataType;
}

pub trait IsDataType: AsDataType {}

pub trait FromId {
    fn from_id(id: usize) -> Self;
}

pub trait SpvStore<Rhs: AsPrimitiveType>: AsPrimitiveType + AsPrimitive {
    fn val(rhs: Rhs) -> crate::data::PrimitiveVal;
}

macro_rules! gen_relation {
    ($name:ident, $rust:ident, $conv:ident) => {
        impl SpvRustEq<$rust> for $name {}

        impl SpvRustEq<$name> for $rust {}

        impl SpvStore<$rust> for $name {
            fn val(rhs: $rust) -> crate::data::PrimitiveVal {
                crate::data::PrimitiveVal::$name((rhs).$conv())
            }
        }

        impl AsPrimitiveType for $rust {
            const TY: crate::data::PrimitiveType = crate::data::PrimitiveType::$name;
        }

        impl AsPrimitive for $rust {
            fn id(&self, b: &dyn RawBuilder) -> usize {
                let id = b.get_new_id();
                b.push_instruction(crate::builder::Instruction::Store {
                    val: crate::data::PrimitiveVal::$name((*self).$conv()),
                    store: id,
                });
                id
            }

            fn ty(&self) -> crate::data::PrimitiveType {
                <Self as AsPrimitiveType>::TY
            }
        }

        impl AsPrimitive for &'_ $rust {
            fn id(&self, b: &dyn RawBuilder) -> usize {
                let id = b.get_new_id();
                b.push_instruction(crate::builder::Instruction::Store {
                    val: crate::data::PrimitiveVal::$name((**self).$conv()),
                    store: id,
                });
                id
            }

            fn ty(&self) -> crate::data::PrimitiveType {
                <$rust as AsPrimitiveType>::TY
            }
        }

        impl AsDataType for $rust {
            const TY: crate::data::DataType =
                crate::data::DataType::Primitive(crate::data::PrimitiveType::$name);
        }

        impl AsData for $rust {
            fn id(&self, b: &dyn RawBuilder) -> usize {
                let id = b.get_new_id();
                b.push_instruction(crate::builder::Instruction::Store {
                    val: crate::data::PrimitiveVal::$name((*self).$conv()),
                    store: id,
                });
                id
            }

            fn ty(&self) -> crate::data::DataType {
                <Self as AsDataType>::TY
            }
        }
    };
}

macro_rules! gen_as_data_single {
    ($name:ident) => {
        impl SpvRustEq<$name> for $name {}

        impl AsPrimitiveType for $name {
            const TY: crate::data::PrimitiveType = crate::data::PrimitiveType::$name;
        }

        impl IsPrimitiveType for $name {}

        impl AsPrimitive for $name {
            fn id(&self, _: &dyn RawBuilder) -> usize {
                self.id
            }

            fn ty(&self) -> crate::data::PrimitiveType {
                <Self as AsPrimitiveType>::TY
            }
        }

        impl AsPrimitive for &'_ $name {
            fn id(&self, _: &dyn RawBuilder) -> usize {
                self.id
            }

            fn ty(&self) -> crate::data::PrimitiveType {
                <$name as AsPrimitiveType>::TY
            }
        }

        impl AsDataType for $name {
            const TY: crate::data::DataType =
                crate::data::DataType::Primitive(crate::data::PrimitiveType::$name);
        }

        impl IsDataType for $name {}

        impl AsData for $name {
            fn id(&self, _: &dyn RawBuilder) -> usize {
                self.id
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
    };
}

macro_rules! gen_as_data_primitive {
    ($($name1:ident, $rust1:ident,)*) => {
        $(
            gen_as_data_single!($name1);
            gen_relation!($name1, $rust1, into);
        )*
    };
}

macro_rules! gen_as_data_vec {
    ($($name2:ident, $rust2:ident, $glm:ident,)*) => {
        $(
            gen_as_data_single!($name2);
            gen_relation!($name2, $rust2, into);
            gen_relation!($name2, $glm, into);
        )*
    };
}

pub trait Matrix: AsPrimitive + AsPrimitiveType {
    type Vector: FromId;
}

macro_rules! impl_mat_vec {
    ($($mat:ident, $vec:ident)*) => {
        $(
            impl Matrix for $mat {
                type Vector = $vec;
            }
        )*
    };
}

macro_rules! gen_as_data_mat {
    ($($name:ident, $rust:ident, $glm:ident, $vec:ident,)*) => {
        $(
            gen_as_data_single!($name);
            gen_relation!($name, $rust, into);
            gen_relation!($name, $glm, to_cols_array_2d);
            impl_mat_vec!($name, $vec);
        )*
    };
}

#[rustfmt::skip]
gen_as_data_primitive!(
    Bool, bool,
    Int, i32,
    UInt, u32,
    Float, f32,
    Double, f64,
);

#[rustfmt::skip]
gen_as_data_vec!(
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
);

#[rustfmt::skip]
gen_as_data_mat!(
    Mat2, RustMat2, GlamMat2, Vec2,
    Mat3, RustMat3, GlamMat3, Vec3,
    Mat4, RustMat4, GlamMat4, Vec4,
    DMat2, RustDMat2, GlamDMat2, DVec2,
    DMat3, RustDMat3, GlamDMat3, DVec3,
    DMat4, RustDMat4, GlamDMat4, DVec4,
);

macro_rules! impl_unit_shuffle {
    ($base:ident, $unit:ident : [ $($f:ident, $component:literal,)* ] ) => {
        $(
            pub fn $f(&self, b: impl Handle) -> $unit {
                let new_id = b.builder().get_new_id();
                b.builder().push_instruction(crate::builder::Instruction::VectorShuffle {
                    src: (self.id, PrimitiveType::$base),
                    dst: (new_id, <$unit as AsPrimitiveType>::TY),
                    components: [$component, 0, 0, 0,]
                });
                $unit::from_id(new_id)
            }
        )*
    };
}

macro_rules! impl_vec2_shuffle {
    ($base:ident, $vec2:ident : [ $($f:ident, $a:literal, $b:literal,)* ]) => {
        $(
            pub fn $f(&self, b: impl Handle) -> $vec2 {
                let new_id = b.builder().get_new_id();
                b.builder().push_instruction(crate::builder::Instruction::VectorShuffle {
                    src: (self.id, PrimitiveType::$base),
                    dst: (new_id, <$vec2 as AsPrimitiveType>::TY),
                    components: [$a, $b, 0, 0,]
                });
                $vec2::from_id(new_id)
            }
        )*
    };
}

macro_rules! impl_vec3_shuffle {
    ($base:ident, $vec3:ident : [ $($f:ident, $a:literal, $b:literal, $c:literal,)* ]) => {
        $(
            pub fn $f(&self, b: impl Handle) -> $vec3 {
                let new_id = b.builder().get_new_id();
                b.builder().push_instruction(crate::builder::Instruction::VectorShuffle {
                    src: (self.id, PrimitiveType::$base),
                    dst: (new_id, <$vec3 as AsPrimitiveType>::TY),
                    components: [$a, $b, $c, 0,]
                });
                $vec3::from_id(new_id)
            }
        )*
    };
}

macro_rules! impl_vec4_shuffle {
    ($base:ident, $vec4:ident : [ $($f:ident, $a:literal, $b:literal, $c:literal, $d:literal,)* ]) => {
        $(
            pub fn $f(&self, b: impl Handle) -> $vec4 {
                let new_id = b.builder().get_new_id();
                b.builder().push_instruction(crate::builder::Instruction::VectorShuffle {
                    src: (self.id, PrimitiveType::$base),
                    dst: (new_id, <$vec4 as AsPrimitiveType>::TY),
                    components: [$a, $b, $c, $d,]
                });
                $vec4::from_id(new_id)
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
                    $vec3, $vec3 : [
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
                    $vec4, $vec3 : [
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

#[rustfmt::skip]
impl_vector_shuffle!(
    Float, Vec2, Vec3, Vec4,
    Int, IVec2, IVec3, IVec4,
    UInt, UVec2, UVec3, UVec4,
    Double, DVec2, DVec3, DVec4,
);

macro_rules! impl_math_glsl {
    ($($name:ident,)*) => {
        $(
            impl $name {
                pub fn exp(&self, b: impl Handle) -> Self {
                    let res_id = b.builder().get_new_id();
                    b.builder().push_instruction(crate::builder::Instruction::GlslOp {
                        lhs: self.id,
                        lhs_ty: AsPrimitive::ty(self),
                        rhs: None,
                        rhs_ty: None,
                        res: res_id,
                        res_ty: <Self as AsPrimitiveType>::TY,
                        op: rspirv::spirv::GLOp::Exp
                    });
                    Self::from_id(res_id)
                }
            }
        )*
    };
}

impl_math_glsl!(Float, Vec2, Vec3, Vec4, Double, DVec2, DVec3, DVec4,);

macro_rules! impl_vec_glsl {
    ($($name:ident, $component:ident,)*) => {
        $(
            impl $name {
                pub fn normalize(&self, b: impl Handle) -> Self {
                    let res_id = b.builder().get_new_id();
                    b.builder().push_instruction(crate::builder::Instruction::GlslOp {
                        lhs: self.id,
                        lhs_ty: AsPrimitive::ty(self),
                        op: rspirv::spirv::GLOp::Normalize,
                        rhs: None,
                        rhs_ty: None,
                        res: res_id,
                        res_ty: AsPrimitive::ty(self),
                    });
                    Self::from_id(res_id)
                }

                pub fn dot(&self, rhs: &dyn SpvRustEq<Self>, b: impl Handle) -> $component {
                    let res_id = b.builder().get_new_id();

                    b.builder().push_instruction(crate::builder::Instruction::Dot {
                        lhs: self.id,
                        ty: AsPrimitive::ty(self),
                        rhs: rhs.id(&*b.builder()),
                        res: res_id,
                        comp_ty: <$component as AsPrimitiveType>::TY,
                    });

                    $component::from_id(res_id)
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_vec_glsl!(
    Vec2, Float,
    Vec3, Float,
    Vec4, Float,
    DVec2, Double,
    DVec3, Double,
    DVec4, Double,
);

impl Vec3 {
    pub fn cross(&self, rhs: &dyn SpvRustEq<Self>, b: impl Handle) -> Vec3 {
        let res_id = b.builder().get_new_id();

        b.builder()
            .push_instruction(crate::builder::Instruction::GlslOp {
                lhs: self.id,
                lhs_ty: AsPrimitive::ty(self),
                op: rspirv::spirv::GLOp::Cross,
                rhs: Some(rhs.id(&*b.builder())),
                rhs_ty: Some(AsPrimitive::ty(self)),
                res: res_id,
                res_ty: PrimitiveType::Vec3,
            });

        Vec3::from_id(res_id)
    }
}

impl DVec3 {
    pub fn cross(&self, rhs: &dyn SpvRustEq<Self>, b: impl Handle) -> DVec3 {
        let res_id = b.builder().get_new_id();

        b.builder()
            .push_instruction(crate::builder::Instruction::GlslOp {
                lhs: self.id,
                lhs_ty: AsPrimitive::ty(self),
                op: rspirv::spirv::GLOp::Cross,
                rhs: Some(rhs.id(&*b.builder())),
                rhs_ty: Some(AsPrimitive::ty(self)),
                res: res_id,
                res_ty: PrimitiveType::DVec3,
            });

        DVec3::from_id(res_id)
    }
}
