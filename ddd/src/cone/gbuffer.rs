use core::panic;
use std::collections::HashMap;
// use std::collections::hash_map::DefaultHasher;

/// How precise the geometry buffer should store data
pub enum GeometryBufferPrecision {
    /// 8 bit normalized textures
    /// 
    /// NOTE: This removes the ability to perform some post-processing effects
    /// like bloom or tonemapping
    Low,
    /// 16 bit floating point textures
    Medium,
    /// 32 bit floating point textures
    High,
    /// 64 bit floating point textures
    Ultra,
}

pub struct GeometryBuffer {
    pub(crate) id: u64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub maps: HashMap<String, gfx::GTexture2D>,
    pub ms_maps: HashMap<String, gfx::GTexture2D>,
    pub depth: gfx::GTexture2D,
    pub ms_depth: Option<gfx::GTexture2D>,
    pub sampler: gpu::Sampler,
}

impl GeometryBuffer {
    pub const SIMPLE_MAPS: &'static [(&'static str, u8)] = &[
        ("position", 3),
        ("normal", 3),
        ("albedo", 4),
        ("roughness", 1),
        ("metallic", 1),
        ("uv", 2),
        ("output", 4),
    ];

    pub const SIMPLE_AND_SUBSURFACE_MAPS: &'static [(&'static str, u8)] = &[
        ("position", 3),
        ("normal", 3),
        ("albedo", 4),
        ("roughness", 1),
        ("metallic", 1),
        ("uv", 2),
        ("output", 4),
        ("subsurface", 4),
    ];

    pub const SIMPLE_AND_AO_MAPS: &'static [(&'static str, u8)] = &[
        ("position", 3),
        ("normal", 3),
        ("albedo", 4),
        ("roughness", 1),
        ("metallic", 1),
        ("uv", 2),
        ("output", 4),
        ("ao", 1),
        ("ao_tmp", 1),
    ];

    pub const ALL_MAPS: &'static [(&'static str, u8)] = &[
        ("position", 3),
        ("normal", 3),
        ("albedo", 4),
        ("roughness", 1),
        ("metallic", 1),
        ("uv", 2),
        ("subsurface", 4),
        ("ao", 1),
        ("ao_tmp", 1),
        ("output", 4),
    ];

    pub fn new<'a>(
        device: &gpu::Device,
        width: u32,
        height: u32,
        ms: gpu::Samples,
        quality: GeometryBufferPrecision,
        maps: impl IntoIterator<Item=&'a (&'a str, u8)>,
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        let maps_iter = maps.into_iter();

        let mut maps = HashMap::new();
        let mut ms_maps = HashMap::new();

        use gpu::Format::*;

        let (r, rg, rgb, rgba) = match quality {
            GeometryBufferPrecision::Low => (R8Unorm, Rg8Unorm, Rgb8Unorm, Rgba8Unorm),
            GeometryBufferPrecision::Medium => (R16Float, Rg16Float, Rgb16Float, Rgba16Float),
            GeometryBufferPrecision::High => (R32Float, Rg32Float, Rgb32Float, Rgba32Float),
            GeometryBufferPrecision::Ultra => (R64Float, Rg64Float, Rgb64Float, Rgba64Float),
        };

        for (n, num_components) in maps_iter {
            let format = match *num_components {
                1 => r,
                2 => rg,
                3 => rgb,
                4 => rgba,
                _ => panic!("Call to create Geometry Buffer with map name {} components {}\nThe number of components must be in the range 1..=4", n, num_components),
            };

            let t = gfx::GTexture2D::from_formats(
                device,
                width,
                height,
                gpu::Samples::S1,
                gpu::TextureUsage::COLOR_OUTPUT
                    | gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COPY_SRC
                    | gpu::TextureUsage::COPY_DST,
                1,
                gfx::alt_formats(format),
                name.as_ref().map(|n0| format!("{}_{}", n0, n)),
            )?
            .unwrap();
            maps.insert(n.to_string(), t);

            match ms {
                gpu::Samples::S1 => (),
                _ => {
                    let t = gfx::GTexture2D::from_formats(
                        device,
                        width,
                        height,
                        ms,
                        gpu::TextureUsage::COLOR_OUTPUT
                            | gpu::TextureUsage::SAMPLED
                            | gpu::TextureUsage::COPY_SRC
                            | gpu::TextureUsage::COPY_DST,
                        1,
                        gfx::alt_formats(format),
                        name.as_ref().map(|n0| format!("{}_{}_ms", n0, n)),
                    )?
                    .unwrap();
                    ms_maps.insert(n.to_string(), t);
                }
            }
        }

        let depth = gfx::GTexture2D::new(
            device,
            width,
            height,
            gpu::Samples::S1,
            gpu::TextureUsage::DEPTH_OUTPUT
                | gpu::TextureUsage::SAMPLED
                | gpu::TextureUsage::COPY_SRC
                | gpu::TextureUsage::COPY_DST,
            1,
            gpu::Format::Depth32Float,
            name.as_ref().map(|n| format!("{}_depth", n)),
        )?;

        let ms_depth = if ms != gpu::Samples::S1 {
            Some(gfx::GTexture2D::new(
                device,
                width,
                height,
                ms,
                gpu::TextureUsage::DEPTH_OUTPUT
                    | gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COPY_SRC
                    | gpu::TextureUsage::COPY_DST,
                1,
                gpu::Format::Depth32Float,
                name.as_ref().map(|n| format!("{}_depth_ms", n)),
            )?)
        } else {
            None
        };

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
            wrap_z: gpu::WrapMode::ClampToEdge,
            min_filter: gpu::FilterMode::Nearest,
            mag_filter: gpu::FilterMode::Nearest,
            name: name.as_ref().map(|n| format!("{}_sampler", n)),
            ..Default::default()
        })?;

        Ok(Self {
            id: sampler.id(),
            maps,
            ms_maps,
            depth,
            ms_depth,
            width,
            height,
            sampler,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn ms(&self) -> bool {
        !self.ms_maps.is_empty()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get(&self, name: &str) -> Option<&gfx::GTexture2D> {
        self.maps.get(name)
    }

    pub fn get_ms(&self, name: &str) -> Option<&gfx::GTexture2D> {
        self.ms_maps.get(name)
    }

    pub fn resolve_ref<'a>(&'a self, encoder: &mut gfx::CommandEncoder<'a>) {
        for (n, src) in self.ms_maps.iter() {
            let dst = self.maps.get(n).unwrap();
            encoder.resolve_texture(src.whole_slice_ref(), dst.whole_slice_ref())
        }
        if let Some(ms_depth) = &self.ms_depth {
            encoder.resolve_texture(ms_depth.whole_slice_ref(), self.depth.whole_slice_ref())
        }
    }

    pub fn resolve_owned<'a>(&'a self, encoder: &mut gfx::CommandEncoder<'_>) {
        for (n, src) in self.ms_maps.iter() {
            let dst = self.maps.get(n).unwrap();
            encoder.resolve_texture(src.whole_slice_owned(), dst.whole_slice_owned())
        }
        if let Some(ms_depth) = &self.ms_depth {
            encoder.resolve_texture(ms_depth.whole_slice_owned(), self.depth.whole_slice_owned())
        }
    }

    pub fn clear_texture_ref<'a>(&'a self, encoder: &mut gfx::CommandEncoder<'a>, name: &str, value: gpu::ClearValue) {
        if let Some(t) = self.get(name) {
            encoder.clear_texture(t.texture.whole_slice_ref(), value);
        } else {
            eprintln!("Called GeometryBuffer::clear_texture(.., {})\nNo entry with that name.", name);
        }
    }

    pub fn clear_texture_owned(&self, encoder: &mut gfx::CommandEncoder<'_>, name: &str, value: gpu::ClearValue) {
        if let Some(t) = self.get(name) {
            encoder.clear_texture(t.texture.whole_slice_owned(), value);
        } else {
            eprintln!("Called GeometryBuffer::clear_texture(.., {})\nNo entry with that name.", name);
        }
    }
}
