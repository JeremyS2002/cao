
pub trait AsDimension: std::fmt::Debug {
    fn as_dimension(&self) -> gpu::TextureDimension;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D1(pub gpu::Size);

impl D1 {
    pub fn new(size: gpu::Size) -> Self {
        Self(size)
    }
}

impl AsDimension for D1 {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D1;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D1(self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D1Array(pub gpu::Size, pub gpu::Layer);

impl D1Array {
    pub fn new(size: gpu::Size, layers: gpu::Layer) -> Self {
        Self(size, layers)
    }
}

impl AsDimension for D1Array {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D1Array;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D1Array(self.0, self.1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D2(pub gpu::Size, pub gpu::Size);

impl D2 {
    pub fn new(width: gpu::Size, height: gpu::Size) -> Self {
        Self(width, height)
    }
}

impl AsDimension for D2 {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D2;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D2(self.0, self.1, gpu::Samples::S1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D2Ms(pub gpu::Size, pub gpu::Size, pub gpu::Samples);

impl D2Ms {
    pub fn new(width: gpu::Size, height: gpu::Size, samples: gpu::Samples) -> Self {
        Self(width, height, samples)
    }
}

impl AsDimension for D2Ms {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D2Ms;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D2(self.0, self.1, self.2)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D2Array(
    pub gpu::Size,
    pub gpu::Size,
    pub gpu::Samples,
    pub gpu::Layer,
);

impl D2Array {
    pub fn new(
        width: gpu::Size,
        height: gpu::Size,
        samples: gpu::Samples,
        layers: gpu::Layer,
    ) -> Self {
        Self(width, height, samples, layers)
    }
}

impl AsDimension for D2Array {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D2Array;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D2Array(self.0, self.1, self.2, self.3)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct D3(pub gpu::Size, pub gpu::Size, pub gpu::Size);

impl AsDimension for D3 {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::D3;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::D3(self.0, self.1, self.2)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Cube(pub gpu::Size, pub gpu::Size);

impl AsDimension for Cube {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::Cube;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::Cube(self.0, self.1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeArray(pub gpu::Size, pub gpu::Size, pub gpu::Layer);

impl AsDimension for CubeArray {
    #[cfg(feature = "spirv")]
    type Spirv = spirv::CubeArray;

    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::CubeArray(self.0, self.1, self.2)
    }
}

/*#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeMs(pub gpu::Size, pub gpu::Size, pub gpu::Samples);
impl AsDimension for CubeMs {
    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::CubeMs(self.0, self.1, self.2)
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeArrayMs(pub gpu::Size, pub gpu::Size, pub gpu::Layer, pub crate:Samples);
impl AsDimension for CubeArrayMs {
    fn as_dimension(&self) -> gpu::TextureDimension {
        gpu::TextureDimension::CubeArray(self.0, self.1, self.2, self.3)
    }
}*/

/// Allows for infering the type of textures from image pixels
pub trait FormatData {
    const FORMAT: gpu::Format;
}

macro_rules! rust_to_format {
    (
        $( ($($rust:tt)*) = $name:ident, )*
    ) => {
        $(
            impl FormatData for $($rust)* {
                const FORMAT: gpu::Format = gpu::Format::$name;
            }
        )*
    };
}

rust_to_format!(
    (u8) = R8Unorm,
    (i8) = R8Snorm,
    (u16) = R16Unorm,
    (i16) = R16Snorm,
    (u32) = R32Uint,
    (i32) = R32Sint,
    (f32) = R32Float,
    (u64) = R64Uint,
    (i64) = R64Sint,
    (f64) = R64Float,
    ((u8, u8)) = Rg8Unorm,
    ((i8, i8)) = Rg8Snorm,
    ((u16, u16)) = Rg16Unorm,
    ((i16, i16)) = Rg16Snorm,
    ((u32, u32)) = Rg32Uint,
    ((i32, i32)) = Rg32Sint,
    ((f32, f32)) = Rg32Float,
    ((u64, u64)) = Rg64Uint,
    ((i64, i64)) = Rg64Sint,
    ((f64, f64)) = Rg64Float,
    ((u8, u8, u8)) = Rgb8Unorm,
    ((i8, i8, i8)) = Rgb8Snorm,
    ((u16, u16, u16)) = Rgb16Unorm,
    ((i16, i16, i16)) = Rgb16Snorm,
    ((u32, u32, u32)) = Rgb32Uint,
    ((i32, i32, i32)) = Rgb32Sint,
    ((f32, f32, f32)) = Rgb32Float,
    ((u64, u64, u64)) = Rgb64Uint,
    ((i64, i64, i64)) = Rgb64Sint,
    ((f64, f64, f64)) = Rgb64Float,
    ((u8, u8, u8, u8)) = Rgba8Unorm,
    ((i8, i8, i8, i8)) = Rgba8Snorm,
    ((u16, u16, u16, u16)) = Rgba16Unorm,
    ((i16, i16, i16, i16)) = Rgba16Snorm,
    ((u32, u32, u32, u32)) = Rgba32Uint,
    ((i32, i32, i32, i32)) = Rgba32Sint,
    ((f32, f32, f32, f32)) = Rgba32Float,
    ((u64, u64, u64, u64)) = Rgba64Uint,
    ((i64, i64, i64, i64)) = Rgba64Sint,
    ((f64, f64, f64, f64)) = Rgba64Float,
    ([u8; 2]) = Rg8Unorm,
    ([i8; 2]) = Rg8Snorm,
    ([u16; 2]) = Rg16Unorm,
    ([i16; 2]) = Rg16Snorm,
    ([u32; 2]) = Rg32Uint,
    ([i32; 2]) = Rg32Sint,
    ([f32; 2]) = Rg32Float,
    ([u64; 2]) = Rg64Uint,
    ([i64; 2]) = Rg64Sint,
    ([f64; 2]) = Rg64Float,
    ([u8; 3]) = Rgb8Unorm,
    ([i8; 3]) = Rgb8Snorm,
    ([u16; 3]) = Rgb16Unorm,
    ([i16; 3]) = Rgb16Snorm,
    ([u32; 3]) = Rgb32Uint,
    ([i32; 3]) = Rgb32Sint,
    ([f32; 3]) = Rgb32Float,
    ([u64; 3]) = Rgb64Uint,
    ([i64; 3]) = Rgb64Sint,
    ([f64; 3]) = Rgb64Float,
    ([u8; 4]) = Rgba8Unorm,
    ([i8; 4]) = Rgba8Snorm,
    ([u16; 4]) = Rgba16Unorm,
    ([i16; 4]) = Rgba16Snorm,
    ([u32; 4]) = Rgba32Uint,
    ([i32; 4]) = Rgba32Sint,
    ([f32; 4]) = Rgba32Float,
    ([u64; 4]) = Rgba64Uint,
    ([i64; 4]) = Rgba64Sint,
    ([f64; 4]) = Rgba64Float,
);

#[cfg(feature = "image")]
rust_to_format!(
    (image::Luma<u8>) = R8Unorm,
    (image::Luma<i8>) = R8Snorm,
    (image::Luma<u16>) = R16Unorm,
    (image::Luma<i16>) = R16Snorm,
    (image::Luma<u32>) = R32Uint,
    (image::Luma<i32>) = R32Sint,
    (image::Luma<f32>) = R32Float,
    (image::Luma<u64>) = R64Uint,
    (image::Luma<i64>) = R64Sint,
    (image::Luma<f64>) = R64Float,

    (image::LumaA<u8>) = Rg8Unorm,
    (image::LumaA<i8>) = Rg8Snorm,
    (image::LumaA<u16>) = Rg16Unorm,
    (image::LumaA<i16>) = Rg16Snorm,
    (image::LumaA<u32>) = Rg32Uint,
    (image::LumaA<i32>) = Rg32Sint,
    (image::LumaA<f32>) = Rg32Float,
    (image::LumaA<u64>) = Rg64Uint,
    (image::LumaA<i64>) = Rg64Sint,
    (image::LumaA<f64>) = Rg64Float,

    (image::Rgb<u8>) = Rgb8Unorm,
    (image::Rgb<i8>) = Rgb8Snorm,
    (image::Rgb<u16>) = Rgb16Unorm,
    (image::Rgb<i16>) = Rgb16Snorm,
    (image::Rgb<u32>) = Rgb32Uint,
    (image::Rgb<i32>) = Rgb32Sint,
    (image::Rgb<f32>) = Rgb32Float,
    (image::Rgb<u64>) = Rgb64Uint,
    (image::Rgb<i64>) = Rgb64Sint,
    (image::Rgb<f64>) = Rgb64Float,

    (image::Rgba<u8>) = Rgba8Unorm,
    (image::Rgba<i8>) = Rgba8Snorm,
    (image::Rgba<u16>) = Rgba16Unorm,
    (image::Rgba<i16>) = Rgba16Snorm,
    (image::Rgba<u32>) = Rgba32Uint,
    (image::Rgba<i32>) = Rgba32Sint,
    (image::Rgba<f32>) = Rgba32Float,
    (image::Rgba<u64>) = Rgba64Uint,
    (image::Rgba<i64>) = Rgba64Sint,
    (image::Rgba<f64>) = Rgba64Float,

    (image::Bgr<u8>) = Bgr8Unorm,
    (image::Bgr<i8>) = Bgr8Snorm,
    (image::Bgra<u8>) = Bgra8Unorm,
    (image::Bgra<i8>) = Bgra8Snorm,
);