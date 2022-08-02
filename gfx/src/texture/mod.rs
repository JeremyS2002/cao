//! Utilities for manipulating textures
//! 
//! [`GTexture`]
//! Wraps a [`gpu::Texture`] and [`gpu::TextureView`] providing utilities for generating mipmaps, 
//! slicing more intuitivly and writing data, as well as statically typed texture dimension
//! 
//! The specific type aliases for each dimensions provide more intuitive methods for creating texture from 
//! images from the image crate as well as more robust methods for dealing with specific format dimension 
//! combinations not being available on the current system.
//! 
//! The _Texture_ types statically enforce both the dimension and the component type, for example:
//! [`Texture2D`] enforces a 2 dimensional texture with floating point components
//! [`DTexture1D`] enforces a 1 dimensional texture with double precision components
//! [`ITextureCube`] enforces a cube dimensional texture with signed integer components
//! [`UTexture2DArray`] enforces a 2d array texture with unsigned integer components
//! Be aware that not all combinations of components and dimension are supported on all systems
//! 
//! TODO:
//! Robust methods for loading textures from dynamic images. Check what format the image is in and then 
//! find a format that works (If necissary can use image methods to change the pixel type of the image)
//!

pub mod formats;
pub mod traits;

pub use formats::*;
pub use traits::*;

