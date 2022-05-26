#![feature(const_type_id)]

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

use data::IsPrimitiveType;
use either::*;
use interface::{In, Out, Storage, StorageAccessDesc, Uniform};
use rspirv::binary::Assemble;

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use builder::{Instruction, RawBaseBuilder};
pub use glam;

pub mod builder;
pub mod data;
pub mod function;
pub mod interface;
pub mod specialisation;
pub mod texture;

pub use specialisation::{
    ComputeBuilder, FragmentBuilder, GeometryBuilder, TessControlBuilder, TessEvalBuilder,
    VertexBuilder,
};

pub use data::ty_structs::*;

pub struct Builder<T> {
    /// Well well well, look who wants implement more features and can't remember how this works. 
    /// 
    /// Overview:
    ///     - stage 1. build a vector of [`Instruction`] for each function
    ///     - stage 2. iterate over the instructions to compile a spir-v module
    /// 
    /// Stage 1 and 2 should be combined and done at the same time. There is no reason not to do this other than
    /// I got confused and it would be alot of work to change it now.
    /// 
    /// Stage 1.
    ///     - Each variable (Primitives, Structs and Arrays) is represented by a unique usize
    ///     - There are multiple Builder types to make creating instructions easier
    ///         - MainBuilder : used to create the main function (TODO implement as a wrapper around FunctionBuilder<Void>)
    ///         - FunctionBuilder : used to create arbitrary functions
    ///         - ConditionBuilder : used to create IfChain instructions
    ///         - LoopBuilder : used to create Loop instructions
    ///     - Each builder type implements the same instruction set (implemented via a macro that duplicates the same code on each builder)
    ///     - Each builder has a RawBuilder inside that points to the previous builder
    ///         - MainBuilder and FunctionBuilder both point to RawBaseBuilder
    ///         - ConditionBuilder and LoopBuilder can point to any other builder
    ///     - RawBuilders provide an interface to push instructions and get new id's
    /// 
    /// Stage 2.
    ///     - basically just ```for instruction in instructions { instruction.process(..) }```
    ///     - maps store relation between my (usize) variables id's and rspirv (u32) variable ids
    ///     - all data is stored as spri-v variables behind function local pointers
    ///     - map from StructDesc to spir-v struct id caches based on pointers not data so if anything breaks this could be it
    
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

    pub fn spv_fn<R: data::DataRef, F: FnOnce(&builder::FnBuilder) -> () + 'static>(
        &self,
        f: F,
    ) -> function::Function<R> {
        let id = 0;

        let b = builder::FnBuilder {
            raw: Rc::new(builder::RawFnBuilder {
                builder: Rc::clone(&self.raw),
                id,
                instructions: RefCell::new(Vec::new()),
                variables: RefCell::default(),
            }),
        };

        f(&b);

        function::Function {
            id,
            _marker: PhantomData,
        }
    }

    pub fn input<P: IsPrimitiveType>(&self, location: u32, name: Option<&'static str>) -> In<P> {
        let index = self.raw.inputs.borrow().len();
        self.raw
            .inputs
            .borrow_mut()
            .push((P::TY, Left(location), name));
        In {
            index,
            _marker: PhantomData,
        }
    }

    pub fn output<P: IsPrimitiveType>(&self, location: u32, name: Option<&'static str>) -> Out<P> {
        let index = self.raw.inputs.borrow().len();
        self.raw
            .outputs
            .borrow_mut()
            .push((P::TY, Left(location), name));
        Out {
            index,
            _marker: PhantomData,
        }
    }

    pub fn uniform<D: IsDataType>(&self, set: u32, binding: u32) -> Uniform<D> {
        let index = self.raw.uniforms.borrow().len();
        self.raw.uniforms.borrow_mut().push((
            D::TY,
            set,
            binding,
        ));
        Uniform { 
            index,
            _marker: PhantomData,
        }
    }

    pub fn storage<D: IsDataType>(&self, _desc: StorageAccessDesc, set: u32, binding: u32) -> Storage<D> {
        self.raw.storages.borrow_mut().push((
            D::TY,
            set,
            binding,
        ));
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
            }),
        };

        f(&b)
    }

    pub fn compile(self) -> Vec<u32> {
        let mut builder = rspirv::dr::Builder::new();

        let _ext = builder.ext_inst_import("GLSL.std.450");
        builder.set_version(1, 0);
        builder.capability(rspirv::spirv::Capability::Shader);
        builder.memory_model(
            rspirv::spirv::AddressingModel::Logical,
            rspirv::spirv::MemoryModel::GLSL450,
        );

        // map from my function id to rspirv function id
        let function_map = HashMap::new();
        let mut struct_map = HashMap::new();

        // for (_, function) in self.functions() {

        // }

        let mut var_map = HashMap::new();

        let void = builder.type_void();
        let void_f = builder.type_function(void, []);
        let main = builder
            .begin_function(void, None, rspirv::spirv::FunctionControl::empty(), void_f)
            .unwrap();
        builder.name(main, "main");

        let mut uniforms = Vec::new();
        for (uniform, set, binding) in &*self.raw.uniforms.borrow() {
            let raw_inner_ty = uniform.raw_ty(&mut builder, &mut struct_map);
            let raw_outer_ty = builder.type_struct([raw_inner_ty]);

            builder.decorate(raw_outer_ty, rspirv::spirv::Decoration::Block, None);
            
            let p_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, raw_outer_ty);
            let variable = builder.variable(p_ty, None, rspirv::spirv::StorageClass::Uniform, None);

            builder.decorate(
                variable, 
                rspirv::spirv::Decoration::DescriptorSet, 
                Some(rspirv::dr::Operand::LiteralInt32(*set))
            );
            builder.decorate(
                variable, 
                rspirv::spirv::Decoration::Binding, 
                Some(rspirv::dr::Operand::LiteralInt32(*binding))
            );
            uniforms.push(variable);
        }

        let storages = Vec::new();

        let inputs = self
            .raw
            .inputs
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
                        [rspirv::dr::Operand::LiteralInt32(*location)],
                    ),
                    Right(built_in) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::BuiltIn,
                        [rspirv::dr::Operand::BuiltIn(*built_in)],
                    ),
                }

                if let Some(name) = name {
                    builder.name(variable, *name);
                }
                variable
            })
            .collect::<Vec<_>>();

        let outputs = self
            .raw
            .outputs
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
                        [rspirv::dr::Operand::LiteralInt32(*location)],
                    ),
                    Right(built_in) => builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::BuiltIn,
                        [rspirv::dr::Operand::BuiltIn(*built_in)],
                    ),
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
                &mut struct_map,
                &uniforms,
                &storages,
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
