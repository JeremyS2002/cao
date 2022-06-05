use std::marker::PhantomData;

use crate::{AsData, IsDataType, AsPrimitiveType};

use super::AsDataType;

#[derive(Clone, Copy, Debug)]
pub struct SpvArray<const N: usize, T: AsPrimitiveType> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<T>,
}

impl<const N: usize, T: AsPrimitiveType> AsDataType for SpvArray<N, T> {
    const TY: crate::data::DataType = crate::data::DataType::Array(T::TY, N);
}

impl<const N: usize, T: AsPrimitiveType> AsData for SpvArray<N, T> {
    fn id(&self, _: &dyn crate::builder::RawBuilder) -> usize {
        self.id
    }

    fn ty(&self) -> crate::data::DataType {
        <Self as AsDataType>::TY
    }
}

impl<const N: usize, T: AsPrimitiveType> IsDataType for SpvArray<N, T> { }
