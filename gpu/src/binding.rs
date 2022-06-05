use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    hash::Hash,
    mem::ManuallyDrop as Md,
    ptr,
    sync::Arc,
};

use ash::vk;

use crate::error::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DescriptorLayoutKey {
    pub entries: Vec<crate::DescriptorLayoutEntry>,
}

/// Describes a DescriptorLayout
#[derive(Debug)]
pub struct DescriptorLayoutDesc<'a> {
    /// The name of the DescriptorLayout
    pub name: Option<String>,
    /// All the entries in the DescriptorLayout
    pub entries: &'a [crate::DescriptorLayoutEntry],
}

/// A DescriptorLayout
///
/// Describes the layout of a DescriptorSet
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDescriptorSetLayout.html>
pub struct DescriptorLayout {
    pub(crate) key: DescriptorLayoutKey,
    pub(crate) shader_stages: crate::ShaderStages,
    // doesn't need device as kept alive for lifetime of device
    // for convinience
    pub(crate) raw: vk::DescriptorSetLayout,
    pub(crate) name: Option<String>,
}

impl std::hash::Hash for DescriptorLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl PartialEq for DescriptorLayout {
    fn eq(&self, other: &DescriptorLayout) -> bool {
        self.raw == other.raw
    }
}

impl Eq for DescriptorLayout {}

impl Clone for DescriptorLayout {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            shader_stages: self.shader_stages,
            raw: self.raw,
            name: self.name.clone(),
        }
    }
}

impl std::fmt::Debug for DescriptorLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DescriptorLayout id: {:?} name: {:?}",
            self.raw, self.name
        )
    }
}

impl DescriptorLayout {
    pub unsafe fn raw_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.raw
    }
}

impl DescriptorLayout {
    /// Create a new DescriptorLayout
    pub fn new(device: &crate::Device, desc: &DescriptorLayoutDesc<'_>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create DescriptorLayout, name {:?}", desc.name);

        let key = Self::key(desc);
        let cache = device.raw.descriptor_set_layouts.read();

        if let None = cache.get(&key) {
            drop(cache);
            Self::raw(device, &key)?;
        }
        let raw = *device.raw.descriptor_set_layouts.read().get(&key).unwrap();

        let mut shader_stages = crate::ShaderStages::empty();
        for e in desc.entries.as_ref() {
            shader_stages |= e.stage;
        }
        let s = Self {
            key,
            raw,
            shader_stages,
            name: desc.name.as_ref().map(|s| s.to_string()),
        };
        if let Some(name) = &desc.name {
            device.raw.set_descriptor_layout_name(&s, name.as_ref())?;
        }
        device.raw.check_errors()?;
        Ok(s)
    }

    fn key(desc: &DescriptorLayoutDesc<'_>) -> DescriptorLayoutKey {
        DescriptorLayoutKey {
            entries: desc.entries.iter().cloned().collect(),
        }
    }

    fn raw(
        device: &crate::Device,
        key: &DescriptorLayoutKey,
    ) -> Result<vk::DescriptorSetLayout, Error> {
        let bindings = key
            .entries
            .iter()
            .enumerate()
            .map(|(binding, e)| vk::DescriptorSetLayoutBinding {
                binding: binding as u32,
                descriptor_type: e.ty.into(),
                stage_flags: e.stage.into(),
                descriptor_count: e.count.get(),
                p_immutable_samplers: ptr::null(),
            })
            .collect::<Vec<vk::DescriptorSetLayoutBinding>>();

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
        };
        let layout_result = unsafe { device.raw.create_descriptor_set_layout(&create_info, None) };
        let layout = match layout_result {
            Ok(l) => l,
            Err(e) => return Err(e.into()),
        };
        device
            .raw
            .descriptor_set_layouts
            .write()
            .insert(key.clone(), layout);
        Ok(layout)
    }

    /// Get the id of the descriptor layout
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(self.raw) }
    }
}

union Descriptor {
    buffer: vk::DescriptorBufferInfo,
    image: vk::DescriptorImageInfo,
}

