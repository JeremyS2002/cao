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

use rspirv::binary::Assemble;

use std::rc::Rc;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::collections::HashMap;

use builder::{RawBaseBuilder, Instruction};
pub use glam;

pub mod builder;
pub mod data;

pub struct Function<R> {
    pub(crate) id: usize,
    _marker: PhantomData<R>,
}

pub struct Builder {
    /// Always BaseBuilder
    raw: Rc<RawBaseBuilder>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            raw: Rc::new(RawBaseBuilder::new())
        }
    }

    pub fn instructions(&self) -> Vec<builder::Instruction> {
        (*self.raw.main.borrow()).clone()
    }

    pub fn functions(&self) -> HashMap<usize, Vec<Instruction>> {
        (*self.raw.functions.borrow()).clone()
    }

    pub fn spv_fn<R: data::DataRef, F: FnOnce(&builder::FnBuilder) -> () + 'static>(&self, f: F) -> Function<R> {
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

        Function { 
            id,
            _marker: PhantomData,
        }
    }

    pub fn spv_main<F: FnOnce(&builder::MainBuilder) -> () + 'static>(&self, f: F) {
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

        // map from my function id to rspirv function id
        let mut function_map = HashMap::new();

        for (_, function) in self.functions() {
            
        }

        for instruction in self.instructions() {
            instruction.process(&mut builder, &function_map);
        }
        builder.module().assemble()
    }
}
