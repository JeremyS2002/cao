use std::{any::TypeId, marker::PhantomData};

use crate::{AsData, FromId};

use super::{AsDataType, DataType, IsDataType};

/// Describes a struct that can be used in shaders
pub struct StructDesc {
    /// The name of the struct
    pub name: &'static str,
    /// The names of the structs fields in order of declaration
    pub names: &'static [&'static str],
    /// The types of the structs fields in order of declaration
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
pub unsafe trait AsSpvStruct: 'static {
    const DESC: StructDesc;

    fn fields<'a>(&'a self) -> Vec<&'a dyn AsData>;
}

impl<T: AsSpvStruct> AsDataType for Struct<T> {
    const TY: DataType = DataType::Struct(
        TypeId::of::<T>(),
        T::DESC.name,
        T::DESC.names,
        T::DESC.fields,
    );
}

impl<T: AsSpvStruct> AsData for Struct<T> {
    fn id(&self, _: &dyn crate::builder::RawBuilder) -> usize {
        // let id = b.get_new_id();
        // let data = self.fields().iter().map(|d| d.id(b)).collect::<Vec<_>>();
        // b.push_instruction(Instruction::NewStruct {
        //     data,
        //     store: id,
        //     ty: DataType::Struct(TypeId::of::<T>(), Self::DESC.name, Self::DESC.names, Self::DESC.fields),
        // });
        // id
        self.id
    }

    fn ty(&self) -> crate::data::DataType {
        crate::data::DataType::Struct(
            TypeId::of::<T>(),
            T::DESC.name,
            T::DESC.names,
            T::DESC.fields,
        )
    }
}

impl<T: AsSpvStruct> IsDataType for Struct<T> {}

#[derive(Clone, Copy, Debug)]
pub struct Struct<S: AsSpvStruct> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<S>,
}

impl<S: AsSpvStruct> FromId for Struct<S> {
    fn from_id(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}
