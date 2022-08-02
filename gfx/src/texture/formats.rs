//! Formats divided by component types and their corresponding texture types
//! 
//! TODO: Statically enforce image and raw image pixel types match format component rather than panicing

use super::*;
use std::convert::TryFrom;

macro_rules! make_format {
    (
        $name:ident,
        $(
            $e:ident,
        )*
    ) => {
        /// Format enum that enforces the pixel data type of the format
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum $name {
            $(
                $e,
            )*
        }

        impl Into<gpu::Format> for $name {
            fn into(self) -> gpu::Format {
                match self {
                    $(
                        Self::$e => gpu::Format::$e,
                    )*
                }
            }
        }

        impl TryFrom<gpu::Format> for $name {
            type Error = String;

            fn try_from(value: gpu::Format) -> Result<Self, Self::Error> {
                match value {
                    $(
                        gpu::Format::$e => Ok(Self::$e),
                    )*
                    n => Err(format!("Cannot use {:?} as {}", n, stringify!($name))),
                }
            }
        }
    };
}

make_format!(
    Format,
    R8Unorm,
    R8Snorm,
    R16Unorm,
    R16Snorm,
    R16Float,
    R32Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rg32Float,
    Rgb8Unorm,
    Rgb8Snorm,
    Rgb8Srgb,
    Rgb16Unorm,
    Rgb16Float,
    Rgb16Snorm,
    Rgb32Float,
    Rgba8Unorm,
    Rgba8Snorm,
    Rgba8Srgb,
    Rgba16Unorm,
    Rgba16Float,
    Rgba16Snorm,
    Rgba32Float,
    Bgr8Unorm,
    Bgr8Snorm,
    Bgr8Srgb,
    Bgra8Unorm,
    Bgra8Snorm,
    Bgra8Srgb,
);

make_format!(DFormat, R64Float, Rg64Float, Rgb64Float,);

make_format!(
    IFormat, R32Sint, R64Sint, Rg32Sint, Rg64Sint, Rgb32Sint, Rgb64Sint, Rgba32Sint, Rgba64Sint,
);

make_format!(
    UFormat, R32Uint, R64Uint, Rg32Uint, Rg64Uint, Rgb32Uint, Rgb64Uint, Rgba32Uint, Rgba64Uint,
);

macro_rules! make_texture_1d {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTexture1D::from_dimension(
                    device,
                    D1(width),
                    usage,
                    mip_levels,
                    format.into(),
                    name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTexture1D::from_formats(
                    device,
                    width,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?
                .map(|t| Self(t)))
            }

            /// Create a new texture from a raw image
            ///
            /// will infer the gpu::Format to use
            pub fn from_raw_image<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                width: gpu::Size,
                raw_texture: &[P],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture1D::from_raw_image(
                    encoder,
                    device,
                    width,
                    raw_texture,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(encoder, device, raw_texture)
            }
        }
    };
}

macro_rules! make_texture_1d_array {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTexture1DArray::from_dimension(
                    device,
                    D1Array(width, layers),
                    usage,
                    mip_levels,
                    format.into(),
                    name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTexture1DArray::from_formats(
                    device,
                    width,
                    layers,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?
                .map(|t| Self(t)))
            }

            /// Create a new Texture from a raw image
            ///
            /// Will infer the gpu::Format to use
            pub fn from_raw_images<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                width: gpu::Size,
                raw_textures: &[&[P]],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture1DArray::from_raw_images(
                    encoder,
                    device,
                    width,
                    raw_textures,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
                array_layer: gpu::Layer,
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(encoder, device, raw_texture, array_layer)
            }
        }
    };
}

macro_rules! make_texture_2d {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                samples: gpu::Samples,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTexture2D::from_dimension(
                    device,
                    D2(width, height, samples),
                    usage,
                    mip_levels,
                    format.into(),
                    name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                samples: gpu::Samples,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTexture2D::from_formats(
                    device,
                    width,
                    height,
                    samples,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?
                .map(|t| Self(t)))
            }

            /// Create a new texture from a raw image
            ///
            /// will infer the gpu::Format to use
            pub fn from_raw_image<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
                width: gpu::Size,
                height: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture2D::from_raw_image(
                    encoder,
                    device,
                    raw_texture,
                    width,
                    height,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(encoder, device, raw_texture)
            }
        }

        #[cfg(feature = "image")]
        impl $name {
            /// Create a new texture from an image
            ///
            /// This will infer the gpu::Format from the component in the image
            /// and will use the dimensions of the image for the dimensions of the texture
            pub fn from_image<C, P>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                image: &image::ImageBuffer<P, C>,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture2D::from_image(
                    encoder, device, image, usage, mip_levels, name,
                )?))
            }

            /// Write an image to self
            ///
            /// Will panic if the dimensions don't match self
            pub fn write_image<C, P>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                image: &image::ImageBuffer<P, C>,
            ) -> Result<(), gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_image(encoder, device, image)
            }
        }
    };
}

