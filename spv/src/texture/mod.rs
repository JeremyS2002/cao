
pub mod component;

pub use component::*;

use std::marker::PhantomData;

use either::*;

pub trait AsDimension {
    const DIM: rspirv::spirv::Dim;

    type Coord;
}

#[derive(Clone, Copy, Debug)]
pub struct D1 {}

impl AsDimension for D1 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim1D;

    type Coord = crate::Float;
}

#[derive(Clone, Copy, Debug)]
pub struct D2 {}

impl AsDimension for D2 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim2D;

    type Coord = crate::Vec2;
}

#[derive(Clone, Copy, Debug)]
pub struct D3 {}

impl AsDimension for D3 {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::Dim3D;

    type Coord = crate::Vec3;
}

#[derive(Clone, Copy, Debug)]
pub struct Cube {}

impl AsDimension for Cube {
    const DIM: rspirv::spirv::Dim = rspirv::spirv::Dim::DimCube;

    type Coord = crate::Vec3;
}

/// A Raw texture, can be used to read pixels or combined with a sampler to
/// create a [`SampledGTexture`] which can then be sampled from
pub struct RawTexture<D: AsDimension> {
    pub(crate) index: usize,
    pub(crate) _dmarker: PhantomData<D>,
}

pub trait GTexture<D: AsDimension> {
    fn raw_texture(&self) -> &RawTexture<D>;

    type Sampleable: SampledGTexture<D>;
}

pub struct Texture<D: AsDimension>(pub RawTexture<D>);

impl<D: AsDimension> GTexture<D> for Texture<D> {
    fn raw_texture(&self) -> &RawTexture<D> {
        &self.0
    }

    type Sampleable = SampledTexture<D>;
}

pub struct DTexture<D: AsDimension>(pub RawTexture<D>);

impl<D: AsDimension> GTexture<D> for DTexture<D> {
    fn raw_texture(&self) -> &RawTexture<D> {
        &self.0
    }

    type Sampleable = SampledDTexture<D>;
}

pub struct ITexture<D: AsDimension>(pub RawTexture<D>);

impl<D: AsDimension> GTexture<D> for ITexture<D> {
    fn raw_texture(&self) -> &RawTexture<D> {
        &self.0
    }

    type Sampleable = SampledITexture<D>;
}

pub struct UTexture<D: AsDimension>(pub RawTexture<D>);

impl<D: AsDimension> GTexture<D> for UTexture<D> {
    fn raw_texture(&self) -> &RawTexture<D> {
        &self.0
    }

    type Sampleable = SampledUTexture<D>;
}

pub type Texture1D = Texture<D1>;
pub type Texture2D = Texture<D2>;
pub type Texture3D = Texture<D3>;
pub type TextureCube = Texture<Cube>;

pub type DTexture1D = DTexture<D1>;
pub type DTexture2D = DTexture<D2>;
pub type DTexture3D = DTexture<D3>;
pub type DTextureCube = DTexture<Cube>;

pub type ITexture1D = ITexture<D1>;
pub type ITexture2D = ITexture<D2>;
pub type ITexture3D = ITexture<D3>;
pub type ITextureCube = ITexture<Cube>;

pub type UTexture1D = UTexture<D1>;
pub type UTexture2D = UTexture<D2>;
pub type UTexture3D = UTexture<D3>;
pub type UTextureCube = UTexture<Cube>;

pub trait SampledGTexture<D: AsDimension> {
    fn from_id(_id: usize) -> Self;

    fn raw_texture(&self) -> SampledRawTexture<D>;

    type Component: AsComponent;    
}

pub struct SampledRawTexture<D: AsDimension> {
    /// Either Left(index) or Right(id)
    pub(crate) id: Either<usize, usize>,
    pub(crate) _dmarker: PhantomData<D>,
}

impl<D: AsDimension> Clone for SampledRawTexture<D> {
    fn clone(&self) -> Self {
        Self { id: self.id, _dmarker: self._dmarker }
    }
}

impl<D: AsDimension> Copy for SampledRawTexture<D> { }

pub struct SampledTexture<D: AsDimension>(pub SampledRawTexture<D>);

impl<D: AsDimension> SampledGTexture<D> for SampledTexture<D> {
    fn from_id(id: usize) -> Self {
        Self(SampledRawTexture { id: Right(id), _dmarker: PhantomData })
    }

    fn raw_texture(&self) -> SampledRawTexture<D> {
        self.0
    }

    type Component = crate::Float;
}

pub struct SampledDTexture<D: AsDimension>(pub SampledRawTexture<D>);

impl<D: AsDimension> SampledGTexture<D> for SampledDTexture<D> {
    fn from_id(id: usize) -> Self {
        Self(SampledRawTexture { id: Right(id), _dmarker: PhantomData })
    }

    fn raw_texture(&self) -> SampledRawTexture<D> {
        self.0
    }

    type Component = crate::Double;
}

pub struct SampledITexture<D: AsDimension>(pub SampledRawTexture<D>);

impl<D: AsDimension> SampledGTexture<D> for SampledITexture<D> {
    fn from_id(id: usize) -> Self {
        Self(SampledRawTexture { id: Right(id), _dmarker: PhantomData })
    }

    fn raw_texture(&self) -> SampledRawTexture<D> {
        self.0
    }

    type Component = crate::Int;
}

pub struct SampledUTexture<D: AsDimension>(pub SampledRawTexture<D>);

impl<D: AsDimension> SampledGTexture<D> for SampledUTexture<D> {
    fn from_id(id: usize) -> Self {
        Self(SampledRawTexture { id: Right(id), _dmarker: PhantomData })
    }

    fn raw_texture(&self) -> SampledRawTexture<D> {
        self.0
    }

    type Component = crate::UInt;
}

pub type SampledTexture1D = SampledTexture<D1>;
pub type SampledTexture2D = SampledTexture<D2>;
pub type SampledTexture3D = SampledTexture<D3>;
pub type SampledTextureCube = SampledTexture<Cube>;

pub type SampledDTexture1D = SampledDTexture<D1>;
pub type SampledDTexture2D = SampledDTexture<D2>;
pub type SampledDTexture3D = SampledDTexture<D3>;
pub type SampledDTextureCube = SampledDTexture<Cube>;

pub type SampledITexture1D = SampledITexture<D1>;
pub type SampledITexture2D = SampledITexture<D2>;
pub type SampledITexture3D = SampledITexture<D3>;
pub type SampledITextureCube = SampledITexture<Cube>;

pub type SampledUTexture1D = SampledUTexture<D1>;
pub type SampledUTexture2D = SampledUTexture<D2>;
pub type SampledUTexture3D = SampledUTexture<D3>;
pub type SampledUTextureCube = SampledUTexture<Cube>;
