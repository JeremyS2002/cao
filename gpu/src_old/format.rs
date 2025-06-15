//! TextureFormat

use ash::vk;

bitflags::bitflags! {
    /// Describes a texture format
    pub struct TextureAspects: u32 {
        /// Format can be used for color operations
        const COLOR     = 0b001;
        /// Format can be used for depth operations
        const DEPTH     = 0b010;
        /// Format can be used for stencil operations
        const STENCIL   = 0b100;
    }
}

impl Into<vk::ImageAspectFlags> for TextureAspects {
    fn into(self) -> vk::ImageAspectFlags {
        let mut res = vk::ImageAspectFlags::empty();
        if self.contains(Self::COLOR) {
            res |= vk::ImageAspectFlags::COLOR;
        }
        if self.contains(Self::DEPTH) {
            res |= vk::ImageAspectFlags::DEPTH;
        }
        if self.contains(Self::STENCIL) {
            res |= vk::ImageAspectFlags::STENCIL;
        }
        res
    }
}

/// Trait that allows for static typing of objects based on format
/// all items in the Format enum have a unit struct with the same name
/// that can be used to represent them
pub trait AsFormat: Sized {
    /// The member of the Format enum that this corresponds to
    const FORMAT: Format;
}

macro_rules! create_formats {
    (
        $($name:ident => $vk:ident => $size:expr => ($($aspect:ident,)*),)*
    ) => {
        /// Represents a format of a texture
        #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
        pub enum Format {
            $(
                #[allow(missing_docs)]
                $name,
            )*
        }

        impl Format {
            /// returns the aspects of the image format
            pub fn aspects(&self) -> TextureAspects {
                match self {
                    $(
                        Self::$name => $(TextureAspects::$aspect | )* TextureAspects::empty(),
                    )*
                }
            }

            /// returns the size in bytes of one pixel of this format
            pub fn size(&self) -> usize {
                match self {
                    $(
                        Self::$name => $size,
                    )*
                }
            }
        }

        impl Into<vk::Format> for Format {
            fn into(self) -> vk::Format {
                match self {
                    $(
                        Self::$name => vk::Format::$vk,
                    )*
                }
            }
        }

        impl From<vk::Format> for Format {
            fn from(f: vk::Format) -> Self {
                match f {
                    $(
                        vk::Format::$vk => Self::$name,
                    )*
                    _ => Self::Unknown,
                }
            }
        }

        $(
            /// equivalent to field of same name in format for static typing
            #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
            pub struct $name;

            impl AsFormat for $name {
                const FORMAT: Format = Format::$name;
            }
        )*
    };
}

/// Indicates that the T should be interpreted as Srgb
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Srgb(pub i8);

unsafe impl bytemuck::Pod for Srgb {}
unsafe impl bytemuck::Zeroable for Srgb {}

/// Indicates that the T should be interpreded as Depth
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Depth<T>(pub T);

unsafe impl<T: bytemuck::Pod + bytemuck::Zeroable> bytemuck::Pod for Depth<T> {}
unsafe impl<T: bytemuck::Pod + bytemuck::Zeroable> bytemuck::Zeroable for Depth<T> {}

/// Indicates that the pixel order is reversed

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Bgr<T>(T);

unsafe impl<T: bytemuck::Pod + bytemuck::Zeroable> bytemuck::Pod for Bgr<T> {}
unsafe impl<T: bytemuck::Pod + bytemuck::Zeroable> bytemuck::Zeroable for Bgr<T> {}