/// Multiple textures formats can be suited to the same job.
/// 
/// For example when rendering to a buffer it doesn't really matter if the buffer
/// is Rgb16Float Rgb32Float or Rgb64Float as long as it has at least 3 components and has floating point value pixels
/// this is important as different systems have support for different formats so it is good
/// to be able to choose between those options at runtime
/// 
/// This function returns alternative formats that can replace the format provided
/// 
/// example usage:
/// ```
/// fn create_framebuffer(device: &gpu::Device, width: u32, height: u32) -> gfx::Texture2D {
///     gfx::Texture2D::from_formats(
///         &device,
///         width,
///         height,
///         gpu::TextureUsage::COLOR_OUTPUT,
///         1,
///         alt_formats(gpu::Format::Rgb32Float),    
///         None,
///     ).unwrap()
/// }
/// ```
/// 
pub fn alt_formats(format: gpu::Format) -> impl Iterator<Item=gpu::Format> {
    use gpu::Format::*;
    match format {
        R8Unorm => vec![R8Unorm, Rg8Unorm, Rgb8Unorm, Rgba8Unorm].into_iter(),
        R8Snorm => vec![R8Snorm, Rg8Snorm, Rgb8Snorm, Rgba8Snorm].into_iter(),
        R16Unorm => vec![R16Unorm, Rg16Unorm, Rgb16Unorm, Rgba16Unorm].into_iter(),
        R16Snorm => vec![R16Snorm, Rg16Snorm, Rgb16Snorm, Rgba16Snorm].into_iter(),
        R16Float => vec![R16Float, Rg16Float, Rgb16Float, Rgba16Float, R32Float, Rg32Float, Rgb32Float, Rgba32Float, R64Float, Rg64Float, Rgb64Float, Rgba64Float].into_iter(),
        R32Uint => vec![R32Uint, Rg32Uint, Rgb32Uint, Rgba32Uint].into_iter(),
        R32Sint => vec![R32Sint, Rg32Sint, Rgb32Sint, Rgba32Sint].into_iter(),
        R32Float => vec![R32Float, Rg32Float, Rgb32Float, Rgba32Float, R64Float, Rg64Float, Rgb64Float, Rgba64Float, R16Float, Rg16Float, Rgb16Float, Rgba16Float].into_iter(),
        R64Uint => vec![R64Uint, Rg64Uint, Rgb64Uint, Rgba64Uint].into_iter(),
        R64Sint => vec![R64Sint, Rg64Sint, Rgb64Sint, Rgba64Sint].into_iter(),
        R64Float => vec![R64Float, Rg64Float, Rgb64Float, Rgba64Float, R32Float, Rg32Float, Rgb32Float, Rgba32Float, R16Float, Rg16Float, Rgb16Float, Rgba16Float].into_iter(),
        Rg8Unorm => vec![Rg8Unorm, Rgb8Unorm, Rgba8Unorm].into_iter(),
        Rg8Snorm => vec![Rg8Snorm, Rgb8Snorm, Rgba8Snorm].into_iter(),
        Rg16Unorm => vec![Rg16Unorm, Rgb16Unorm, Rgba16Unorm].into_iter(),
        Rg16Snorm => vec![Rg16Snorm, Rgb16Snorm, Rgba16Snorm].into_iter(),
        Rg16Float => vec![Rg16Float, Rgb16Float, Rgba16Float, Rg32Float, Rgb32Float, Rg64Float, Rgba32Float, Rgb64Float, Rgba64Float].into_iter(),
        Rg32Uint => vec![Rg32Uint, Rgb32Uint, Rgba32Uint].into_iter(),
        Rg32Sint => vec![Rg32Sint, Rgb32Sint, Rgba32Sint].into_iter(),
        Rg32Float => vec![Rg32Float, Rgb32Float, Rgba32Float, Rg64Float, Rgb64Float, Rgba64Float, Rg16Float, Rgb16Float, Rgba16Float].into_iter(),
        Rg64Uint => vec![Rg64Uint, Rgb64Uint, Rgba64Uint].into_iter(),
        Rg64Sint => vec![Rg64Sint, Rgb64Sint, Rgba64Sint].into_iter(),
        Rg64Float => vec![Rg64Float, Rgb64Float, Rgba64Float, Rg32Float, Rgb32Float, Rgba32Float, Rg16Float, Rgb16Float, Rgba16Float].into_iter(),
        Rgb8Unorm => vec![Rgb8Unorm, Rgba8Unorm].into_iter(),
        Rgb8Snorm => vec![Rgb8Snorm, Rgba8Snorm].into_iter(),
        Rgb8Srgb => vec![Rgb8Srgb, Rgba8Srgb].into_iter(),
        Rgb16Unorm => vec![Rgb16Unorm, Rgba16Unorm].into_iter(),
        Rgb16Float => vec![Rgb16Float, Rgba16Float, Rgb32Float, Rgba32Float, Rgb64Float, Rgba64Float].into_iter(),
        Rgb16Snorm => vec![Rgb16Snorm, Rgba16Snorm].into_iter(),
        Rgb32Uint => vec![Rgb32Uint, Rgba32Uint].into_iter(),
        Rgb32Sint => vec![Rgb32Sint, Rgba32Sint].into_iter(),
        Rgb32Float => vec![Rgb32Float, Rgba32Float, Rgb64Float, Rgba64Float, Rgb16Float, Rgba16Float].into_iter(),
        Rgb64Uint => vec![Rgb64Uint, Rgba64Uint].into_iter(),
        Rgb64Sint => vec![Rgb64Sint, Rgba64Sint].into_iter(),
        Rgb64Float => vec![Rgb64Float, Rgba64Float, Rgb32Float, Rgba32Float, Rgb16Float, Rgba16Float].into_iter(),
        Rgba8Unorm => vec![Rgba8Unorm].into_iter(),
        Rgba8Snorm => vec![Rgba8Snorm].into_iter(),
        Rgba8Srgb => vec![Rgba8Srgb].into_iter(),
        Rgba16Unorm => vec![Rgba16Unorm].into_iter(),
        Rgba16Float => vec![Rgba16Float, Rgba32Float, Rgba64Float].into_iter(),
        Rgba16Snorm => vec![Rgba16Snorm].into_iter(),
        Rgba32Uint => vec![Rgba32Uint].into_iter(),
        Rgba32Sint => vec![Rgba32Sint].into_iter(),
        Rgba32Float => vec![Rgba32Float, Rgba64Float, Rgba16Float].into_iter(),
        Rgba64Uint => vec![Rgba64Uint].into_iter(),
        Rgba64Sint => vec![Rgba64Sint].into_iter(),
        Rgba64Float => vec![Rgba64Float, Rgba32Float, Rgba16Float].into_iter(),
        Bgr8Unorm => vec![Bgr8Unorm].into_iter(),
        Bgr8Snorm => vec![Bgr8Snorm].into_iter(),
        Bgr8Srgb => vec![Bgr8Srgb].into_iter(),
        Bgra8Unorm => vec![Bgra8Unorm].into_iter(),
        Bgra8Snorm => vec![Bgra8Snorm].into_iter(),
        Bgra8Srgb => vec![Bgra8Srgb].into_iter(),
        Depth32Float => vec![Depth32Float].into_iter(),
        Depth16Unorm => vec![Depth16Unorm].into_iter(),
        Depth32FloatStencil8Uint => vec![Depth32FloatStencil8Uint].into_iter(),
        Depth24UnormStencil8Uint => vec![Depth24UnormStencil8Uint, Depth16UnormStencil8Uint].into_iter(),
        Depth16UnormStencil8Uint => vec![Depth16UnormStencil8Uint, Depth24UnormStencil8Uint].into_iter(),
        Stencil8Uint => vec![Stencil8Uint].into_iter(),
        Unknown => vec![Unknown].into_iter(),
    }
}

