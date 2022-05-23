#![allow(dead_code)]

use crate::data::{AsDataType, IsPrimitiveType};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct In<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Out<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Uniform<T: AsDataType> {
    pub(crate) set: usize,
    pub(crate) binding: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct StorageAccessDesc {
    pub read: bool,
    pub write: bool,
    pub atomic: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Storage<T: AsDataType> {
    pub(crate) set: usize,
    pub(crate) binding: usize,
    pub(crate) _marker: PhantomData<T>,
}
