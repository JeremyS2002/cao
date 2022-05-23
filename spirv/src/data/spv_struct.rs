
use std::marker::PhantomData;

use crate::{AsData, builder::Instruction};

use super::{DataType};

pub struct StructDesc {
    pub names: &'static [&'static str],
    pub fields: &'static [DataType],
}

/// Marks a type as being available to be used in a shader
/// 
/// The rust compiler can re-order fields, to prevent this types that
/// implement the AsSpvStruct should be marked repr(C)
/// 
/// The declaration of the type in rust must match the DESC
/// and the fields method should return the fields in order of declaration
/// 
/// If the type is going to be used as a uniform or storage type then 
/// it should also match padding requirements by the spir-v specicifation
pub unsafe trait AsSpvStruct {
    const DESC: StructDesc;

    fn fields<'a>(&'a self) -> &'a [&dyn AsData];
}

impl<T: AsSpvStruct> AsData for T {
    fn id(&self, b: &dyn crate::builder::RawBuilder) -> usize {
        let id = b.get_new_id();
        let data = self.fields().iter().map(|d| d.id(b)).collect::<Vec<_>>();
        b.push_instruction(Instruction::NewStruct {
            data,
            store: id,
            ty: DataType::Struct(Self::DESC.names, Self::DESC.fields)
        });
        id
    }

    fn ty(&self) -> crate::data::DataType {
        crate::data::DataType::Struct(Self::DESC.names, Self::DESC.fields)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpvStruct<S: AsSpvStruct> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<S>,
}
