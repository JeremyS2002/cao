
use std::marker::PhantomData;

use crate::AsPrimitiveType;

use super::AsDataType;

#[derive(Clone, Copy, Debug)]
pub struct SpvArray<const N: usize, T: AsPrimitiveType> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<T>,
}

impl<const N: usize, T: AsPrimitiveType> AsDataType for SpvArray<N, T> {
    const TY: crate::data::DataType = crate::data::DataType::Array(T::TY, N);
}