/// Describes a DescriptorSet
#[derive(Debug)]
pub struct DescriptorSetDesc<'a, 'b> {
    /// The name of the DescriptorSet
    pub name: Option<String>,
    /// The layout of the DescriptorSet
    pub layout: &'a DescriptorLayout,
    /// The entries in the DescriptorSet
    pub entries: &'a [crate::DescriptorSetEntry<'b>],
}

/// A DescriptorSet
///
/// Contians resources sent to the gpu to be accessed in shaders
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDescriptorSet.html>
pub struct DescriptorSet {
    pub(crate) layout: vk::DescriptorSetLayout,
    pub(crate) pool: Md<Arc<vk::DescriptorPool>>,
    pub(crate) set: Md<Arc<vk::DescriptorSet>>,
    pub(crate) shader_stages: crate::ShaderStages,
    pub(crate) device: Arc<crate::RawDevice>,
    // keep resources alive while bind group is alive
    pub(crate) textures: Arc<[(crate::TextureView, crate::TextureLayout)]>,
    pub(crate) buffers: Arc<[crate::BufferSlice<'static>]>,
    pub(crate) samplers: Arc<[crate::Sampler]>,
    pub(crate) name: Option<String>,
}

impl std::hash::Hash for DescriptorSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.pool).hash(state)
    }
}

impl PartialEq for DescriptorSet {
    fn eq(&self, other: &DescriptorSet) -> bool {
        **self.pool == **other.pool
    }
}

impl Eq for DescriptorSet {}

impl Clone for DescriptorSet {
    fn clone(&self) -> Self {
        Self {
            layout: self.layout,
            pool: Md::new(Arc::clone(&self.pool)),
            set: Md::new(Arc::clone(&self.set)),
            shader_stages: self.shader_stages,
            device: Arc::clone(&self.device),
            textures: Arc::clone(&self.textures),
            buffers: Arc::clone(&self.buffers),
            samplers: Arc::clone(&self.samplers),
            name: self.name.clone(),
        }
    }
}

impl DescriptorSet {
    pub unsafe fn raw_pool(&self) -> vk::DescriptorPool {
        **self.pool
    }

    pub unsafe fn raw_set(&self) -> vk::DescriptorSet {
        **self.set
    }
}

impl std::fmt::Debug for DescriptorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DescriptorSet id: {:?} name: {:?}",
            **self.pool, self.name
        )
    }
}

impl DescriptorSet {
    /// Create a new DescriptorSet
    pub fn new(device: &crate::Device, desc: &DescriptorSetDesc<'_, '_>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create DescriptorSet, name {:?}", desc.name);

        let (textures, buffers, samplers) = Self::make_cache(desc);

        let (pool, set) = Self::raw(device, desc)?;
        let descriptors = match Self::descriptors(desc) {
            Ok(d) => d,
            Err(e) => {
                unsafe {
                    device.wait_idle().unwrap();
                    device.raw.destroy_descriptor_pool(pool, None);
                }
                return Err(e.into());
            }
        };
        Self::write_descriptors(device, descriptors, desc, set);
        let s = Self {
            pool: Md::new(Arc::new(pool)),
            set: Md::new(Arc::new(set)),
            shader_stages: desc.layout.shader_stages,
            layout: desc.layout.raw,
            device: Arc::clone(&device.raw),

            textures: textures.into_iter().collect::<Arc<[_]>>(),
            buffers: buffers.into_iter().collect::<Arc<[_]>>(),
            samplers: samplers.into_iter().collect::<Arc<[_]>>(),

            name: desc.name.as_ref().map(|s| s.to_string()),
        };
        if let Some(name) = &desc.name {
            device.raw.set_descriptor_set_name(&s, name.as_ref())?;
        }
        device.raw.check_errors()?;
        Ok(s)
    }

