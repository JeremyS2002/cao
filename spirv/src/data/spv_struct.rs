
use std::marker::PhantomData;

use super::PrimitiveType;

pub trait AsSpvStruct<const N: usize> {
    const DESC: StructDesc<N>;
}

pub struct StructDesc<const N: usize> {
    pub names: [&'static str; N],
    pub fields: [PrimitiveType; N]
}

pub struct SpvStruct<const N: usize, S: AsSpvStruct<N>> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<S>,
}