/// Iterate over the formats and see if it is compaitble with the dimension and
pub fn choose_format<'a>(
    device: &gpu::Device,
    options: impl IntoIterator<Item = gpu::Format>,
    dimension: gpu::TextureDimension,
    usage: gpu::TextureUsage,
    mip_levels: u32,
) -> Option<gpu::Format> {
    let samples = dimension.samples();
    for format in options {
        if let Ok(p) = device.texture_properties(format, dimension.kind(), usage) {
            let extent: gpu::Extent3D = dimension.into();
            if extent.width > p.max_extent.width
                || extent.height > p.max_extent.height
                || extent.depth > p.max_extent.depth
            {
                continue;
            }
            if mip_levels > p.max_mip_levels {
                continue;
            }
            if dimension.layers() > p.max_array_layers {
                continue;
            }
            if !p.sample_counts.contains(samples.flags()) {
                continue;
            }
            return Some(format);
        }
    }
    None
}

/// Calculates the maximum number of mip levels for a texture of dimensions supplied
pub fn max_mip_levels<D: AsDimension>(dimension: D) -> u32 {
    let extent: gpu::Extent3D = dimension.as_dimension().into();
    let max = extent.width.max(extent.height.max(extent.depth));
    (max as f32).log2().floor() as u32 + 1
}

/// Represent a face in a cube texture
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CubeFace {
    #[allow(missing_docs)]
    PosX = 0,
    #[allow(missing_docs)]
    NegX = 1,
    #[allow(missing_docs)]
    PosY = 2,
    #[allow(missing_docs)]
    NegY = 3,
    #[allow(missing_docs)]
    PosZ = 4,
    #[allow(missing_docs)]
    NegZ = 5,
}

impl CubeFace {
    /// Returns an Iterator that iterates over the faces in gpu order
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            CubeFace::PosX,
            CubeFace::NegX,
            CubeFace::PosY,
            CubeFace::NegY,
            CubeFace::PosZ,
            CubeFace::NegZ,
        ]
        .into_iter()
    }
}

/// A Staticly typed texture That provides assaurances when loading from files
/// and Simple methods to do so,
/// Also allows access to the base texture and view
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GTexture<D: AsDimension> {
    /// the base texture dimension: D::as_dimension()
    pub texture: gpu::Texture,
    /// the base view into the whole texture
    pub view: gpu::TextureView,
    /// the dimension of the texture
    pub dimension: D,
}

impl<D: AsDimension> std::ops::Deref for GTexture<D> {
    type Target = gpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl<D: AsDimension> GTexture<D> {
    /// create a new Gtexture from dimension and other infomation
    pub fn from_dimension(
        device: &gpu::Device,
        dimension: D,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        let texture = device.create_texture(&gpu::TextureDesc {
            format,
            dimension: dimension.as_dimension(),
            mip_levels: std::num::NonZeroU32::new(mip_levels).unwrap(),
            usage,
            memory: gpu::MemoryType::Device,
            layout: gpu::TextureLayout::General,
            name,
        })?;
        let view = texture.create_default_view()?;
        Ok(Self {
            texture,
            view,
            dimension,
        })
    }

    /// Write the data to the texture
    /// Internally this will fill a staging buffer with the data and then copy that to the first
    /// mip level of self, if there are multiple mip levels then texture blits will be used to fill the mip chain
    pub fn write_data_ref<'a>(
        &'a self,
        encoder: &mut crate::CommandEncoder<'a>,
        device: &gpu::Device,
        data: &[u8],
        offset: gpu::Offset3D,
        extent: gpu::Extent3D,
        base_array_layer: u32,
        array_layers: u32,
    ) -> Result<(), gpu::Error> {
        let staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: data.len() as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;
        staging_buffer.slice_ref(..).write(data)?;
        encoder.copy_buffer_to_texture(
            staging_buffer.into_slice(..),
            self.texture.slice_ref(&gpu::TextureSliceDesc {
                offset,
                extent,
                base_array_layer,
                array_layers,
                base_mip_level: 0,
                mip_levels: 1,
            }),
        );
        Ok(())
    }