    fn make_cache(
        desc: &DescriptorSetDesc<'_, '_>,
    ) -> (
        HashMap<crate::TextureView, crate::TextureLayout>,
        HashSet<crate::BufferSlice<'static>>,
        HashSet<crate::Sampler>,
    ) {
        let mut textures = HashMap::new();
        let mut buffers = HashSet::new();
        let mut samplers = HashSet::new();
        for entry in desc.entries.as_ref() {
            match entry {
                crate::DescriptorSetEntry::Buffer(slice) => {
                    let buffer = slice.buffer.clone().into_owned();
                    buffers.insert(crate::BufferSlice {
                        buffer: Cow::Owned(buffer),
                        offset: slice.offset,
                        size: slice.size,
                    });
                }
                crate::DescriptorSetEntry::BufferArray(buffer_array) => {
                    for slice in buffer_array.as_ref() {
                        let buffer = slice.buffer.clone().into_owned();
                        buffers.insert(crate::BufferSlice {
                            buffer: Cow::Owned(buffer),
                            offset: slice.offset,
                            size: slice.size,
                        });
                    }
                }
                crate::DescriptorSetEntry::Texture(texture, layout) => {
                    if let Some(l) = textures.insert(texture.clone().into_owned(), *layout) {
                        if *layout != l {
                            panic!(
                                "ERROR: DescriptorSetCreation texture {:?} wanted in layout {:?} and {:?} cannot be in both", 
                                texture,
                                *layout,
                                l
                            );
                        }
                    }
                }
                crate::DescriptorSetEntry::TextureArray(array) => {
                    for (texture, layout) in array.as_ref() {
                        if let Some(l) = textures.insert(texture.clone().into_owned(), *layout) {
                            if *layout != l {
                                panic!(
                                    "ERROR: DescriptorSetCreation texture {:?} wanted in layout {:?} and {:?} cannot be in both", 
                                    texture,
                                    *layout,
                                    l
                                );
                            }
                        }
                    }
                }
                crate::DescriptorSetEntry::Sampler(sampler) => {
                    samplers.insert(sampler.clone().into_owned());
                }
                crate::DescriptorSetEntry::SamplerArray(array) => {
                    for sampler in array.as_ref() {
                        samplers.insert(sampler.clone().into_owned());
                    }
                }
                crate::DescriptorSetEntry::CombinedTextureSampler(texture, layout, sampler) => {
                    if let Some(l) = textures.insert(texture.clone().into_owned(), *layout) {
                        if *layout != l {
                            panic!(
                                "ERROR: DescriptorSetCreation texture {:?} wanted in layout {:?} and {:?} cannot be in both", 
                                texture,
                                *layout,
                                l
                            );
                        }
                    }
                    samplers.insert(sampler.clone().into_owned());
                }
                crate::DescriptorSetEntry::CombinedTextureSamplerArray(array) => {
                    for (texture, layout, sampler) in array.as_ref() {
                        if let Some(l) = textures.insert(texture.clone().into_owned(), *layout) {
                            if *layout != l {
                                panic!(
                                    "ERROR: DescriptorSetCreation texture {:?} wanted in layout {:?} and {:?} cannot be in both", 
                                    texture,
                                    *layout,
                                    l
                                );
                            }
                        }
                        samplers.insert(sampler.clone().into_owned());
                    }
                }
            }
        }
        (textures, buffers, samplers)
    }

    fn write_descriptors(
        device: &crate::Device,
        descriptors: Vec<Vec<Descriptor>>,
        desc: &DescriptorSetDesc<'_, '_>,
        set: vk::DescriptorSet,
    ) {
        let mut write = Vec::new();
        let mut i = 0;
        for list in &descriptors {
            let mut j = 0;
            let buffer = match desc.layout.key.entries[i].ty {
                crate::DescriptorLayoutEntryType::UniformBuffer => true,
                crate::DescriptorLayoutEntryType::StorageBuffer { .. } => true,
                _ => false,
            };
            for d in list {
                write.push(vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: set,
                    dst_binding: i as u32,
                    dst_array_element: j,
                    descriptor_type: desc.layout.key.entries[i].ty.into(),
                    descriptor_count: 1,
                    p_buffer_info: if buffer {
                        unsafe { &d.buffer }
                    } else {
                        ptr::null()
                    },
                    p_image_info: if !buffer {
                        unsafe { &d.image }
                    } else {
                        ptr::null()
                    },
                    p_texel_buffer_view: ptr::null(),
                });
                j += 1;
            }
            i += 1;
        }

