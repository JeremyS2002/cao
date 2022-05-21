//! let mut b = builder::new();
//! let f: Function<Vec2> = b.spv_fn(|a| {
//!     inputs!(a, x: Vec2, y: Vec2);
//!     // a.inputs(&[Vec2, Vec2])
//!     // let x: Vec2 = a.input(0).unwrap();
//!     // let y: Vec2 = a.input(1).unwrap();
//! 
//!     let t = x + y;
//!     
//!     return!(a, t);
//!     // a.ret(&t: &dyn Data);
//! });
//! 
//! b.spv_main(|m| {
//!     spv_if!(m, (bool1) {
//!         // code1
//!     } (bool2) {
//!         // code2
//!     } (bool3) {
//!         // code3
//!     });
//!     // {
//!     //      let condition_builder = m.condition_builder(bool1, //code1);
//!     //      condition_builder.else_if(bool2, // code2);
//!     //      condition_builder.else_if(bool4, // code3);
//!     //      condition_builder.end_if();
//!     // }
//! 
//!     let a = b.vec2([0.0, 1.0]);
//!     let b = b.vec2([1.0, 0.0]);
//!     let c = f.call(&[&a, &b]);
//! });
//! 

use data::{AsDataType, IsPrimitive};
use interface::{Uniform, Out, In, StorageAccessDesc, Storage};
use rspirv::binary::Assemble;
use either::*;

use std::rc::Rc;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::collections::HashMap;

use builder::{RawBaseBuilder, Instruction};
pub use glam;

pub mod builder;
pub mod data;
pub mod texture;
pub mod interface;
pub mod function;
pub mod specialisation;

pub use specialisation::{
    VertexBuilder,
    FragmentBuilder,
    TessControlBuilder,
    TessEvalBuilder,
    GeometryBuilder,
    ComputeBuilder,
};

pub use data::ty_structs::*;

pub struct Builder<T> {
    /// Always BaseBuilder
    raw: Rc<RawBaseBuilder>,
    _marker: PhantomData<T>,
}

impl<T: specialisation::ShaderTY> Builder<T> {
    pub fn new() -> Self {
        Self {
            raw: Rc::new(RawBaseBuilder::new()),
            _marker: PhantomData,
        }
    }

    pub fn instructions(&self) -> Vec<builder::Instruction> {
        (*self.raw.main.borrow()).clone()
    }

    pub fn functions(&self) -> HashMap<usize, Vec<Instruction>> {
        (*self.raw.functions.borrow()).clone()
    }

    pub fn spv_fn<R: data::DataRef, F: FnOnce(&builder::FnBuilder) -> () + 'static>(&self, f: F) -> function::Function<R> {
        let id = 0;
        
        let b = builder::FnBuilder {
            raw: Rc::new(builder::RawFnBuilder {
                builder: Rc::clone(&self.raw),
                id,
                instructions: RefCell::new(Vec::new()),
                variables: RefCell::default()
            })
        };

        f(&b);

        function::Function { 
            id,
            _marker: PhantomData,
        }
    }

    pub fn input<P: IsPrimitive>(&self, location: u32, name: Option<&'static str>) -> In<P> {
        let index = self.raw.inputs.borrow().len();
        self.raw.inputs.borrow_mut().push((P::TY, Left(location), name));
        In {
            index,
            _marker: PhantomData,
        }
    }

    pub fn output<P: IsPrimitive>(&self, location: u32, name: Option<&'static str>) -> Out<P> {
        let index = self.raw.inputs.borrow().len();
        self.raw.outputs.borrow_mut().push((P::TY, Left(location), name));
        Out { 
            index, 
            _marker: PhantomData
        }
    }

    pub fn uniform<D: AsDataType>(&self) -> Uniform<D> {
        self.raw.uniforms.borrow_mut().push(D::TY);
        // Uniform {
        //     set: todo!(),
        //     binding: todo!(),
        //     _marker: PhantomData,
        // }
        todo!();
    }

    pub fn storage<D: AsDataType>(&self, _desc: StorageAccessDesc) -> Storage<D> {
        self.raw.storages.borrow_mut().push(D::TY);
        // Storage {
        //     set: todo!(),
        //     binding: todo!(),
        //     _marker: PhantomData,
        // }
        todo!();
    }

    pub fn main<F: FnOnce(&builder::MainBuilder) -> ()>(&self, f: F) {
        let b = builder::MainBuilder {
            raw: Rc::new(builder::RawMainBuilder {
                builder: Rc::clone(&self.raw),
                instructions: RefCell::new(Vec::new()),
                variables: RefCell::default(),
            })
        };

        f(&b)
    }

    pub fn compile(self) -> Vec<u32> {
        let mut builder = rspirv::dr::Builder::new();

        let _ext = builder.ext_inst_import("GLSL.std.450");
        builder.set_version(1, 0);
        builder.capability(rspirv::spirv::Capability::Shader);  
        builder.memory_model(rspirv::spirv::AddressingModel::Logical, rspirv::spirv::MemoryModel::GLSL450);

        // map from my function id to rspirv function id
        let function_map = HashMap::new();


        // for (_, function) in self.functions() {
            
        // }

        let mut var_map = HashMap::new();

        let void = builder.type_void();
        let void_f = builder.type_function(void, []);
        let main = builder.begin_function(
            void, 
            None, 
            rspirv::spirv::FunctionControl::empty(),
            void_f
        ).unwrap();
        builder.name(main, "main");

        let inputs = self.raw.inputs
            .borrow()
            .iter()
            .map(|(v, t, name)| {
                let storage = rspirv::spirv::StorageClass::Input;
                let ty = v.raw_ty(&mut builder);
                let pointer_ty = builder.type_pointer(None, storage, ty);
                let variable = builder.variable(pointer_ty, None, storage, None);
                
                match t {
                    Left(location) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::Location,
                        [rspirv::dr::Operand::LiteralInt32(*location)]
                    ),
                    Right(built_in) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::BuiltIn,
                        [rspirv::dr::Operand::BuiltIn(*built_in)]
                    )
                }

                if let Some(name) = name {
                    builder.name(variable, *name);
                }
                variable
            })
            .collect::<Vec<_>>();

        let outputs = self.raw.outputs
            .borrow()
            .iter()
            .map(|(v, t, name)| {
                let storage = rspirv::spirv::StorageClass::Output;
                let ty = v.raw_ty(&mut builder);
                let pointer_ty = builder.type_pointer(None, storage, ty);
                let variable = builder.variable(pointer_ty, None, storage, None);
                
                match t {
                    Left(location) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::Location,
                        [rspirv::dr::Operand::LiteralInt32(*location)]
                    ),
                    Right(built_in) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::BuiltIn,
                        [rspirv::dr::Operand::BuiltIn(*built_in)]
                    )
                }

                if let Some(name) = name {
                    builder.name(variable, *name);
                }
                variable
            })
            .collect::<Vec<_>>();

        let mut interface = inputs.clone();
        interface.extend_from_slice(&outputs);

        builder.entry_point(T::TY, main, "main", interface);

        builder.begin_block(None).unwrap();

        for mut instruction in self.instructions() {
            instruction.process(
                &mut builder, 
                &mut var_map, 
                &function_map,
                &inputs,
                &outputs,
                None,
                None,
            );
        }

        builder.ret().unwrap();
        builder.end_function().unwrap();

        builder.module().assemble()
    }
}