    /// Generate mipmaps from the base mip level
    pub fn gen_mipmaps_ref<'a>(&'a self, encoder: &mut crate::CommandEncoder<'a>) {
        for level in 1..self.texture.mip_levels() {
            encoder.blit_textures(
                self.mip_slice_ref(level - 1),
                self.mip_slice_ref(level),
                gpu::FilterMode::Linear,
            );
        }
    }

    /// Write the data to the texture
    /// Internally this will fill a staging buffer with the data and then copy that to the first
    /// mip level of self, if there are multiple mip levels then texture blits will be used to fill the mip chain
    pub fn write_data_owned(
        &self,
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        data: &[u8],
        offset: gpu::Offset3D,
        extent: gpu::Extent3D,
        base_array_layer: u32,
        array_layers: u32,
    ) -> Result<(), gpu::Error> {
        let staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: data.len() as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;
        staging_buffer.slice_ref(..).write(data)?;
        encoder.copy_buffer_to_texture(
            staging_buffer.into_slice(..),
            self.texture.slice_owned(&gpu::TextureSliceDesc {
                offset,
                extent,
                base_array_layer,
                array_layers,
                base_mip_level: 0,
                mip_levels: 1,
            }),
        );
        self.gen_mipmaps_owned(encoder);
        Ok(())
    }

    /// Generate mipmaps from the base mip level
    pub fn gen_mipmaps_owned(&self, encoder: &mut crate::CommandEncoder<'_>) {
        for level in 1..self.texture.mip_levels() {
            encoder.blit_textures(
                self.mip_slice_owned(level - 1),
                self.mip_slice_owned(level),
                gpu::FilterMode::Linear,
            );
        }
    }

    /// Slice the texture by reference containg only the array layer and mip level specified
    /// Note that depending on how the texture was created this won't always produce a valid slice
    pub fn layer_mip_slice_ref<'a>(&'a self, array: u32, mip: u32) -> gpu::TextureSlice<'a> {
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: array,
            array_layers: 1,
            base_mip_level: mip,
            mip_levels: 1,
        })
    }

    /// Slice the texture by cloning containg only the array layer and mip level specified
    /// Note that depending on how the texture was created this won't always produce a valid slice
    pub fn layer_mip_slice_owned<'a>(&self, array: u32, mip: u32) -> gpu::TextureSlice<'a> {
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: array,
            array_layers: 1,
            base_mip_level: mip,
            mip_levels: 1,
        })
    }

    /// Slice the texture by reference containing the whole texture at the first mip level
    /// Note that depending on the dimension of the texture this won't always produce a valid slice
    pub fn layer_slice_ref<'a>(&'a self, layer: u32) -> gpu::TextureSlice<'a> {
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: 0,
            array_layers: layer,
            base_mip_level: 0,
            mip_levels: 1,
        })
    }

    /// Slice the texture by cloning containing the whole texture at the first mip level
    /// Note that depending on the dimension of the texture this won't always produce a valid slice
    pub fn layer_slice_owned(&self, layer: u32) -> gpu::TextureSlice<'_> {
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: 0,
            array_layers: layer,
            base_mip_level: 0,
            mip_levels: 1,
        })
    }

    /// Slice the texture by reference at a specific mip level
    /// Note that depending on the how the texture was created this won't always produce a valid slice
    pub fn mip_slice_ref<'a>(&'a self, level: u32) -> gpu::TextureSlice<'a> {
        let mut extent: gpu::Extent3D = self.dimension().into();
        extent.width /= 2u32.pow(level);
        extent.height /= 2u32.pow(level);
        // TODO fix for 3d textures
        //extent.depth /= 2u32.pow(level);
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent,
            base_array_layer: 0,
            array_layers: self.dimension().layers(),
            base_mip_level: level,
            mip_levels: 1,
        })
    }

    /// Slice the texture by cloning at a specific mip level
    /// Note that depending on the how the texture was created this won't always produce a valid slice
    pub fn mip_slice_owned<'a>(&self, level: u32) -> gpu::TextureSlice<'a> {
        let mut extent: gpu::Extent3D = self.dimension().into();
        extent.width /= 2u32.pow(level);
        extent.height /= 2u32.pow(level);
        //extent.depth /= 2u32.pow(level);
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent,
            base_array_layer: 0,
            array_layers: self.dimension().layers(),
            base_mip_level: level,
            mip_levels: 1,
        })
    }

    /// Slice the texture by reference containing the whole texture at the first mip level
    pub fn whole_slice_ref<'a>(&'a self) -> gpu::TextureSlice<'a> {
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: 0,
            array_layers: self.dimension().layers(),
            base_mip_level: 0,
            mip_levels: self.texture.mip_levels(),
        })
    }

    /// Slice the texture by cloning containing the whole texture at the first mip level
    pub fn whole_slice_owned<'a>(&self) -> gpu::TextureSlice<'a> {
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: 0,
            array_layers: self.dimension().layers(),
            base_mip_level: 0,
            mip_levels: self.texture.mip_levels(),
        })
    }
}