        unsafe {
            device.raw.update_descriptor_sets(&write, &[]);
        }
    }

    fn make_descriptor(
        e: &crate::DescriptorSetEntry<'_>,
        l: &crate::DescriptorLayoutEntry,
    ) -> Result<Vec<Descriptor>, Error> {
        let count = l.count;
        match l.ty {
            crate::DescriptorLayoutEntryType::UniformBuffer => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::Buffer(b) = e {
                        Ok(vec![Descriptor {
                            buffer: vk::DescriptorBufferInfo {
                                buffer: **b.buffer.raw,
                                offset: b.offset,
                                range: b.size,
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::UniformBuffer with type not Buffer")
                    }
                } else {
                    if let crate::DescriptorSetEntry::BufferArray(b) = e {
                        if count.get() > b.len() as u32 {
                            panic!("ERROR: Attempt to write BufferArray of len {} to DescriptorLayoutEntry of len {}", b.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(b.iter()
                            .map_while(|s| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        buffer: vk::DescriptorBufferInfo {
                                            buffer: **s.buffer.raw,
                                            offset: s.offset,
                                            range: s.size,
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::UniformBuffer with type not Buffer");
                    }
                }
            }
            crate::DescriptorLayoutEntryType::StorageBuffer { .. } => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::Buffer(b) = e {
                        Ok(vec![Descriptor {
                            buffer: vk::DescriptorBufferInfo {
                                buffer: **b.buffer.raw,
                                offset: b.offset,
                                range: b.size,
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::StorageBuffer with type not Buffer")
                    }
                } else {
                    if let crate::DescriptorSetEntry::BufferArray(b) = e {
                        if count.get() > b.len() as u32 {
                            panic!("ERROR: Attempt to write BufferArray of len {} to DescriptorLayoutEntry of len {}", b.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(b.iter()
                            .map_while(|s| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        buffer: vk::DescriptorBufferInfo {
                                            buffer: **s.buffer.raw,
                                            offset: s.offset,
                                            range: s.size,
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::StorageBuffer with type not Buffer")
                    }
                }
            }
            crate::DescriptorLayoutEntryType::SampledTexture { .. } => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::Texture(i, lo) = e {
                        Ok(vec![Descriptor {
                            image: vk::DescriptorImageInfo {
                                sampler: vk::Sampler::null(),
                                image_view: **i.raw,
                                image_layout: (*lo).into(),
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write ")
                    }
                } else {
                    if let crate::DescriptorSetEntry::TextureArray(b) = e {
                        if count.get() > b.len() as u32 {
                            panic!("ERROR: Attempt to write TextureArray of len {} to DescriptorLayoutEntry of len {}", b.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(b.iter()
                            .map_while(|(v, lo)| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        image: vk::DescriptorImageInfo {
                                            sampler: vk::Sampler::null(),
                                            image_view: **v.raw,
                                            image_layout: (*lo).into(),
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::SampledTexture with type not texture");
                    }
                }
            }
            crate::DescriptorLayoutEntryType::StorageTexture { .. } => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::Texture(i, lo) = e {
                        Ok(vec![Descriptor {
                            image: vk::DescriptorImageInfo {
                                sampler: vk::Sampler::null(),
                                image_view: **i.raw,
                                image_layout: (*lo).into(),
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::StorageTexture with type not texture");
                    }
                } else {
                    if let crate::DescriptorSetEntry::TextureArray(b) = e {
                        if count.get() > b.len() as u32 {
                            panic!("ERROR: Attempt to write TextureArray of len {} to DescriptorLayoutEntry of len {}", b.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(b.iter()
                            .map_while(|(v, lo)| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        image: vk::DescriptorImageInfo {
                                            sampler: vk::Sampler::null(),
                                            image_view: **v.raw,
                                            image_layout: (*lo).into(),
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::StorageTexture with type not texture");
                    }
                }
            }
            crate::DescriptorLayoutEntryType::CombinedTextureSampler => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::CombinedTextureSampler(i, lo, s) = e {
                        Ok(vec![Descriptor {
                            image: vk::DescriptorImageInfo {
                                sampler: **s.raw,
                                image_view: **i.raw,
                                image_layout: (*lo).into(),
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::CombinedTextureSampler with type not combined");
                    }
                } else {
                    if let crate::DescriptorSetEntry::CombinedTextureSamplerArray(a) = e {
                        if count.get() > a.len() as u32 {
                            panic!("ERROR: Attempt to write CombinedTextureSamplerArray of len {} to DescriptorLayoutEntry of len {}", a.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(a.iter()
                            .map_while(|(v, lo, s)| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        image: vk::DescriptorImageInfo {
                                            sampler: **s.raw,
                                            image_view: **v.raw,
                                            image_layout: (*lo).into(),
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::CombinedTextureSampler with type not combined");
                    }
                }
            }
            crate::DescriptorLayoutEntryType::Sampler => {
                if count.get() == 1 {
                    if let crate::DescriptorSetEntry::Sampler(s) = e {
                        Ok(vec![Descriptor {
                            image: vk::DescriptorImageInfo {
                                sampler: **s.raw,
                                image_view: vk::ImageView::null(),
                                image_layout: vk::ImageLayout::GENERAL,
                            },
                        }])
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::Sampler with type not sampler");
                    }
                } else {
                    if let crate::DescriptorSetEntry::SamplerArray(s) = e {
                        if count.get() > s.len() as u32 {
                            panic!("ERROR: Attempt to write SamplerArray of len {} to DescriptorLayoutEntry of len {}", s.len(), count.get());
                        }

                        let mut i = 0;
                        Ok(s.iter()
                            .map_while(|s| {
                                if i < count.get() {
                                    i += 1;
                                    Some(Descriptor {
                                        image: vk::DescriptorImageInfo {
                                            sampler: **s.raw,
                                            image_view: vk::ImageView::null(),
                                            image_layout: vk::ImageLayout::GENERAL,
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<_>())
                    } else {
                        panic!("ERROR: Attempt to write to DescriptorLayoutEntryType::Sampler with type not sampler");
                    }
                }
            }
        }
    }

    fn descriptors(desc: &DescriptorSetDesc<'_, '_>) -> Result<Vec<Vec<Descriptor>>, Error> {
        Ok(desc
            .entries
            .iter()
            .zip(&desc.layout.key.entries)
            .map(|(e, l)| Self::make_descriptor(e, l))
            .collect::<Result<Vec<_>, Error>>()?)
    }

    fn raw(
        device: &crate::Device,
        desc: &DescriptorSetDesc<'_, '_>,
    ) -> Result<(vk::DescriptorPool, vk::DescriptorSet), Error> {
        let pool_sizes = desc
            .layout
            .key
            .entries
            .iter()
            .map(|e| (*e).into())
            .collect::<Vec<_>>();
        let pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: 1,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };
        let pool_result = unsafe { device.raw.create_descriptor_pool(&pool_create_info, None) };

        let pool = match pool_result {
            Ok(p) => p,
            Err(e) => return Err(e.into()),
        };

        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: pool,
            descriptor_set_count: 1,
            p_set_layouts: &desc.layout.raw,
        };

        let set_result = unsafe { device.raw.allocate_descriptor_sets(&allocate_info) };

        let set = match set_result {
            Ok(s) => s[0],
            Err(e) => return Err(e.into()),
        };

        Ok((pool, set))
    }

    /// Get a reference to all the buffers used in self
    pub fn buffers<'a>(&'a self) -> &'a [crate::BufferSlice<'static>] {
        &self.buffers
    }

    /// Get a reference to all the textures used in self and the layout that they should be in before setting the DescriptorSet
    pub fn textures<'a>(&'a self) -> &'a [(crate::TextureView, crate::TextureLayout)] {
        &self.textures
    }

    /// Get a reference to all the samplers used in self
    pub fn samplers<'a>(&'a self) -> &'a [crate::Sampler] {
        &self.samplers
    }

    /// Get the id of the descriptor set
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.pool) }
    }
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        unsafe {
            let pool = Md::take(&mut self.pool);
            if let Ok(pool) = Arc::try_unwrap(pool) {
                self.device.wait_idle().unwrap();
                self.device.destroy_descriptor_pool(pool, None);
            }
        }
    }
}
