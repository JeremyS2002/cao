#![allow(dead_code)]

use crate::data::{IsDataType, IsPrimitiveType};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct SpvInput<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct SpvOutput<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct SpvUniform<T: IsDataType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct StorageAccessDesc {
    pub read: bool,
    pub write: bool,
    pub atomic: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SpvStorage<T: IsDataType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}