/// A Statically typed 1d texture
pub type GTexture1D = GTexture<D1>;

/// A Statically typed 1d array texture
pub type GTexture1DArray = GTexture<D1Array>;

/// A Statically typed 2d texture
pub type GTexture2D = GTexture<D2>;

/// A Statically typed 2d array texture
pub type GTexture2DArray = GTexture<D2Array>;

/// A Statically typed cube texture
pub type GTextureCube = GTexture<Cube>;

/// A Statically typed cube array texture
pub type GTextureCubeArray = GTexture<CubeArray>;

/// A Statically typed cube multisampled texture
// pub type CubeMsTexture = GTexture<CubeMs>;

/// A Statically typed cube array multisampled texture
// pub type CubeArrayMsTexture = GTexture<CubeArrayMs>;

/// A Statically typed 3d texture
pub type GTexture3D = GTexture<D3>;

impl GTexture1D {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(device, D1(width), usage, mip_levels, format, name)
    }

    /// Create a new Texture from dimensions and a list of possible formats
    /// Returns Ok(None) if none of the possible formats are valid
    pub fn from_formats(
        device: &gpu::Device,
        width: gpu::Size,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::D1(width),
            usage,
            mip_levels,
        ) {
            Self::new(device, width, usage, mip_levels, format, name).map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::new(
            device,
            width,
            usage | gpu::TextureUsage::COPY_DST,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        t.write_raw_image(encoder, device, raw_texture)?;

        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D { x: 0, y: 0, z: 0 },
            gpu::Extent3D {
                width: self.dimension.0,
                height: 1,
                depth: 1,
            },
            0,
            1,
        )?;

        Ok(())
    }
}

impl GTexture1DArray {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        layers: gpu::Layer,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(
            device,
            D1Array(width, layers),
            usage,
            mip_levels,
            format,
            name,
        )
    }

    /// Create a new Texture from dimensions and a list of possible formats
    /// Returns Ok(None) if none of the possible formats are valid
    pub fn from_formats(
        device: &gpu::Device,
        width: gpu::Size,
        layers: gpu::Layer,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::D1Array(width, layers),
            usage,
            mip_levels,
        ) {
            Self::new(device, width, layers, usage, mip_levels, format, name).map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::new(
            device,
            width,
            raw_textures.len() as _,
            usage | gpu::TextureUsage::COPY_DST,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for (i, &raw_texture) in raw_textures.iter().enumerate() {
            t.write_raw_image(encoder, device, raw_texture, i as _)?;
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D {
                x: 0,
                y: 0,
                z: array_layer as _,
            },
            gpu::Extent3D {
                width: self.dimension.0,
                height: 1,
                depth: 1,
            },
            array_layer,
            1,
        )
    }
}

impl GTexture2D {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        samples: gpu::Samples,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(device, D2(width, height, samples), usage, mip_levels, format, name)
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
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::D2(width, height, samples),
            usage,
            mip_levels,
        ) {
            Self::new(device, width, height, samples, usage, mip_levels, format, name).map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::from_dimension(
            device,
            D2(width, height, gpu::Samples::S1),
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        t.write_raw_image(encoder, device, raw_texture)?;
        Ok(t)
    }

    /// Write a raw texture to self
    ///
    /// Will panic if the texture isn't the right dimensions
    /// 
    /// At the moment the texture must have been created with [`gpu::Samples::S1`]
    /// If it was created with more then you must create a staging image and blit between them
    /// This limit should be lifted as soon as I get around to it
    pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
        &self,
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        raw_texture: &[P],
    ) -> Result<(), gpu::Error> {
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            0,
            1,
        )
    }
}

