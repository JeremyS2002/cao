use core::panic;
use std::collections::HashMap;
// use std::collections::hash_map::DefaultHasher;

/// How precise the geometry buffer should store data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

/// Describes how a texture's dimensions in a geometry buffer should be scaled
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MapScale {
    /// Half this textures dimensions
    Half,
    /// Quater this textures dimensions
    Quater,
    /// Eighth this textures dimensions
    Eighth,
}

impl MapScale {
    /// how many bits to shift to scale the map's dimensions by self
    pub fn shift(&self) -> usize {
        match self {
            MapScale::Half => 1,
            MapScale::Quater => 2,
            MapScale::Eighth => 3,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct GeometryBufferDesc<'a, F: Fn(&str) -> Option<MapScale>> {
    /// The default width of maps in the geometry buffer
    pub width: u32,
    /// The default height of maps in the geometry buffer
    pub height: u32,
    /// The number of samples in the geometry buffer
    pub samples: gpu::Samples,
    /// The precisiion of components of maps in the geometry buffer
    pub precision: GeometryBufferPrecision,
    /// The maps the geometry buffer contiains (name, components, shift)
    pub maps: &'a [(
        &'a str,
        u32,
    )],
    pub map_scale: F,
    pub name: Option<String>,
}

fn default_map_scale(_: &str) -> Option<MapScale> {
    None
}

impl<'a> GeometryBufferDesc<'a, fn(&str) -> Option<MapScale>> {
    /// A simple geometry buffer 
    /// 
    /// Doesn't have capabilities for:
    ///  - ambient occlusion
    ///  - subsurface materials
    pub const SIMPLE: Self = Self {
        width: 512,
        height: 512,
        samples: gpu::Samples::S1,
        precision: GeometryBufferPrecision::Medium,
        maps: &[
            ("world_pos", 3),
            ("view_pos", 3),
            ("normal", 3),
            ("albedo", 4),
            ("roughness", 1),
            ("metallic", 1),
            ("uv", 2),
            ("output", 4),
        ],
        map_scale: default_map_scale,
        name: None,
    };

    /// Adds subsurface material capabilities to [`Self::SIMPLE`]
    pub const SUBSURFACE: Self = Self {
        width: 512,
        height: 512,
        samples: gpu::Samples::S1,
        precision: GeometryBufferPrecision::Medium,
        maps: &[
            ("world_pos", 3),
            ("view_pos", 3),
            ("normal", 3),
            ("albedo", 4),
            ("roughness", 1),
            ("metallic", 1),
            ("uv", 2),
            ("output", 4),
            ("subsurface", 4),
        ],
        map_scale: default_map_scale,
        name: None,
    };

    /// Adds ambient occlusion capabilities to [`Self::SIMPLE`]
    pub const AO: Self = Self {
        width: 512,
        height: 512,
        samples: gpu::Samples::S1,
        precision: GeometryBufferPrecision::Medium,
        maps: &[
            ("world_pos", 3),
            ("view_pos", 3),
            ("normal", 3),
            ("albedo", 4),
            ("roughness", 1),
            ("metallic", 1),
            ("uv", 2),
            ("output", 4),
            ("ao", 1),
        ],
        map_scale: default_map_scale,
        name: None,
    };

    /// Has all maps 
    pub const ALL: Self = Self {
        width: 512,
        height: 512,
        samples: gpu::Samples::S1,
        precision: GeometryBufferPrecision::Medium,
        maps: &[
            ("world_pos", 3),
            ("view_pos", 3),
            ("normal", 3),
            ("albedo", 4),
            ("roughness", 1),
            ("metallic", 1),
            ("uv", 2),
            ("output", 4),
            ("subsurface", 4),
            ("ao", 1),
        ],
        map_scale: default_map_scale,
        name: None,
    };
}

pub struct GeometryBuffer {
    pub(crate) id: u64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) name: Option<String>,
    /// HashMap from name to texture
    pub maps: HashMap<String, gfx::GTexture2D>,
    /// HashMap from name to texture 
    /// If created with [`gpu::Samples::S1`] then this will be empty
    pub ms_maps: HashMap<String, gfx::GTexture2D>,
    /// Depth texture
    pub depth: gfx::GTexture2D,
    /// Multisampled depth texture
    pub ms_depth: Option<gfx::GTexture2D>,
    /// Sampler
    pub sampler: gpu::Sampler,
}

impl std::fmt::Debug for GeometryBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.debug_struct("GeometryBuffer").field("id", &self.id).field("width", &self.width).field("height", &self.height).field("name", &self.name).field("maps", &self.maps).field("ms_maps", &self.ms_maps).field("depth", &self.depth).field("ms_depth", &self.ms_depth).field("sampler", &self.sampler).finish()
        if let Some(name) = &self.name {
            writeln!(f, "{}", name)
        } else {
            writeln!(f, "geometry_buffer_{}", self.id)
        }
    }
}

