
use core::panic;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IOType {
    Int,
    IVec2,
    IVec3,
    IVec4,
    UInt,
    UVec2,
    UVec3,
    UVec4,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Double,
    DVec2,
    DVec3,
    DVec4,
}

impl IOType {
    pub fn ty(&self) -> crate::Type {
        match self {
            IOType::Int => crate::Type::Scalar(crate::ScalarType::Signed(32)),
            IOType::IVec2 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Signed(32),
                n_scalar: 2,
            }),
            IOType::IVec3 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Signed(32),
                n_scalar: 3,
            }),
            IOType::IVec4 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Signed(32),
                n_scalar: 4,
            }),
            IOType::UInt => crate::Type::Scalar(crate::ScalarType::Unsigned(32)),
            IOType::UVec2 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Unsigned(32),
                n_scalar: 2,
            }),
            IOType::UVec3 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Unsigned(32),
                n_scalar: 3,
            }),
            IOType::UVec4 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Unsigned(32),
                n_scalar: 4,
            }),
            IOType::Float => crate::Type::Scalar(crate::ScalarType::Float(32)),
            IOType::Vec2 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(32),
                n_scalar: 2,
            }),
            IOType::Vec3 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(32),
                n_scalar: 3,
            }),
            IOType::Vec4 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(32),
                n_scalar: 4,
            }),
            IOType::Double => crate::Type::Scalar(crate::ScalarType::Float(64)),
            IOType::DVec2 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(64),
                n_scalar: 2,
            }),
            IOType::DVec3 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(64),
                n_scalar: 3,
            }),
            IOType::DVec4 => crate::Type::Vector(crate::VectorType {
                scalar_ty: crate::ScalarType::Float(64),
                n_scalar: 4,
            }),
        }
    }
}

pub struct IOInt;
pub struct IOIVec2;
pub struct IOIVec3;
pub struct IOIVec4;
pub struct IOUInt;
pub struct IOUVec2;
pub struct IOUVec3;
pub struct IOUVec4;
pub struct IOFloat;
pub struct IOVec2;
pub struct IOVec3;
pub struct IOVec4;
pub struct IODouble;
pub struct IODVec2;
pub struct IODVec3;
pub struct IODVec4;

pub trait AsIOTypeConst { 
    const IO_TY: IOType;
}

impl AsIOTypeConst for IOInt { 
    const IO_TY: IOType = IOType::Int;
}
impl AsIOTypeConst for IOIVec2 { 
    const IO_TY: IOType = IOType::IVec2;
}
impl AsIOTypeConst for IOIVec3 { 
    const IO_TY: IOType = IOType::IVec3;
}
impl AsIOTypeConst for IOIVec4 { 
    const IO_TY: IOType = IOType::IVec4;
}

impl AsIOTypeConst for IOUInt { 
    const IO_TY: IOType = IOType::UInt;
}
impl AsIOTypeConst for IOUVec2 { 
    const IO_TY: IOType = IOType::UVec2;
}
impl AsIOTypeConst for IOUVec3 { 
    const IO_TY: IOType = IOType::UVec3;
}
impl AsIOTypeConst for IOUVec4 { 
    const IO_TY: IOType = IOType::UVec4;
}

impl AsIOTypeConst for IOFloat { 
    const IO_TY: IOType = IOType::Float;
}
impl AsIOTypeConst for IOVec2 { 
    const IO_TY: IOType = IOType::Vec2;
}
impl AsIOTypeConst for IOVec3 { 
    const IO_TY: IOType = IOType::Vec3;
}
impl AsIOTypeConst for IOVec4 { 
    const IO_TY: IOType = IOType::Vec4;
}

impl AsIOTypeConst for IODouble { 
    const IO_TY: IOType = IOType::Double;
}
impl AsIOTypeConst for IODVec2 { 
    const IO_TY: IOType = IOType::DVec2;
}
impl AsIOTypeConst for IODVec3 { 
    const IO_TY: IOType = IOType::DVec3;
}
impl AsIOTypeConst for IODVec4 { 
    const IO_TY: IOType = IOType::DVec4;
}

pub struct Input<T: AsIOTypeConst> {
    pub(crate) id: usize,
    pub(crate) inner: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T: AsIOTypeConst> Input<T> {
    fn raw_load(&self) -> usize {
        let mut inner = self.inner.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let store = scope.get_new_id();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::IO_TY.ty(),
                src: crate::OpLoadStoreData::Input { location: self.id },
                dst: crate::OpLoadStoreData::Variable { id: store },
            }));

            store
        } else {
            panic!("Error cannot load input when not in function");
        }
    }
}

pub struct Output<T: AsIOTypeConst> {
    pub(crate) id: usize,
    pub(crate) inner: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>, 
}

impl<T: AsIOTypeConst> Output<T> {
    fn raw_store(&self, id: usize) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::IO_TY.ty(),
                src: crate::OpLoadStoreData::Variable { id },
                dst: crate::OpLoadStoreData::Output { location: self.id }
            }))
        } else {
            panic!("Error cannot store output when not in function");
        }
    }
}

macro_rules! impl_io {
    ($($io:ident, $ty:ident,)*) => {
        $(
            impl Input<$io> {
                pub fn load<'a>(&'a self) -> crate::$ty<'a> {
                    let id = self.raw_load();//<$io as AsIOTypeConst>::IO_TY);
                    crate::$ty {
                        id,
                        b: &self.inner,
                    }
                }
            }

            impl Output<$io> {
                pub fn store(&self, data: crate::$ty<'_>) {
                    self.raw_store(data.id);//, <$io as AsIOTypeConst>::IO_TY);
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_io!(
    IOInt, Int,
    IOIVec2, IVec2,
    IOIVec3, IVec3,
    IOIVec4, IVec4,

    IOUInt, UInt,
    IOUVec2, UVec2,
    IOUVec3, UVec3,
    IOUVec4, UVec4,

    IOFloat, Float,
    IOVec2, Vec2,
    IOVec3, Vec3,
    IOVec4, Vec4,

    IODouble, Double,
    IODVec2, DVec2,
    IODVec3, DVec3,
    IODVec4, DVec4,
);
