use std::marker::PhantomData;

use crate::PrimitiveType;


pub trait AsDimension {
    const DIM: rspirv::spirv::Dim;

    type Coord;
}

pub struct D1 { }

impl AsDimension for D1 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim1D;

    type Coord = crate::Float;
}

pub struct D2 { }

impl AsDimension for D2 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim2D;

    type Coord = crate::Vec2;
}

pub struct D3 { }

impl AsDimension for D3 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim3D;

    type Coord = crate::Vec3;
}

pub struct Cube { }

impl AsDimension for Cube {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::DimCube;

    type Coord = crate::Vec3;
}

#[derive(Clone, Copy, Debug)]
pub enum Component {
    Float,
    Double,
    Int,
    UInt,
}

impl From<Component> for PrimitiveType {
    fn from(c: Component) -> Self {
        Self::from(&c)
    }
}

impl From<&'_ Component> for PrimitiveType {
    fn from(c: &'_ Component) -> Self {
        match c {
            Component::Float => Self::Float,
            Component::Double => Self::Double,
            Component::Int => Self::Int,
            Component::UInt => Self::UInt,
        }
    }
}

impl Component {
    pub(crate) fn base_type(&self, builder: &mut rspirv::dr::Builder) -> u32 {
        PrimitiveType::from(*self).base_type(builder)
    }
}

pub trait AsComponent {
    const COMPONENT: Component;

    type Read;
}

impl AsComponent for crate::Float {
    const COMPONENT: Component = Component::Float;

    type Read = crate::Vec4;
}

impl AsComponent for crate::Double {
    const COMPONENT: Component = Component::Double;

    type Read = crate::DVec4;
}

impl AsComponent for crate::Int {
    const COMPONENT: Component = Component::Int;

    type Read = crate::IVec4;
}

impl AsComponent for crate::UInt {
    const COMPONENT: Component = Component::UInt;

    type Read = crate::UVec2;
}

pub struct SpvGTexture<D: AsDimension, C: AsComponent> {
    pub(crate) index: usize,
    pub(crate) _dmarker: PhantomData<D>,
    pub(crate) _cmarker: PhantomData<C>,
}

pub type SpvTexture<D> = SpvGTexture<D, crate::Float>;
pub type SpvDTexture<D> = SpvGTexture<D, crate::Double>;
pub type SpvITexture<D> = SpvGTexture<D, crate::Int>;
pub type SpvUTexture<D> = SpvGTexture<D, crate::UInt>;

pub type SpvTexture1D = SpvTexture<D1>;
pub type SpvTexture2D = SpvTexture<D2>;
pub type SpvTexture3D = SpvTexture<D3>;
pub type SpvTextureCube = SpvTexture<Cube>;

pub type SpvDTexture1D = SpvDTexture<D1>;
pub type SpvDTexture2D = SpvDTexture<D2>;
pub type SpvDTexture3D = SpvDTexture<D3>;
pub type SpvDTextureCube = SpvDTexture<Cube>;

pub type SpvITexture1D = SpvITexture<D1>;
pub type SpvITexture2D = SpvITexture<D2>;
pub type SpvITexture3D = SpvITexture<D3>;
pub type SpvITextureCube = SpvITexture<Cube>;

pub type SpvUTexture1D = SpvUTexture<D1>;
pub type SpvUTexture2D = SpvUTexture<D2>;
pub type SpvUTexture3D = SpvUTexture<D3>;
pub type SpvUTextureCube = SpvUTexture<Cube>;