use std::collections::HashMap;
// use std::collections::hash_map::DefaultHasher;

pub struct GeometryBuffer {
    pub(crate) id: u64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    maps: HashMap<String, gfx::GTexture2D>,
    ms_maps: HashMap<String, gfx::GTexture2D>,
    pub depth: gfx::GTexture2D,
    pub ms_depth: Option<gfx::GTexture2D>,
    pub sampler: gpu::Sampler,
    pub depth_sampler: gpu::Sampler,
}

impl GeometryBuffer {
    pub fn new(
        device: &gpu::Device,
        width: u32,
        height: u32,
        ms: gpu::Samples,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let mut maps = HashMap::new();
        let mut ms_maps = HashMap::new();

        let map_name_formats = [
            ("position", gpu::Format::Rgb32Float),
            ("normal", gpu::Format::Rgb32Float),
            ("albedo", gpu::Format::Rgba32Float),
            ("roughness", gpu::Format::R32Float),
            ("metallic", gpu::Format::R32Float),
            ("subsurface", gpu::Format::Rgba32Float),
            ("uv", gpu::Format::Rg32Float),
            ("ao", gpu::Format::R32Float),
            ("output", gpu::Format::Rgba32Float),
        ];

        for (n, format) in map_name_formats {
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
                name.map(|n0| format!("{}_{}", n0, n)),
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
                        name.map(|n0| format!("{}_{}_ms", n0, n)),
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
            name.map(|n| format!("{}_depth", n)),
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
                name.map(|n| format!("{}_depth_ms", n)),
            )?)
        } else {
            None
        };

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
            wrap_z: gpu::WrapMode::ClampToEdge,
            min_filter: gpu::FilterMode::Linear,
            mag_filter: gpu::FilterMode::Linear,
            mipmap_filter: gpu::FilterMode::Linear,
            ..Default::default()
        })?;

        let depth_sampler = device.create_sampler(&gpu::SamplerDesc {
            compare: Some(gpu::CompareOp::LessEqual),
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
            depth_sampler,
        })
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