#[cfg(feature = "image")]
impl GTexture2D {
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
        let (width, height) = image.dimensions();
        let t = Self::from_dimension(
            device,
            D2(width, height, gpu::Samples::S1),
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        t.write_image(encoder, device, image)?;
        Ok(t)
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
    ) -> Result<(), gpu::Error>
    where
        P: FormatData + image::Pixel + 'static,
        P::Subpixel: 'static + bytemuck::Pod + bytemuck::Zeroable,
        C: std::ops::Deref<Target = [P::Subpixel]>,
    {
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(image),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            0,
            1,
        )
    }
}

impl GTexture2DArray {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        samples: gpu::Samples,
        layers: gpu::Layer,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(
            device,
            D2Array(width, height, samples, layers),
            usage,
            mip_levels,
            format,
            name,
        )
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
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::D2Array(width, height, samples, layers),
            usage,
            mip_levels,
        ) {
            Self::new(
                device, width, height, samples, layers, usage, mip_levels, format, name,
            )
            .map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::from_dimension(
            device,
            D2Array(width, height, gpu::Samples::S1, raw_textures.len() as _),
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for (i, texture) in raw_textures.iter().enumerate() {
            t.write_raw_image(encoder, device, *texture, i as _)?;
        }
        Ok(t)
    }

    /// Write a raw texture to self
    ///
    /// Will panic if the texture isn't the right dimensions
    /// 
    /// At the moment the texture must have been created with [`gpu::Samples::S1`]
    /// If it was created with more then you must create a staging image and blit between them
    /// This limit should be lifted as soon as I get around to it
    pub fn write_raw_image<P: FormatData + bytemuck::Pod>(
        &self,
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        raw_texture: &[P],
        array_layer: gpu::Layer,
    ) -> Result<(), gpu::Error> {
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            array_layer,
            1,
        )
    }
}

#[cfg(feature = "image")]
impl GTexture2DArray {
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
        let (width, height) = images[0].dimensions();
        let t = Self::from_dimension(
            device,
            D2Array(width, height, gpu::Samples::S1, images.len() as _),
            usage | gpu::TextureUsage::COPY_DST,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for (i, texture) in images.iter().enumerate() {
            t.write_image(encoder, device, *texture, i as _)?;
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(image),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            array_layer,
            1,
        )
    }
}

impl GTextureCube {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(device, Cube(width, height), usage, mip_levels, format, name)
    }

    /// Create a new Texture from dimensions and a list of possible formats
    /// Returns Ok(None) if none of the possible formats are valid
    pub fn from_formats(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::Cube(width, height),
            usage,
            mip_levels,
        ) {
            Self::new(device, width, height, usage, mip_levels, format, name).map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::new(
            device,
            width,
            height,
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for face in CubeFace::iter() {
            t.write_raw_image(encoder, device, raw_textures[face as usize], face)?;
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            face as _,
            1,
        )
    }

    /// Slice the texture based on a face by reference
    pub fn face_slice_ref<'a>(&'a self, face: CubeFace) -> gpu::TextureSlice<'a> {
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: face as _,
            array_layers: 1,
            base_mip_level: 0,
            mip_levels: self.mip_levels(),
        })
    }

    /// Slice the texture based on a face by reference
    pub fn face_slice_owned<'a>(&self, face: CubeFace) -> gpu::TextureSlice<'a> {
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: face as _,
            array_layers: 1,
            base_mip_level: 0,
            mip_levels: self.mip_levels(),
        })
    }

    /// Slice the texture based on a face and mip level by reference
    pub fn face_mip_slice_ref<'a>(&'a self, face: CubeFace, mip: u32) -> gpu::TextureSlice<'a> {
        self.texture.slice_ref(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: face as _,
            array_layers: 1,
            base_mip_level: mip,
            mip_levels: 1,
        })
    }

    /// Slice the texture based on a face and mip level by reference
    pub fn face_mip_slice_owned<'a>(&self, face: CubeFace, mip: u32) -> gpu::TextureSlice<'a> {
        self.texture.slice_owned(&gpu::TextureSliceDesc {
            offset: gpu::Offset3D::ZERO,
            extent: self.dimension().into(),
            base_array_layer: face as _,
            array_layers: 1,
            base_mip_level: mip,
            mip_levels: 1,
        })
    }

    /// Create a view into the texture at the specific face
    pub fn face_view(&self, face: CubeFace) -> Result<gpu::TextureView, gpu::Error> {
        let w = self.dimension.0;
        let h = self.dimension.1;
        self.create_view(&gpu::TextureViewDesc {
            name: None,
            dimension: gpu::TextureDimension::D2(w, h, gpu::Samples::S1),
            base_mip_level: 0,
            mip_levels: self.mip_levels(),
            base_array_layer: face as _,
            format_change: None,
        })
    }

    /// Create a view into the texture at the specific face and mip level
    pub fn face_mip_view(&self, face: CubeFace, mip: u32) -> Result<gpu::TextureView, gpu::Error> {
        let w = self.dimension.0;
        let h = self.dimension.1;
        self.create_view(&gpu::TextureViewDesc {
            name: None,
            dimension: gpu::TextureDimension::D2(w, h, gpu::Samples::S1),
            base_mip_level: mip,
            mip_levels: mip,
            base_array_layer: face as _,
            format_change: None,
        })
    }
}

