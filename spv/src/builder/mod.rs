use std::any::Any;
use std::any::TypeId;
use std::marker::PhantomData;
use std::rc::Rc;

use either::*;

// If I knew how to write macros properly this wouldn't be here but this is easier than learning proper macros
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

pub mod base_builder;
pub mod condition_builder;
pub mod fn_builder;
pub mod instruction;
pub mod loop_builder;
pub mod main_builder;
pub mod var;

pub(crate) use base_builder::*;
pub use condition_builder::*;
pub use fn_builder::*;
pub use instruction::*;
pub use loop_builder::*;
pub use main_builder::*;
pub(crate) use var::*;

use crate::data::*;
use crate::interface::*;
use crate::sampler::*;
use crate::texture::*;

pub trait AsAny {
    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub trait RawBuilder: AsAny {
    fn push_instruction(&self, instruction: Instruction);

    fn name_var(&self, id: usize, name: String);

    fn get_new_id(&self) -> usize;

    fn in_loop(&self) -> bool;

    fn push_constant(&self) -> Option<(DataType, u32, Option<&'static str>)>;
}

impl dyn RawBuilder {
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref()
    }

    #[allow(dead_code)]
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }

    #[allow(dead_code)]
    fn downcast<T: Any>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        use std::ops::Deref;

        match self.deref().as_any_ref().type_id() == ::std::any::TypeId::of::<T>() {
            true => Ok(self.as_any_box().downcast().unwrap()),
            false => Err(self),
        }
    }
}

macro_rules! gen_vec2_construct {
    ($($f_name:ident, $vec_ty:ident, $comp_ty:ident,)*) => {
        $(
            pub fn $f_name(&self, x: &dyn SpvRustEq<$comp_ty>, y: &dyn SpvRustEq<$comp_ty>) -> $vec_ty {
                let id = self.raw.get_new_id();
                self.raw.push_instruction(Instruction::VectorComposite {
                    components: [x.id(&*self.raw), y.id(&*self.raw), 0, 0],
                    ty: crate::data::PrimitiveType::$vec_ty,
                    store: id,
                });
                $vec_ty {
                    id,
                }
            }
        )*
    };
}

macro_rules! gen_vec3_construct {
    ($($f_name:ident, $vec_ty:ident, $comp_ty:ident,)*) => {
        $(
            pub fn $f_name(&self, x: &dyn SpvRustEq<$comp_ty>, y: &dyn SpvRustEq<$comp_ty>, z: &dyn SpvRustEq<$comp_ty>) -> $vec_ty {
                let id = self.raw.get_new_id();
                self.raw.push_instruction(Instruction::VectorComposite {
                    components: [x.id(&*self.raw), y.id(&*self.raw), z.id(&*self.raw), 0],
                    ty: crate::data::PrimitiveType::$vec_ty,
                    store: id,
                });
                $vec_ty {
                    id,
                }
            }
        )*
    };
}

macro_rules! gen_vec4_construct {
    ($($f_name:ident, $vec_ty:ident, $comp_ty:ident,)*) => {
        $(
            pub fn $f_name(&self, x: &dyn SpvRustEq<$comp_ty>, y: &dyn SpvRustEq<$comp_ty>, z: &dyn SpvRustEq<$comp_ty>, w: &dyn SpvRustEq<$comp_ty>) -> $vec_ty {
                let id = self.raw.get_new_id();
                self.raw.push_instruction(Instruction::VectorComposite {
                    components: [x.id(&*self.raw), y.id(&*self.raw), z.id(&*self.raw), w.id(&*self.raw)],
                    ty: crate::data::PrimitiveType::$vec_ty,
                    store: id,
                });
                $vec_ty {
                    id,
                }
            }
        )*
    };
}

macro_rules! gen_vec_construct {
    ($($name:ident,)*) => {
        $(
            impl $name {
                gen_vec2_construct!(
                    vec2, Vec2, Float,
                    ivec2, IVec2, Int,
                    uvec2, UVec2, UInt,
                    dvec2, DVec2, Double,
                );

                gen_vec3_construct!(
                    vec3, Vec3, Float,
                    ivec3, IVec3, Int,
                    uvec3, UVec3, UInt,
                    dvec3, DVec3, Double,
                );

                gen_vec4_construct!(
                    vec4, Vec4, Float,
                    ivec4, IVec4, Int,
                    uvec4, UVec4, UInt,
                    dvec4, DVec4, Double,
                );
            }
        )*
    };
}