macro_rules! make_texture_2d_array {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                samples: gpu::Samples,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTexture2DArray::new(
                    device, width, height, samples, layers, usage, mip_levels, format.into(), name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                samples: gpu::Samples,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTexture2DArray::from_formats(
                    device,
                    width,
                    height,
                    samples,
                    layers,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?
                .map(|t| Self(t)))
            }

            /// Create a new Texture from a raw image
            ///
            /// Will infer the gpu::Format to use
            pub fn from_raw_images<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_textures: &[&[P]],
                width: gpu::Size,
                height: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture2DArray::from_raw_images(
                    encoder,
                    device,
                    raw_textures,
                    width,
                    height,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
                array_layer: gpu::Layer,
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(encoder, device, raw_texture, array_layer)
            }
        }

        #[cfg(feature = "image")]
        impl $name {
            /// Create a new texture from images
            ///
            /// Will infer the gpu::Format to use and use the width and height from the images
            /// All the images must have the same dimensions
            pub fn from_images<C, P>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                images: &[&image::ImageBuffer<P, C>],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTexture2DArray::from_images(
                    encoder,
                    device,
                    images,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write an image to self
            ///
            /// Will panic if the dimensions don't match self
            /// 
            /// At the moment the texture must have been created with [`gpu::Samples::S1`]
            /// If it was created with more then you must create a staging image and blit between them
            /// This limit should be lifted as soon as I get around to it
            pub fn write_image<C, P>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                image: &image::ImageBuffer<P, C>,
                array_layer: gpu::Layer,
            ) -> Result<(), gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_image(encoder, device, image, array_layer)
            }
        }
    };
}

macro_rules! make_texture_cube {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTextureCube::new(
                    device,
                    width,
                    height,
                    usage,
                    mip_levels,
                    format.into(),
                    name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTextureCube::from_formats(
                    device,
                    width,
                    height,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?
                .map(|t| Self(t)))
            }

            /// Create a new Texture from a raw image
            ///
            /// Will infer the gpu::Format to use
            pub fn from_raw_images<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                raw_textures: &[&[P]; 6],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTextureCube::from_raw_images(encoder, device, width, height, raw_textures, usage, mip_levels, name)?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
                face: CubeFace,
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(encoder, device, raw_texture, face)
            }
        }

        #[cfg(feature = "image")]
        impl $name {
            /// Create a new texture from an image
            ///
            /// This will infer the gpu::Format from the component in the image
            /// and will use the dimensions of the image for the dimensions of the texture
            pub fn from_images<C, P>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                images: &[&image::ImageBuffer<P, C>; 6],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTextureCube::from_images(
                    encoder,
                    device,
                    images,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write an image to self
            ///
            /// Will panic if the dimensions don't match self
            pub fn write_image<C, P>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                image: &image::ImageBuffer<P, C>,
                face: CubeFace,
            ) -> Result<(), gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_image(
                    encoder,
                    device,
                    image,
                    face,
                )
            }
        }
    };
}

macro_rules! make_texture_cube_array {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTextureCubeArray::new(
                    device,
                    width,
                    height,
                    layers,
                    usage,
                    mip_levels,
                    format.into(),
                    name,
                )?))
            }

            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                layers: gpu::Layer,
                usage: gpu::TextureUsage,
                mip_levels: u32,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTextureCubeArray::from_formats(
                    device,
                    width,
                    height,
                    layers,
                    usage,
                    mip_levels,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?.map(|t| Self(t)))
            }

            /// Create a new Texture from a raw image
            ///
            /// Will infer the gpu::Format to use
            pub fn from_raw_images<P: FormatData + bytemuck::Pod>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                raw_textures: &[&[&[P]; 6]],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTextureCubeArray::from_raw_images(
                    encoder,
                    device,
                    width,
                    height,
                    raw_textures,
                    usage,
                    mip_levels,
                    name,
                )?))
            }

            /// Write a raw texture to self
            ///
            /// Will panic if the texture isn't the right dimensions
            pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                raw_texture: &[P],
                array_layer: gpu::Layer,
                face: CubeFace,
            ) -> Result<(), gpu::Error> {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_raw_image(
                    encoder,
                    device,
                    raw_texture,
                    array_layer,
                    face,
                )
            }
        }

        #[cfg(feature = "image")]
        impl $name {
            /// Create a new texture from an image
            ///
            /// This will infer the gpu::Format from the component in the image
            /// and will use the dimensions of the image for the dimensions of the texture
            pub fn from_images<C, P>(
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                images: &[&[&image::ImageBuffer<P, C>; 6]],
                usage: gpu::TextureUsage,
                mip_levels: u32,
                name: Option<String>,
            ) -> Result<Self, gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                Ok(Self(GTextureCubeArray::from_images(
                    encoder,
                    device,
                    images,
                    usage,
                    mip_levels,
                    name,
                )?))
            }
        
            /// Write an image to self
            ///
            /// Will panic if the dimensions don't match self
            pub fn write_image<C, P>(
                &self,
                encoder: &mut crate::CommandEncoder<'_>,
                device: &gpu::Device,
                image: &image::ImageBuffer<P, C>,
                array_layer: gpu::Layer,
                face: CubeFace,
            ) -> Result<(), gpu::Error>
            where
                P: FormatData + image::Pixel + 'static,
                P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
                C: std::ops::Deref<Target = [P::Subpixel]>,
            {
                $format::try_from(P::FORMAT).unwrap();
                self.0.write_image(
                    encoder,
                    device,
                    image,
                    array_layer,
                    face,
                )
            }
        }
    };
}