#[cfg(feature = "image")]
impl GTextureCube {
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
        let (width, height) = images[0].dimensions();
        let t = Self::new(
            device,
            width,
            height,
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for face in CubeFace::iter() {
            t.write_image(encoder, device, images[face as usize], face)?;
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(image),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            face as _,
            1,
        )
    }
}

impl GTextureCubeArray {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        layers: gpu::Layer,
        usage: gpu::TextureUsage,
        mip_levels: u32,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(
            device,
            CubeArray(width, height, layers),
            usage,
            mip_levels,
            format,
            name,
        )
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
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::CubeArray(width, height, layers),
            usage,
            mip_levels,
        ) {
            Self::new(
                device, width, height, layers, usage, mip_levels, format, name,
            )
            .map(|t| Some(t))
        } else {
            Ok(None)
        }
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
        let t = Self::new(
            device,
            width,
            height,
            raw_textures.len() as _,
            usage | gpu::TextureUsage::COPY_DST,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for (i, &cube) in raw_textures.iter().enumerate() {
            for face in CubeFace::iter() {
                t.write_raw_image(encoder, device, cube[face as usize], i as _, face)?;
            }
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(raw_texture),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            6 * array_layer + face as u32,
            1,
        )
    }
}

#[cfg(feature = "image")]
impl GTextureCubeArray {
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
        let (width, height) = images[0][0].dimensions();
        let t = Self::new(
            device,
            width,
            height,
            images.len() as _,
            usage | gpu::TextureUsage::COPY_DST | gpu::TextureUsage::COPY_SRC,
            mip_levels,
            P::FORMAT,
            name,
        )?;
        for (i, &cube) in images.iter().enumerate() {
            for face in CubeFace::iter() {
                t.write_image(encoder, device, cube[face as usize], i as _, face)?;
            }
        }
        Ok(t)
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
        self.write_data_owned(
            encoder,
            device,
            bytemuck::cast_slice(image),
            gpu::Offset3D::ZERO,
            gpu::Extent3D {
                width: self.dimension.0,
                height: self.dimension.1,
                depth: 1,
            },
            6 * array_layer + face as u32,
            1,
        )
    }
}

impl GTexture3D {
    /// Create a new Texture from dimensions
    pub fn new(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        depth: gpu::Size,
        usage: gpu::TextureUsage,
        format: gpu::Format,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_dimension(device, D3(width, height, depth), usage, 1, format, name)
    }

    /// Create a new Texture from dimensions and a list of possible formats
    /// Returns Ok(None) if none of the possible formats are valid
    pub fn from_formats(
        device: &gpu::Device,
        width: gpu::Size,
        height: gpu::Size,
        depth: gpu::Size,
        usage: gpu::TextureUsage,
        formats: impl IntoIterator<Item = gpu::Format>,
        name: Option<String>,
    ) -> Result<Option<Self>, gpu::Error> {
        if let Some(format) = choose_format(
            device,
            formats,
            gpu::TextureDimension::D3(width, height, depth),
            usage,
            1,
        ) {
            Self::new(device, width, height, depth, usage, format, name).map(|t| Some(t))
        } else {
            Ok(None)
        }
    }
}