gen_vec_construct!(FnBuilder, ConditionBuilder, LoopBuilder, MainBuilder,);

macro_rules! gen_const_type {
    ($($f:ident, $t:ident, $rust:ident,)*) => {
        $(
            pub fn $f(&self, v: $rust) -> crate::data::$t {
                let id = self.raw.get_new_id();
                self.raw.push_instruction(Instruction::Store {
                    val: crate::data::PrimitiveVal::$t(v),
                    store: id,
                });
                $t {
                    id,
                }
            }
        )*
    };
}

macro_rules! gen_const_types {
    ($($name:ident,)*) => {
        $(
            impl $name {
                gen_const_type!(
                    const_bool, Bool, bool,
                    const_float, Float, f32,
                    const_int, Int, i32,
                    const_uint, UInt, u32,
                    const_double, Double, f64,
                    const_ivec2, IVec2, RustIVec2,
                    const_ivec3, IVec3, RustIVec3,
                    const_ivec4, IVec4, RustIVec4,
                    const_uvec2, UVec2, RustUVec2,
                    const_uvec3, UVec3, RustUVec3,
                    const_uvec4, UVec4, RustUVec4,
                    const_vec2, Vec2, RustVec2,
                    const_vec3, Vec3, RustVec3,
                    const_vec4, Vec4, RustVec4,
                    const_dvec2, DVec2, RustDVec2,
                    const_dvec3, DVec3, RustDVec3,
                    const_dvec4, DVec4, RustDVec4,
                    const_mat2, Mat2, RustMat2,
                    const_mat3, Mat3, RustMat3,
                    const_mat4, Mat4, RustMat4,
                    const_dmat2, DMat2, RustDMat2,
                    const_dmat3, DMat3, RustDMat3,
                    const_dmat4, DMat4, RustDMat4,
                );
            }
        )*
    };
}

gen_const_types!(FnBuilder, ConditionBuilder, LoopBuilder, MainBuilder,);

