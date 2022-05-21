
use std::rc::Rc;
use std::any::Any;

// If I knew how to write macros properly this wouldn't be here but this is easier than learning proper macros
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

pub mod base_builder;
pub mod fn_builder;
pub mod main_builder;
pub mod condition_builder;
pub mod loop_builder;
pub mod var;
pub mod instruction;

pub(crate) use base_builder::*;
pub(crate) use var::*;
pub use fn_builder::*;
pub use main_builder::*;
pub use condition_builder::*;
pub use loop_builder::*;
pub use instruction::*;

use crate::data::*;

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

    fn name_var(&self, ty: PrimitiveType, id: usize, name: String);

    fn get_new_id(&self, ty: PrimitiveType) -> usize;

    fn in_loop(&self) -> bool;
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
            false => Err(self)
        }
    }
}

#[macro_export]
macro_rules! gen_get_types {
    ($($name:ident,)*) => {
        $(
            impl $name {
                crate::gen_get_type!(
                    new_bool, Bool, bool,
                    new_float, Float, f32,
                    new_int, Int, i32,
                    new_uint, UInt, u32,
                    new_double, Double, f64,
                    new_ivec2, IVec2, GlamIVec2,
                    new_ivec3, IVec3, GlamIVec3,
                    new_ivec4, IVec4, GlamIVec4,
                    new_uvec2, UVec2, GlamUVec2,
                    new_uvec3, UVec3, GlamUVec3,
                    new_uvec4, UVec4, GlamUVec4,
                    new_vec2, Vec2, GlamVec2,
                    new_vec3, Vec3, GlamVec3,
                    new_vec4, Vec4, GlamVec4,
                    new_dvec2, DVec2, GlamDVec2,
                    new_dvec3, DVec3, GlamDVec3,
                    new_dvec4, DVec4, GlamDVec4,
                    new_mat2, Mat2, GlamMat2,
                    new_mat3, Mat3, GlamMat3,
                    new_mat4, Mat4, GlamMat4,
                    new_dmat2, DMat2, GlamDMat2,
                    new_dmat3, DMat3, GlamDMat3,
                    new_dmat4, DMat4, GlamDMat4,
                );
            }
        )*
    };
}

#[macro_export]
macro_rules! gen_get_type {
    ($($f:ident, $t:ident, $rust:ident,)*) => {
        $(
            pub fn $f(&self, v: $rust) -> crate::data::$t {
                let id = self.raw.get_new_id(crate::data::PrimitiveType::$t);
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

gen_get_types!(
    FnBuilder,
    ConditionBuilder,
    LoopBuilder,
    MainBuilder,
);

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

                pub fn call<R: AsPrimitiveType + FromId>(&self, f: crate::function::Function<R>, args: &[&dyn AsPrimitive]) -> R {
                    let store_id = self.raw.get_new_id(R::TY);

                    self.raw.push_instruction(Instruction::FnCall {
                        fn_id: f.id,
                        store_id,
                        arguments: args.iter().map(|a| (a.id(&*self.raw), a.ty())).collect(),
                    });

                    R::from_id(store_id)
                }

                pub fn load_in<T: crate::data::IsPrimitive + FromId>(&self, input: crate::interface::In<T>) -> T {
                    let store = self.raw.get_new_id(T::TY);
                    self.raw.push_instruction(Instruction::LoadIn {
                        index: input.index,
                        ty: T::TY,
                        store,
                    });
                    T::from_id(store)
                }

                pub fn store_out<T, S>(&self, output: crate::interface::Out<T>, store: S) 
                where
                    T: crate::IsPrimitive + crate::data::SpvRustEq<S>,
                    S: AsPrimitive,
                {
                    self.raw.push_instruction(Instruction::StoreOut {
                        index: output.index,
                        ty: T::TY,
                        read: crate::data::AsPrimitive::id(&store, &*self.raw),
                    })
                }

                pub fn vector_shuffle<V: AsPrimitiveType + FromId>(&self, s: VectorShuffle<V>) -> V {
                    let new_id = self.raw.get_new_id(V::TY);
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
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Add {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn sub<Rhs: AsPrimitiveType + AsPrimitive, T: SpvSub<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Sub {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn div<Rhs: AsPrimitiveType + AsPrimitive, T: SpvDiv<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Div {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn mul<Rhs: AsPrimitiveType + AsPrimitive, T: SpvMul<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
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
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::BitAnd {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn bit_or<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitOr<Rhs>>(&self, lhs: T, rhs: T) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::BitOr {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn bit_xor<Rhs: AsPrimitiveType + AsPrimitive, T: SpvBitXor<Rhs>>(&self, lhs: T, rhs: T) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
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
            }
        )*
    };
}

gen_intrinsics!(
    FnBuilder,
    ConditionBuilder,
    LoopBuilder,
    MainBuilder,
);