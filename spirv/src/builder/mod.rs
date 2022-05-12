
use std::rc::Rc;
use std::any::Any;
use std::collections::HashMap;

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

pub mod base_builder;
pub mod fn_builder;
pub mod main_builder;
pub mod condition_builder;
pub mod loop_builder;
pub mod var;

pub(crate) use base_builder::*;
pub(crate) use var::*;
pub use fn_builder::*;
pub use main_builder::*;
pub use condition_builder::*;
pub use loop_builder::*;

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

    fn name_var(&self, ty: DataType, id: usize, name: String);

    fn get_new_id(&self, ty: DataType) -> usize;
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

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum Instruction {
    Store {
        /// Declare a constant with this value
        val: DataVal,
        /// Store into variable with this id
        store: (usize, crate::data::DataType),
    },
    Add {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    Sub {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    Mul {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    Div {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    BitAnd {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    BitOr {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    BitXor {
        lhs: (usize, crate::data::DataType),
        rhs: (usize, crate::data::DataType),
        res: (usize, crate::data::DataType),
    },
    IfChain {
        conditions: Vec<usize>,
        instructions: Vec<Vec<Instruction>>,
    },
    Loop {
        condition: usize,
        body: Vec<Instruction>,
    },
    Break,
    Continue,
    FnCall {
        fn_id: usize,
        store_id: usize,
        arguments: Vec<(usize, crate::data::DataType)>,
    },
    Return {
        id: usize,
    }
}

impl Instruction {
    pub fn process(&self, builder: &mut rspirv::dr::Builder, function_map: &HashMap<usize, usize>) {
        match self {
            Instruction::Store { 
                val, 
                store 
            } => {
                
            },
            Instruction::Add { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::Sub { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::Mul { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::Div { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::BitAnd { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::BitOr { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::BitXor { 
                lhs, 
                rhs, 
                res 
            } => {
                
            },
            Instruction::IfChain { 
                conditions, 
                instructions 
            } => {
                
            },
            Instruction::Loop { 
                condition, 
                body 
            } => {
                
            },
            Instruction::Break => {
                
            },
            Instruction::Continue => {
                
            },
            Instruction::FnCall { 
                fn_id, 
                store_id, 
                arguments 
            } => {
                
            },
            Instruction::Return { 
                id 
            } => {
                
            },
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
                    new_bvec2, BVec2, GlamBVec2,
                    new_bvec3, BVec3, GlamBVec3,
                    new_bvec4, BVec4, GlamBVec4,
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
                let id = self.raw.get_new_id(crate::data::DataType::$t);
                self.raw.push_instruction(Instruction::Store {
                    val: crate::data::DataVal::$t(v),
                    store: (id, crate::data::DataType::$t),
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
                pub fn spv_if<F: FnOnce(&ConditionBuilder)>(&self, b: &Bool, f: F) -> ConditionBuilder {                
                    let b = ConditionBuilder {
                        raw: RawConditionBuilder::new(Rc::clone(&self.raw), b.id),
                    };
    
                    f(&b);
    
                    b
                }
    
                pub fn spv_while<F: FnOnce(&LoopBuilder)>(&self, b: &Bool, f: F) {
                    let b = LoopBuilder {
                        raw: RawLoopBuilder::new(Rc::clone(&self.raw), b.id),
                    };

                    f(&b);

                    drop(b);
                }

                pub fn spv_break(&self) {
                    self.raw.push_instruction(Instruction::Break);
                }

                pub fn spv_continue(&self) {
                    self.raw.push_instruction(Instruction::Continue);
                }

                pub fn spv_return(&self, val: impl crate::data::AsData) {
                    self.raw.push_instruction(Instruction::Return {
                        id: val.id(&*self.raw)
                    })
                }

                pub fn spv_call<R: AsDataType + FromId>(&self, f: crate::Function<R>, args: &[&dyn AsData]) -> R {
                    let store_id = self.raw.get_new_id(R::TY);

                    self.raw.push_instruction(Instruction::FnCall {
                        fn_id: f.id,
                        store_id,
                        arguments: args.iter().map(|a| (a.id(&*self.raw), a.ty())).collect(),
                    });

                    R::from_id(store_id)
                }

                pub fn spv_store<Rhs: AsDataType, T: SpvStore<Rhs>>(&self, lhs: T, rhs: Rhs) {
                    self.raw.push_instruction(Instruction::Store {
                        store: (lhs.id(&*self.raw), T::TY),
                        val: T::val(rhs),
                    })
                }

                pub fn spv_add<Rhs: AsDataType + AsData, T: SpvAdd<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Add {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn spv_sub<Rhs: AsDataType + AsData, T: SpvSub<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Sub {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn spv_div<Rhs: AsDataType + AsData, T: SpvDiv<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Div {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
                }

                pub fn spv_mul<Rhs: AsDataType + AsData, T: SpvMul<Rhs>>(&self, lhs: T, rhs: Rhs) -> T::Output {
                    let new_id = self.raw.get_new_id(T::Output::TY);
                    self.raw.push_instruction(Instruction::Mul {
                        lhs: (lhs.id(&*self.raw), T::TY),
                        rhs: (rhs.id(&*self.raw), Rhs::TY),
                        res: (new_id, T::Output::TY),
                    });
                    T::Output::from_id(new_id)
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