macro_rules! gen_intrinsics {
    ($($name:ident,)*) => {
        $(
            impl $name {

                /// Adds an if condition to the current function
                pub fn spv_if<F: FnOnce(&ConditionBuilder)>(&self, b: impl crate::data::SpvRustEq<Bool>, f: F) -> ConditionBuilder {
                    let b = ConditionBuilder {
                        raw: RawConditionBuilder::new(Rc::clone(&self.raw), b.id(&*self.raw)),
                    };

                    f(&b);

                    b
                }

                /// Adds a while loop to the current function
                ///
                /// Note that the boolean condition must be updated by calling spv_store or it will always
                /// store the same condition and if it is initially true the loop will never terminate
                pub fn spv_while<F: FnOnce(&LoopBuilder)>(&self, b: impl crate::data::SpvRustEq<Bool>, f: F) {
                    let b = LoopBuilder {
                        raw: RawLoopBuilder::new(Rc::clone(&self.raw), b.id(&*self.raw)),
                    };

                    f(&b);

                    drop(b);
                }

                /// Adds a break instruction to the current function
                ///
                /// panics if the builder called on doesn't descend from a loop builder
                pub fn spv_break(&self) {
                    if !self.raw.in_loop() {
                        panic!("Cannot call spv_break not in loop");
                    }
                    self.raw.push_instruction(Instruction::Break);
                }

                /// Adds a continue instruction to the current function
                ///
                /// panics if the builder called on doesn't descend from a loop builder
                pub fn spv_continue(&self) {
                    if !self.raw.in_loop() {
                        panic!("Cannot call spv_continue in loop")
                    }
                    self.raw.push_instruction(Instruction::Continue);
                }

                pub fn spv_return(&self, val: impl crate::data::AsPrimitive) {
                    self.raw.push_instruction(Instruction::Return {
                        id: val.id(&*self.raw)
                    })
                }

                pub fn call<R: AsPrimitiveType + FromId>(&self, f: crate::function::Function<R>, args: &[&dyn AsData]) -> R {
                    let store_id = self.raw.get_new_id();

                    self.raw.push_instruction(Instruction::FnCall {
                        fn_id: f.id,
                        store_id,
                        arguments: args.iter().map(|a| (a.id(&*self.raw), a.ty())).collect(),
                    });

                    R::from_id(store_id)
                }

                pub fn load_in<T: crate::data::IsPrimitiveType + FromId>(&self, input: SpvInput<T>) -> T {
                    let store = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LoadIn {
                        index: input.index,
                        ty: T::TY,
                        store,
                    });
                    T::from_id(store)
                }

                pub fn store_out<T, S>(&self, output: SpvOutput<T>, store: S)
                where
                    T: crate::IsPrimitiveType + crate::data::SpvRustEq<S>,
                    S: AsPrimitive,
                {
                    self.raw.push_instruction(Instruction::StoreOut {
                        index: output.index,
                        ty: T::TY,
                        read: crate::data::AsPrimitive::id(&store, &*self.raw),
                    })
                }

                /// Load the uniform into a new variable
                pub fn load_uniform<T: IsDataType + FromId>(&self, uniform: SpvUniform<T>) -> T {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LoadUniform {
                        index: uniform.index,
                        store: new_id,
                        ty: T::TY,
                    });
                    T::from_id(new_id)
                }

                /// Load one field from the uniform containing a struct
                ///
                /// Will panic if the struct has no field by the name supplied
                pub fn load_uniform_field<S: AsSpvStruct, T: FromId>(&self, uniform: SpvUniform<SpvStruct<S>>, field: &str) -> T {
                    let f_index = S::DESC.names.iter().position(|&f| f == field).unwrap();
                    let f_ty = *S::DESC.fields.get(f_index).unwrap();
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LoadUniformField {
                        u_index: uniform.index,
                        f_index,
                        store: new_id,
                        f_ty,
                        ty: SpvStruct::<S>::TY,
                    });

                    T::from_id(new_id)
                }

                /// Load the push constant into a new variable
                /// 
                /// will panic if the builder has no push constant 
                /// or if the push constant is a primitive or array
                pub fn load_push_constant<T: IsDataType + FromId>(&self) -> T {
                    if self.raw.push_constant().is_some() {
                        let new_id = self.raw.get_new_id();
                        self.raw.push_instruction(Instruction::LoadPushConstant {
                            store: new_id,
                        });
                        T::from_id(new_id)
                    } else {
                        panic!("ERROR: Cannot load_push_constant when no push constant on builder")
                    }
                    
                }

                /// Load one field from the push_constant containing a struct
                ///
                /// Will panic if the struct has no field by the name supplied or if the builder has
                /// no push constant or if the push constant is a primitive or array
                pub fn load_push_constant_field<T: FromId>(&self, field: &str) -> T {
                    if let Some((ty, _, _)) = self.raw.push_constant() {
                        if let DataType::Struct(_, _, names, fields) = ty {
                            let f_index = names.iter().position(|&f| f == field).unwrap();
                            let f_ty = *fields.get(f_index).unwrap();
                            let new_id = self.raw.get_new_id();
                            self.raw.push_instruction(Instruction::LoadPushConstantField {
                                f_index,
                                store: new_id,
                                f_ty,
                            });
        
                            T::from_id(new_id)
                        } else {
                            panic!("ERROR: Cannot load_push_constant_field when push constant isn't a struct");
                        }
                    } else {
                        panic!("ERROR: Cannot load_push_constant field when no push constant on builder");
                    }
                }

                /// Load one field from the uniform containing a struct by the index of the field
                ///
                /// Will panic if the index is out of bounds of the number of structs fields
                pub fn load_uniform_field_by_index<S: AsSpvStruct, T: FromId>(&self, uniform: SpvUniform<SpvStruct<S>>, field_index: usize) -> T {
                    let new_id = self.raw.get_new_id();

                    let f_ty = *S::DESC.fields.get(field_index).unwrap();

                    self.raw.push_instruction(Instruction::LoadUniformField {
                        u_index: uniform.index,
                        f_index: field_index,
                        store: new_id,
                        f_ty,
                        ty: SpvStruct::<S>::TY,
                    });

                    T::from_id(new_id)
                }

                pub fn vector_shuffle<V: AsPrimitiveType + FromId>(&self, s: VectorShuffle<V>) -> V {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::VectorShuffle {
                        src: (s.src, s.src_ty),
                        dst: (new_id, V::TY),
                        components: s.components
                    });
                    V::from_id(new_id)
                }

                pub fn store<Rhs: AsPrimitiveType, T: SpvStore<Rhs>>(&self, lhs: T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::Store {
                        store: lhs.id(&*self.raw),
                        val: T::val(rhs),
                    })
                }

                pub fn add<Rhs: AsPrimitiveType + AsPrimitive, T: SpvAdd<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::Add {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn sub<Rhs: AsPrimitiveType + AsPrimitive, T: SpvSub<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::Sub {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn div<Rhs: AsPrimitiveType + AsPrimitive, T: SpvDiv<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::Div {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn mul<Rhs: AsPrimitiveType + AsPrimitive, T: SpvMul<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::Mul {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn add_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvAddAssign<Rhs>>(&self, lhs: &mut T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::AddAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    })
                }

                pub fn sub_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvSubAssign<Rhs>>(&self, lhs: &mut T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::SubAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    })
                }

                pub fn mul_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvMulAssign<Rhs>>(&self, lhs: &mut T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::MulAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    })
                }

                pub fn div_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvDivAssign<Rhs>>(&self, lhs: &mut T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::DivAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    })
                }

                pub fn bit_and<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitAnd<Rhs>>(&self, lhs: T, rhs: T) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::BitAnd {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn bit_or<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitOr<Rhs>>(&self, lhs: T, rhs: T) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::BitOr {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn bit_xor<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitXor<Rhs>>(&self, lhs: T, rhs: T) -> T::Output {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::BitXor {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn bit_and_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitAnd<Rhs>>(&self, lhs: &mut T, rhs: T) {
                    self.raw.push_instruction(Instruction::BitAndAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    });
                }

                pub fn bit_or_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitOr<Rhs>>(&self, lhs: &mut T, rhs: T) {
                    self.raw.push_instruction(Instruction::BitOrAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    });
                }

                pub fn bit_xor_assign<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitXor<Rhs>>(&self, lhs: &mut T, rhs: T) {
                    self.raw.push_instruction(Instruction::BitXorAssign {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                    });
                }

                pub fn logical_and<B1, B2>(&self, lhs: B1, rhs: B2) -> Bool
                where
                    B1: SpvRustEq<Bool> + AsPrimitive,
                    B2: SpvRustEq<Bool> + AsPrimitive,
                {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LogicalAnd {
                        lhs: lhs.id(&*self.raw),
                        rhs: rhs.id(&*self.raw),
                        res: new_id,
                    });

                    Bool::from_id(new_id)
                }

                pub fn logical_or<B1, B2>(&self, lhs: B1, rhs: B2) -> Bool
                where
                    B1: SpvRustEq<Bool> + AsPrimitive,
                    B2: SpvRustEq<Bool> + AsPrimitive,
                {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LogicalOr {
                        lhs: lhs.id(&*self.raw),
                        rhs: rhs.id(&*self.raw),
                        res: new_id,
                    });

                    Bool::from_id(new_id)
                }

                pub fn logical_equal<B1, B2>(&self, lhs: B1, rhs: B2) -> Bool
                where
                    B1: SpvRustEq<Bool> + AsPrimitive,
                    B2: SpvRustEq<Bool> + AsPrimitive,
                {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LogicalEqual {
                        lhs: lhs.id(&*self.raw),
                        rhs: rhs.id(&*self.raw),
                        res: new_id,
                    });

                    Bool::from_id(new_id)
                }

                pub fn logical_not<B>(&self, var: B) -> Bool
                where
                    B: SpvRustEq<Bool> + AsPrimitive,
                {
                    let new_id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::LogicalNot {
                        lhs: var.id(&*self.raw),
                        res: new_id,
                    });

                    Bool::from_id(new_id)
                }

                /// Create a new struct
                ///
                /// The fields should be supplied as a slice in order declared
                /// This is to allow creating structs from a composition of both spv types and rust types
                ///
                /// TODO update AsSpvStruct to have an associated type with the same field names
                /// storing &dyn AsData and implement require Into for that associated type. Could be
                /// implemented as a proc macro and also a duplicate type that could have fields of Int, Float ...
                /// or maybe different constructors idk i'm rambling.
                pub fn new_struct<S: AsSpvStruct>(&self, data: &[&dyn AsData]) -> SpvStruct<S> {
                    let id = self.raw.get_new_id();
                    let data = data.iter().map(|d| d.id(&*self.raw)).collect::<Vec<_>>();
                    self.raw.push_instruction(Instruction::NewStruct {
                        data,
                        store: id,
                        ty: DataType::Struct(TypeId::of::<S>(), S::DESC.name, S::DESC.names, S::DESC.fields)
                    });

                    SpvStruct {
                        id,
                        _marker: PhantomData,
                    }
                }

                /// Store the variable into the struct field
                ///
                /// Will panic if the field type doesn't match the type of T
                pub fn struct_store<S, T>(&self, s: SpvStruct<S>, field: &str, data: T)
                where
                    S: AsSpvStruct,
                    T: AsData,
                {
                    let index = S::DESC.names.iter().position(|&name| name.eq(field)).expect(&format!("No field {} on struct", field));
                    assert_eq!(data.ty(), S::DESC.fields[index]);
                    self.raw.push_instruction(Instruction::StructStore {
                        struct_id: s.id,
                        field: index,
                        ty: data.ty(),
                        data: data.id(&*self.raw),
                    })
                }

                /// Load a struct field and return a variable containing the data from that field
                ///
                /// Will panic if the field types doesn't match the type of T
                pub fn struct_load<S, T>(&self, s: SpvStruct<S>, field: &str) -> T
                where
                    S: AsSpvStruct,
                    T: AsData + AsDataType + FromId,
                {
                    let new_id = self.raw.get_new_id();
                    let index = S::DESC.names.iter().position(|&name| name.eq(field)).expect(&format!("Not field {} on struct", field));
                    self.raw.push_instruction(Instruction::StructLoad {
                        struct_id: s.id,
                        field: index,
                        ty: T::TY,
                        store: new_id,
                    });

                    T::from_id(new_id)
                }

                pub fn new_array<const N: usize, T>(&self, data: [&dyn SpvRustEq<T>; N]) -> SpvArray<N, T>
                where
                    T: AsPrimitiveType + AsPrimitive,
                {
                    let id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::NewArray {
                        store: id,
                        ty: T::TY,
                        data: data.iter().map(|e| e.id(&*self.raw)).collect::<Vec<_>>(),
                    });
                    SpvArray {
                        id,
                        _marker: PhantomData,
                    }
                }

                pub fn array_store<const N: usize, S, T>(&self, array: SpvArray<N, T>, index: usize, data: S)
                where
                    S: SpvRustEq<T> + AsPrimitive,
                    T: AsPrimitiveType + AsPrimitive,
                {
                    assert!(index < N);
                    self.raw.push_instruction(Instruction::ArrayStore {
                        array: array.id,
                        index,
                        data: data.id(&*self.raw),
                        element_ty: T::TY,
                    });
                }

                pub fn array_load<const N: usize, T>(&self, array: SpvArray<N, T>, index: usize) -> T
                where
                    T: AsPrimitiveType + AsPrimitive + FromId,
                {
                    assert!(index < N);
                    let id = self.raw.get_new_id();
                    self.raw.push_instruction(Instruction::ArrayLoad {
                        array: array.id,
                        index,
                        store: id,
                        element_ty: T::TY,
                    });
                    T::from_id(id)
                }

                pub fn combine_texture_sampler<D, C>(&self, texture: SpvGTexture<D, C>, sampler: SpvSampler) -> SpvSampledGTexture<D, C>
                where
                    D: AsDimension,
                    C: AsComponent,
                {
                    let new_id = self.raw.get_new_id();

                    self.raw.push_instruction(Instruction::CombineTextureSampler {
                        texture: texture.index,
                        sampler: sampler.index,
                        store: new_id,
                    });

                    SpvSampledGTexture {
                        id: Right(new_id),
                        _dmarker: PhantomData,
                        _cmarker: PhantomData,
                    }
                }

                pub fn sample_texture<D, C, V>(&self, texture: SpvSampledGTexture<D, C>, coordinate: V) -> C::Read
                where
                    D: AsDimension,
                    D::Coord: AsPrimitive + AsPrimitiveType,
                    C: AsComponent,
                    C::Read: FromId + AsPrimitiveType,
                    V: SpvRustEq<D::Coord>,
                {
                    let new_id = self.raw.get_new_id();

                    self.raw.push_instruction(Instruction::SampleTexture {
                        sampled_texture: texture.id,
                        coordinate: coordinate.id(&*self.raw),
                        coordinate_ty: D::Coord::TY,
                        store: new_id,
                        res_ty: C::Read::TY,
                    });

                    C::Read::from_id(new_id)
                }
            }
        )*
    };
}

gen_intrinsics!(FnBuilder, ConditionBuilder, LoopBuilder, MainBuilder,);