macro_rules! make_texture_3d {
    ($name:ident, $format:ident) => {
        impl $name {
            /// Create a new Texture from dimensions
            pub fn new(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                depth: gpu::Size,
                usage: gpu::TextureUsage,
                format: $format,
                name: Option<String>,
            ) -> Result<Self, gpu::Error> {
                Ok(Self(GTexture3D::new(
                    device,
                    width,
                    height,
                    depth,
                    usage,
                    format.into(),
                    name,
                )?))
            }
        
            /// Create a new Texture from dimensions and a list of possible formats
            /// Returns Ok(None) if none of the possible formats are valid
            pub fn from_formats(
                device: &gpu::Device,
                width: gpu::Size,
                height: gpu::Size,
                depth: gpu::Size,
                usage: gpu::TextureUsage,
                formats: impl IntoIterator<Item = $format>,
                name: Option<String>,
            ) -> Result<Option<Self>, gpu::Error> {
                Ok(GTexture3D::from_formats(
                    device,
                    width,
                    height,
                    depth,
                    usage,
                    formats.into_iter().map(|f| f.into()),
                    name,
                )?.map(|t| Self(t)))
            }
        }
    };
}

macro_rules! impl_textures {
    (
        $name:ident, $format:ident,
        [$(
            $alias:ident, $dim:ident, $mac:ident,
        )*]
    ) => {
        pub struct $name<D: AsDimension>(pub GTexture<D>);

        impl<D: AsDimension> std::ops::Deref for $name<D> {
            type Target = GTexture<D>;

            fn deref(&self) -> &GTexture<D> {
                &self.0
            }
        }

        $(
            pub type $alias = $name<$dim>;

            $mac!($alias, $format);
        )*
    };
}

impl_textures!(
    Texture, Format,
    [
        Texture1D, D1, make_texture_1d,
        Texture1DArray, D1Array, make_texture_1d_array,
        Texture2D, D2, make_texture_2d,
        Texture2DArray, D2Array, make_texture_2d_array,
        TextureCube, Cube, make_texture_cube,
        TextureCubeArray, CubeArray, make_texture_cube_array,
        Texture3D, D3, make_texture_3d,
    ]
);

impl_textures!(
    DTexture, DFormat,
    [
        DTexture1D, D1, make_texture_1d,
        DTexture1DArray, D1Array, make_texture_1d_array,
        DTexture2D, D2, make_texture_2d,
        DTexture2DArray, D2Array, make_texture_2d_array,
        DTextureCube, Cube, make_texture_cube,
        DTextureCubeArray, CubeArray, make_texture_cube_array,
        DTexture3D, D3, make_texture_3d,
    ]
);

impl_textures!(
    ITexture, IFormat,
    [
        ITexture1D, D1, make_texture_1d,
        ITexture1DArray, D1Array, make_texture_1d_array,
        ITexture2D, D2, make_texture_2d,
        ITexture2DArray, D2Array, make_texture_2d_array,
        ITextureCube, Cube, make_texture_cube,
        ITextureCubeArray, CubeArray, make_texture_cube_array,
        ITexture3D, D3, make_texture_3d,
    ]
);

impl_textures!(
    UTexture, UFormat,
    [
        UTexture1D, D1, make_texture_1d,
        UTexture1DArray, D1Array, make_texture_1d_array,
        UTexture2D, D2, make_texture_2d,
        UTexture2DArray, D2Array, make_texture_2d_array,
        UTextureCube, Cube, make_texture_cube,
        UTextureCubeArray, CubeArray, make_texture_cube_array,
        UTexture3D, D3, make_texture_3d,
    ]
);