impl GeometryBuffer {
    /// Create a new [`GeometryBuffer`]
    /// 
    /// ms indicates to create multisampled textures or not
    /// if ms is true then for each entry in maps two textures will be created
    /// identicle except one will have ms samples and one will have [`gpu::Samples::S1`] samples
    /// 
    /// bloom indicates if to create bloom textures or not
    pub fn new<'a, F: Fn(&str) -> Option<MapScale>>(
        device: &gpu::Device,
        desc: &GeometryBufferDesc<F>,
    ) -> Result<Self, gpu::Error> {
        let maps_iter = desc.maps.into_iter();

        let mut maps = HashMap::new();
        let mut ms_maps = HashMap::new();

        use gpu::Format::*;

        let (r, rg, rgb, rgba) = match desc.precision {
            GeometryBufferPrecision::Low => (R8Unorm, Rg8Unorm, Rgb8Unorm, Rgba8Unorm),
            GeometryBufferPrecision::Medium => (R16Float, Rg16Float, Rgb16Float, Rgba16Float),
            GeometryBufferPrecision::High => (R32Float, Rg32Float, Rgb32Float, Rgba32Float),
            GeometryBufferPrecision::Ultra => (R64Float, Rg64Float, Rgb64Float, Rgba64Float),
        };

        for (n, num_components) in maps_iter {
            let shift = if let Some(scale) = (desc.map_scale)(*n) {
                scale.shift()
            } else {
                0
            };

            let format = match *num_components {
                1 => r,
                2 => rg,
                3 => rgb,
                4 => rgba,
                _ => panic!("Call to create Geometry Buffer with map name {} components {}\nThe number of components must be in the range 1..=4", n, num_components),
            };

            let tn = desc.name.as_ref().map(|n0| format!("{}_{}", n0, n));
            let t = gfx::GTexture2D::from_formats(
                device,
                desc.width >> shift,
                desc.height >> shift,
                gpu::Samples::S1,
                gpu::TextureUsage::COLOR_OUTPUT
                    | gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COPY_SRC
                    | gpu::TextureUsage::COPY_DST,
                1,
                gfx::alt_formats(format),
                tn.as_ref().map(|n| &**n),
            )?
            .unwrap();
            maps.insert(n.to_string(), t);

            match desc.samples {
                gpu::Samples::S1 => (),
                _ => {
                    let tn = desc.name.as_ref().map(|n0| format!("{}_{}_ms", n0, n));
                    let t = gfx::GTexture2D::from_formats(
                        device,
                        desc.width >> shift,
                        desc.height >> shift,
                        desc.samples,
                        gpu::TextureUsage::COLOR_OUTPUT
                            | gpu::TextureUsage::SAMPLED
                            | gpu::TextureUsage::COPY_SRC
                            | gpu::TextureUsage::COPY_DST,
                        1,
                        gfx::alt_formats(format),
                        tn.as_ref().map(|n| &**n),
                    )?
                    .unwrap();
                    ms_maps.insert(n.to_string(), t);
                }
            }
        }

        let dn = desc.name.as_ref().map(|n| format!("{}_depth", n));
        let depth = gfx::GTexture2D::new(
            device,
            desc.width,
            desc.height,
            gpu::Samples::S1,
            gpu::TextureUsage::DEPTH_OUTPUT
                | gpu::TextureUsage::SAMPLED
                | gpu::TextureUsage::COPY_SRC
                | gpu::TextureUsage::COPY_DST,
            1,
            gpu::Format::Depth32Float,
            dn.as_ref().map(|n| &**n),
        )?;

        let dn = desc.name.as_ref().map(|n| format!("{}_depth_ms", n));
        let ms_depth = if desc.samples != gpu::Samples::S1 {
            Some(gfx::GTexture2D::new(
                device,
                desc.width,
                desc.height,
                desc.samples,
                gpu::TextureUsage::DEPTH_OUTPUT
                    | gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COPY_SRC
                    | gpu::TextureUsage::COPY_DST,
                1,
                gpu::Format::Depth32Float,
                dn.as_ref().map(|n| &**n),
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
            name: desc.name.as_ref().map(|n| format!("{}_sampler", n)),
            ..Default::default()
        })?;

        Ok(Self {
            id: sampler.id(),
            maps,
            name: desc.name.clone(),
            ms_maps,
            depth,
            ms_depth,
            width: desc.width,
            height: desc.height,
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