create_formats! {
    R8Unorm                  => R8_UNORM               => 1       => (COLOR,),
    R8Snorm                  => R8_SNORM               => 1       => (COLOR,),
    R16Unorm                 => R16_UNORM              => 2       => (COLOR,),
    R16Snorm                 => R16_SNORM              => 2       => (COLOR,),
    R16Float                 => R16_SFLOAT             => 2       => (COLOR,),
    R32Uint                  => R32_UINT               => 4       => (COLOR,),
    R32Sint                  => R32_SINT               => 4       => (COLOR,),
    R32Float                 => R32_SFLOAT             => 4       => (COLOR,),
    R64Uint                  => R64_UINT               => 8       => (COLOR,),
    R64Sint                  => R64_SINT               => 8       => (COLOR,),
    R64Float                 => R64_SFLOAT             => 8       => (COLOR,),

    Rg8Unorm                 => R8G8_UNORM             => 1*2     => (COLOR,),
    Rg8Snorm                 => R8G8_SNORM             => 1*2     => (COLOR,),
    Rg16Unorm                => R16G16_UNORM           => 2*2     => (COLOR,),
    Rg16Snorm                => R16G16_SNORM           => 2*2     => (COLOR,),
    Rg16Float                => R16G16_SFLOAT          => 2*2     => (COLOR,),
    Rg32Uint                 => R32G32_UINT            => 4*2     => (COLOR,),
    Rg32Sint                 => R32G32_SINT            => 4*2     => (COLOR,),
    Rg32Float                => R32G32_SFLOAT          => 4*2     => (COLOR,),
    Rg64Uint                 => R64G64_UINT            => 8*2     => (COLOR,),
    Rg64Sint                 => R64G64_SINT            => 8*2     => (COLOR,),
    Rg64Float                => R64G64_SFLOAT          => 8*2     => (COLOR,),

    Rgb8Unorm                => R8G8B8_UNORM           => 1*3     => (COLOR,),
    Rgb8Snorm                => R8G8B8_SNORM           => 1*3     => (COLOR,),
    Rgb8Srgb                 => R8G8B8_SRGB            => 1*3     => (COLOR,),
    Rgb16Unorm               => R16G16B16_UNORM        => 2*3     => (COLOR,),
    Rgb16Float               => R16G16B16_SFLOAT       => 2*3     => (COLOR,),
    Rgb16Snorm               => R16G16B16_SNORM        => 2*3     => (COLOR,),
    Rgb32Uint                => R32G32B32_UINT         => 4*3     => (COLOR,),
    Rgb32Sint                => R32G32B32_SINT         => 4*3     => (COLOR,),
    Rgb32Float               => R32G32B32_SFLOAT       => 4*3     => (COLOR,),
    Rgb64Uint                => R64G64B64_UINT         => 8*3     => (COLOR,),
    Rgb64Sint                => R64G64B64_SINT         => 8*3     => (COLOR,),
    Rgb64Float               => R64G64B64_SFLOAT       => 8*3     => (COLOR,),

    Rgba8Unorm               => R8G8B8A8_UNORM         => 1*4     => (COLOR,),
    Rgba8Snorm               => R8G8B8A8_SNORM         => 1*4     => (COLOR,),
    Rgba8Srgb                => R8G8B8A8_SRGB          => 1*4     => (COLOR,),
    Rgba16Unorm              => R16G16B16A16_UNORM     => 2*4     => (COLOR,),
    Rgba16Float              => R16G16B16A16_SFLOAT    => 2*4     => (COLOR,),
    Rgba16Snorm              => R16G16B16A16_SNORM     => 2*4     => (COLOR,),
    Rgba32Uint               => R32G32B32A32_UINT      => 4*4     => (COLOR,),
    Rgba32Sint               => R32G32B32A32_SINT      => 4*4     => (COLOR,),
    Rgba32Float              => R32G32B32A32_SFLOAT    => 4*4     => (COLOR,),
    Rgba64Uint               => R64G64B64A64_UINT      => 8*4     => (COLOR,),
    Rgba64Sint               => R64G64B64A64_SINT      => 8*4     => (COLOR,),
    Rgba64Float              => R64G64B64A64_SFLOAT    => 8*4     => (COLOR,),

    Bgr8Unorm                => B8G8R8_UNORM           => 1*3     => (COLOR,),
    Bgr8Snorm                => B8G8R8_SNORM           => 1*3     => (COLOR,),
    Bgr8Srgb                 => B8G8R8_SRGB            => 1*3     => (COLOR,),
    Bgra8Unorm               => B8G8R8A8_UNORM         => 1*4     => (COLOR,),
    Bgra8Snorm               => B8G8R8A8_SNORM         => 1*4     => (COLOR,),
    Bgra8Srgb                => B8G8R8A8_SRGB          => 1*4     => (COLOR,),

    Depth32Float             => D32_SFLOAT             => 32      => (DEPTH,),
    Depth16Unorm             => D16_UNORM              => 16      => (DEPTH,),
    Depth32FloatStencil8Uint => D32_SFLOAT_S8_UINT     => 40      => (DEPTH, STENCIL,),
    Depth24UnormStencil8Uint => D24_UNORM_S8_UINT      => 32      => (DEPTH, STENCIL,),
    Depth16UnormStencil8Uint => D16_UNORM_S8_UINT      => 24      => (DEPTH, STENCIL,),
    Stencil8Uint             => S8_UINT                => 8       => (STENCIL,),

    Unknown                  => UNDEFINED              => 0     => (COLOR,),
}
