
use slab::Slab;

use std::any::Any;

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

pub trait Scope: AsAny {
    fn push_instruction(&mut self, instruction: crate::Instruction);

    fn get_new_id(&mut self) -> usize;

    fn name_var(&mut self, id: usize, name: String);
}

impl dyn Scope {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref()
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }

    pub fn downcast<T: Any>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        use std::ops::Deref;

        match self.deref().as_any_ref().type_id() == ::std::any::TypeId::of::<T>() {
            true => Ok(self.as_any_box().downcast().unwrap()),
            false => Err(self),
        }
    }
}

pub struct FuncScope {
    pub(crate) instructions: Vec<crate::Instruction>,
    pub(crate) variables: Slab<Option<String>>,
} 

impl Scope for FuncScope {
    fn push_instruction(&mut self, instruction: crate::Instruction) {
        self.instructions.push(instruction)
    }

    fn get_new_id(&mut self) -> usize {
        self.variables.insert(None)
    }

    fn name_var(&mut self, id: usize, name: String) {
        if let Some(n) = self.variables.get_mut(id) {
            *n = Some(name)
        } else {
            eprintln!("Call to FuncScope::name_var({}, {}) no variable match found", id, name);
        }
    }
}

impl FuncScope {
    pub(crate) fn new() -> Self {
        Self {
            variables: Slab::new(),
            instructions: Vec::new(),
        }
    }
}

pub struct IfScope {
    pub(crate) instructions: Vec<crate::Instruction>,
    pub(crate) outer: Box<dyn Scope>,
}

impl Scope for IfScope {
    fn push_instruction(&mut self, instruction: crate::Instruction) {
        self.instructions.push(instruction);
    }

    fn get_new_id(&mut self) -> usize {
        self.outer.get_new_id()
    }

    fn name_var(&mut self, id: usize, name: String) {
        self.outer.name_var(id, name)
    }
}