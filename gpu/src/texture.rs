//! Texture, TextureView, TextureSlice + descriptions

use std::{borrow::Cow, mem::ManuallyDrop as Md, num::NonZeroU32, ptr, sync::Arc};

use ash::vk;

use parking_lot::Mutex;

use crate::error::*;

pub(crate) fn init_image_layout(
    device: &crate::Device,
    texture: &crate::Texture,
    layout: crate::TextureLayout,
) -> Result<(), Error> {
    use crate::command::raw;

    raw::begin_primary(device.command_buffer, &device.raw, true)?;

    raw::pipeline_barrier(
        device.command_buffer,
        &device.raw,
        crate::PipelineStageFlags::TOP_OF_PIPE,
        crate::PipelineStageFlags::BOTTOM_OF_PIPE,
        &[],
        &[crate::TextureAccessInfo {
            texture: Cow::Borrowed(&texture),
            base_array_layer: 0,
            array_layers: texture.dimension().layers(),
            base_mip_level: 0,
            mip_levels: texture.mip_levels(),
            src_access: crate::AccessFlags::empty(),
            dst_access: crate::AccessFlags::empty(),
            src_layout: crate::TextureLayout::Undefined,
            dst_layout: layout,
        }],
    )?;

    raw::end_recording(device.command_buffer, &device.raw)?;

    raw::submit(
        &device.raw,
        device.queue,
        device.command_buffer,
        device.semaphore,
        None,
        device.fence,
    )?;

    // let begin_info = vk::CommandBufferBeginInfo {
    //     s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
    //     p_next: ptr::null(),
    //     p_inheritance_info: ptr::null(),
    //     flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
    // };

    // let begin_result = unsafe {
    //     device
    //         .raw
    //         .begin_command_buffer(device.command_buffer, &begin_info)
    // };

    // match begin_result {
    //     Ok(_) => (),
    //     Err(e) => return Err(e.into()),
    // }

    // unsafe {
    //     device.raw.cmd_pipeline_barrier(
    //         device.command_buffer,
    //         vk::PipelineStageFlags::TOP_OF_PIPE,
    //         vk::PipelineStageFlags::BOTTOM_OF_PIPE,
    //         vk::DependencyFlags::empty(),
    //         &[],
    //         &[],
    //         &[vk::ImageMemoryBarrier {
    //             s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
    //             p_next: ptr::null(),
    //             src_access_mask: vk::AccessFlags::empty(),
    //             dst_access_mask: vk::AccessFlags::empty(),
    //             old_layout: vk::ImageLayout::UNDEFINED,
    //             new_layout: layout,
    //             image,
    //             src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    //             dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    //             subresource_range: vk::ImageSubresourceRange {
    //                 aspect_mask: format.aspects().into(),
    //                 base_mip_level: 0,
    //                 level_count: vk::REMAINING_MIP_LEVELS,
    //                 base_array_layer: 0,
    //                 layer_count: vk::REMAINING_ARRAY_LAYERS,
    //             },
    //         }],
    //     );
    // }

    // let end_result = unsafe { device.raw.end_command_buffer(device.command_buffer) };

    // match end_result {
    //     Ok(_) => (),
    //     Err(e) => return Err(e.into()),
    // }

    // let submit_info = vk::SubmitInfo {
    //     s_type: vk::StructureType::SUBMIT_INFO,
    //     p_next: ptr::null(),
    //     wait_semaphore_count: 0,
    //     p_wait_semaphores: ptr::null(),
    //     p_wait_dst_stage_mask: ptr::null(),
    //     signal_semaphore_count: 0,
    //     p_signal_semaphores: ptr::null(),
    //     command_buffer_count: 1,
    //     p_command_buffers: &device.command_buffer,
    // };

    // let submit_result = unsafe {
    //     device
    //         .raw
    //         .queue_submit(device.queue, &[submit_info], device.fence)
    // };

    // match submit_result {
    //     Ok(_) => (),
    //     Err(e) => return Err(e.into()),
    // }

    let wait_result = unsafe { device.raw.wait_for_fences(&[device.fence], true, !0) };

    match wait_result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    }

    let reset_result = unsafe { device.raw.reset_fences(&[device.fence]) };

    match reset_result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

/// Describes a Texture on the gpu
#[derive(Debug)]
pub struct TextureDesc {
    /// The name of the texture
    pub name: Option<String>,
    /// The format of the texture
    /// Determins what a pixel looks like
    pub format: crate::Format,
    /// The usage of the texture
    /// Determins how the texture can be used
    pub usage: crate::TextureUsage,
    /// The dimension of the texture
    /// Determins what type of texture will be created
    pub dimension: crate::TextureDimension,
    /// The mip levels of the texture
    /// Determins how many levels in the [mipmap](https://en.wikipedia.org/wiki/Mipmap)
    pub mip_levels: NonZeroU32,
    /// The memory type of the texture
    /// The type of memory, Device should be the goto
    pub memory: crate::MemoryType,
    /// The initial layout of the texture
    pub layout: crate::TextureLayout,
}

/// Represents an image on the gpu
///
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImage.html>
pub struct Texture {
    pub(crate) name: Option<String>,
    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) raw: Md<Arc<vk::Image>>,
    pub(crate) memory: Option<Arc<vk::DeviceMemory>>,
    pub(crate) usage: crate::TextureUsage,
    pub(crate) format: crate::Format,
    pub(crate) mem_ty: crate::MemoryType,
    pub(crate) mip_levels: u32,
    pub(crate) initial_layout: crate::TextureLayout,
    pub(crate) dimension: crate::TextureDimension,
}

impl std::hash::Hash for Texture {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Texture) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for Texture {}

impl Clone for Texture {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            device: Arc::clone(&self.device),
            raw: Md::new(Arc::clone(&self.raw)),
            memory: self.memory.clone(),
            usage: self.usage,
            format: self.format,
            mem_ty: self.mem_ty,
            mip_levels: self.mip_levels,
            initial_layout: self.initial_layout,
            dimension: self.dimension,
        }
    }
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Texture id: {:?} name: {:?}", **self.raw, self.name)
    }
}

impl Texture {
    pub unsafe fn raw_image(&self) -> vk::Image {
        **self.raw
    }

    /// If the texture if from the swapchain then will return None
    pub unsafe fn raw_memory(&self) -> Option<vk::DeviceMemory> {
        self.memory.as_ref().map(|m| **m)
    }
}

impl Texture {
    /// Create a new Texture from the device and description
    pub fn new(device: &crate::Device, desc: &TextureDesc) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create Texture, name {:?}", desc.name);

        let dimension_flags = desc.dimension.flags();
        let usage_flags = desc.usage.flags();

        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            flags: dimension_flags | usage_flags,
            format: desc.format.into(),
            extent: desc.dimension.into(),
            mip_levels: desc.mip_levels.get(),
            array_layers: desc.dimension.layers(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: desc.usage.into(),
            initial_layout: vk::ImageLayout::UNDEFINED,
            samples: desc.dimension.samples().into(),
            image_type: desc.dimension.into(),
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        let raw_result = unsafe { device.raw.create_image(&create_info, None) };

        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        let mem_req = unsafe { device.raw.get_image_memory_requirements(raw) };

        let memory_alloc = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_req.size,
            memory_type_index: crate::find_memory_type(
                mem_req,
                desc.memory,
                device.info.mem_properties,
            )?,
        };

        let memory_result = unsafe { device.raw.allocate_memory(&memory_alloc, None) };

        let memory = match memory_result {
            Ok(m) => m,
            Err(e) => return Err(e.into()),
        };

        let bind_result = unsafe { device.raw.bind_image_memory(raw, memory, 0) };

        match bind_result {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }

        let s = Self {
            name: desc.name.clone(),
            raw: Md::new(Arc::new(raw)),
            memory: Some(Arc::new(memory)),
            device: Arc::clone(&device.raw),
            usage: desc.usage,
            format: desc.format,
            mip_levels: desc.mip_levels.get(),
            mem_ty: desc.memory,
            dimension: desc.dimension,
            initial_layout: desc.layout,
        };

        match desc.layout {
            // already in desired layout
            crate::TextureLayout::Undefined => (),
            // set layout to desired
            l => init_image_layout(device, &s, l)?,
        }

        if let Some(name) = &desc.name {
            device.raw.set_texture_name(&s, name)?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Create the default view that encompases the whole image
    pub fn create_default_view(&self) -> Result<TextureView, Error> {
        self.create_view(&TextureViewDesc {
            name: None,
            dimension: self.dimension.into(),
            base_array_layer: 0,
            base_mip_level: 0,
            mip_levels: self.mip_levels,
            format_change: None,
        })
    }

    /// Create a TextureView from description
    pub fn create_view(&self, desc: &TextureViewDesc) -> Result<TextureView, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create TextureView, name {:?}", desc.name);

        let create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: **self.raw,
            format: if let Some(format) = desc.format_change {
                format.into()
            } else {
                self.format.into()
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            view_type: desc.dimension.into(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: self.format.aspects().into(),
                base_mip_level: desc.base_mip_level,
                level_count: desc.mip_levels,
                base_array_layer: desc.base_array_layer,
                layer_count: desc.dimension.layers(),
            },
        };

        let view_result = unsafe { self.device.create_image_view(&create_info, None) };

        let view = match view_result {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        let s = TextureView {
            name: desc.name.clone(),
            device: Arc::clone(&self.device),
            extent: desc.dimension.into(),
            raw: Md::new(Arc::new(view)),
            texture: self.clone(),
            base_array_layer: desc.base_array_layer,
            array_layers: desc.dimension.layers(),
            base_mip_level: desc.base_mip_level,
            mip_levels: desc.mip_levels,
            framebuffers: Md::new(Arc::new(Mutex::new(Vec::new()))),
        };

        if let Some(name) = &desc.name {
            self.device.set_texture_view_name(&s, name)?;
        }

        self.device.check_errors()?;

        Ok(s)
    }

    /// Create a new TextureSlice referencing self
    pub fn slice_ref<'a>(&'a self, desc: &TextureSliceDesc) -> TextureSlice<'a> {
        let extent: crate::Extent3D = self.dimension.into();
        if (desc.offset.x as u32 + desc.extent.width) > extent.width
            || (desc.offset.y as u32 + desc.extent.height) > extent.height
            || (desc.offset.z as u32 + desc.extent.depth) > extent.depth
        {
            panic!("ERROR: Attempt to create TextureSlice exceeding base\noffset: {:?}\nextent{:?}\nbase: {:?}", desc.offset, desc.extent, self.dimension);
        }
        if desc.base_array_layer + desc.array_layers > self.dimension.layers() {
            panic!("ERROR: Attempt to create TextureSlice exeeding array layers: base_layer: {} layers: {} base: {:?}", desc.base_array_layer, desc.array_layers, self.dimension);
        }
        if desc.base_mip_level > self.mip_levels {
            panic!("ERROR: Attempt to create TextureSlice with base mip level: {}, from texture with {} mip levels", desc.base_mip_level, self.mip_levels);
        }
        TextureSlice {
            texture: Cow::Borrowed(self),
            offset: desc.offset,
            extent: desc.extent,
            base_array_layer: desc.base_array_layer,
            array_layers: desc.array_layers,
            base_mip_level: desc.base_mip_level,
            mip_levels: desc.mip_levels,
        }
    }

    /// Create a new TextureSlice of the whole texture referencing self
    pub fn whole_slice_ref<'a>(&'a self) -> TextureSlice<'a> {
        self.slice_ref(&TextureSliceDesc {
            offset: crate::Offset3D::ZERO,
            extent: self.dimension.into(),
            base_array_layer: 0,
            array_layers: self.dimension.layers(),
            base_mip_level: 0,
            mip_levels: self.mip_levels,
        })
    }

    /// Create a texture slice owning a clone of self
    pub fn slice_owned<'a, 'b>(&'b self, desc: &TextureSliceDesc) -> TextureSlice<'a> {
        let extent: crate::Extent3D = self.dimension.into();
        if (desc.offset.x as u32 + desc.extent.width) > extent.width
            || (desc.offset.y as u32 + desc.extent.height) > extent.height
            || (desc.offset.z as u32 + desc.extent.depth) > extent.depth
        {
            panic!("ERROR: Attempt to create TextureSlice exceeding base\noffset: {:?}\nextent{:?}\nbase: {:?}", desc.offset, desc.extent, self.dimension);
        }
        if desc.base_array_layer + desc.array_layers > self.dimension.layers() {
            panic!("ERROR: Attempt to create TextureSlice exeeding array layers: base_layer: {} layers: {} base: {:?}", desc.base_array_layer, desc.array_layers, self.dimension);
        }
        if desc.base_mip_level > self.mip_levels {
            panic!("ERROR: Attempt to create TextureSlice with base mip level: {}, from texture with {} mip levels", desc.base_mip_level, self.mip_levels);
        }
        TextureSlice {
            texture: Cow::Owned(self.clone()),
            offset: desc.offset,
            extent: desc.extent,
            base_array_layer: desc.base_array_layer,
            array_layers: desc.array_layers,
            base_mip_level: desc.base_mip_level,
            mip_levels: desc.mip_levels,
        }
    }

    /// Create a new TextureSlice of the whole texture owning a clone of self
    pub fn whole_slice_owned<'a, 'b>(&'b self) -> TextureSlice<'a> {
        self.slice_owned(&TextureSliceDesc {
            offset: crate::Offset3D::ZERO,
            extent: self.dimension.into(),
            base_array_layer: 0,
            array_layers: self.dimension.layers(),
            base_mip_level: 0,
            mip_levels: self.mip_levels,
        })
    }

    /// Create a new TextureSlice owning self
    pub fn into_slice<'a>(self, desc: &TextureSliceDesc) -> TextureSlice<'a> {
        let extent: crate::Extent3D = self.dimension.into();
        if (desc.offset.x as u32 + desc.extent.width) > extent.width
            || (desc.offset.y as u32 + desc.extent.height) > extent.height
            || (desc.offset.z as u32 + desc.extent.depth) > extent.depth
        {
            panic!("ERROR: Attempt to create TextureSlice exceeding base\noffset: {:?}\nextent{:?}\nbase: {:?}", desc.offset, desc.extent, self.dimension);
        }
        if desc.base_array_layer + desc.array_layers > self.dimension.layers() {
            panic!("ERROR: Attempt to create TextureSlice exeeding array layers: base_layer: {} layers: {} base: {:?}", desc.base_array_layer, desc.array_layers, self.dimension);
        }
        if desc.base_mip_level > self.mip_levels {
            panic!("ERROR: Attempt to create TextureSlice with base mip level: {}, from texture with {} mip levels", desc.base_mip_level, self.mip_levels);
        }
        TextureSlice {
            texture: Cow::Owned(self),
            offset: desc.offset,
            extent: desc.extent,
            base_array_layer: desc.base_array_layer,
            array_layers: desc.array_layers,
            base_mip_level: desc.base_mip_level,
            mip_levels: desc.mip_levels,
        }
    }

    /// Create a new TextureSlice of the whole texture owning self
    pub fn into_whole_slice<'a>(self) -> TextureSlice<'a> {
        let desc = TextureSliceDesc {
            offset: crate::Offset3D::ZERO,
            extent: self.dimension.into(),
            base_array_layer: 0,
            array_layers: self.dimension.layers(),
            base_mip_level: 0,
            mip_levels: self.mip_levels,
        };
        self.into_slice(&desc)
    }

    /// Get the usage of the texture
    pub fn usage(&self) -> crate::TextureUsage {
        self.usage
    }

    /// Get the format of the texture
    pub fn format(&self) -> crate::Format {
        self.format
    }

    /// Get the mip levels of the texture
    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    /// Get the dimension of the texture
    pub fn dimension(&self) -> crate::TextureDimension {
        self.dimension
    }

    /// Get the memory type of the texture
    pub fn mem_ty(&self) -> crate::MemoryType {
        self.mem_ty
    }

    /// Get the initial layout of the texture
    pub fn initial_layout(&self) -> crate::TextureLayout {
        self.initial_layout
    }

    /// Get the id of the texture
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.raw) }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            if let Some(memory) = self.memory.take() {
                let raw = Md::take(&mut self.raw);
                if let Ok(raw) = Arc::try_unwrap(raw) {
                    self.device.wait_idle().unwrap();
                    self.device.destroy_image(raw, None);
                }
                if let Ok(memory) = Arc::try_unwrap(memory) {
                    self.device.wait_idle().unwrap();
                    self.device.free_memory(memory, None);
                }
            }
        }
    }
}

/// Describes a TextureView
#[derive(Debug)]
pub struct TextureViewDesc {
    /// The name of the texture view
    pub name: Option<String>,
    /// The dimension of the view
    pub dimension: crate::TextureDimension,
    /// The first mip level in the view
    pub base_mip_level: u32,
    /// The number of mip levels in the view
    pub mip_levels: u32,
    /// the first array layer in the view
    pub base_array_layer: u32,
    /// if the format should be changed
    pub format_change: Option<crate::Format>,
}

/// A view into a texture
///
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageView.html>
pub struct TextureView {
    pub(crate) name: Option<String>,
    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) raw: Md<Arc<vk::ImageView>>,
    pub(crate) texture: Texture,
    pub(crate) extent: crate::Extent3D,
    pub(crate) base_array_layer: u32,
    pub(crate) array_layers: u32,
    pub(crate) base_mip_level: u32,
    pub(crate) mip_levels: u32,
    pub(crate) framebuffers: Md<Arc<Mutex<Vec<crate::FramebufferKey>>>>,
}

impl std::hash::Hash for TextureView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for TextureView {
    fn eq(&self, other: &TextureView) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for TextureView {}

impl Clone for TextureView {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            device: Arc::clone(&self.device),
            raw: Md::new(Arc::clone(&self.raw)),
            texture: self.texture.clone(),
            extent: self.extent,
            array_layers: self.array_layers,
            base_array_layer: self.base_array_layer,
            base_mip_level: self.base_mip_level,
            mip_levels: self.mip_levels,
            framebuffers: Md::new(Arc::clone(&self.framebuffers)),
        }
    }
}

impl std::fmt::Debug for TextureView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TextureView id {:?} from Image id {:?}\nview name {:?}, texture name {:?}",
            **self.raw, **self.texture.raw, self.name, self.texture.name,
        )
    }
}

impl TextureView {
    /// Get the usage of the view
    pub fn usage(&self) -> crate::TextureUsage {
        self.texture.usage
    }

    /// Get the samples of the view
    pub fn samples(&self) -> crate::Samples {
        self.texture.dimension.samples()
    }

    /// Get the extent of the view
    pub fn extent(&self) -> crate::Extent3D {
        self.extent
    }

    /// Get the format of the view
    pub fn format(&self) -> crate::Format {
        self.texture.format
    }

    /// Get the texture that the view looks into
    pub fn texture<'a>(&'a self) -> &'a Texture {
        &self.texture
    }

    /// Get the base mip level of the view
    pub fn base_mip_level(&self) -> u32 {
        self.base_mip_level
    }

    /// Get the number of mip levels of the view
    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    /// Get the base array of the view
    pub fn base_array_layer(&self) -> u32 {
        self.base_array_layer
    }

    /// Get the number of array layers of the view
    pub fn array_layers(&self) -> u32 {
        self.array_layers
    }

    /// Get the id of the view
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.raw) }
    }

    /// Interpret the view as a slice
    pub fn as_slice_ref<'a>(&'a self) -> TextureSlice<'a> {
        self.texture.slice_ref(&TextureSliceDesc {
            offset: crate::Offset3D::ZERO,
            extent: self.extent,
            base_array_layer: self.base_array_layer,
            array_layers: self.array_layers,
            base_mip_level: self.base_mip_level,
            mip_levels: self.mip_levels,
        })
    }

    /// Interpret the view as a slice
    pub fn as_slice_owned<'a>(&self) -> TextureSlice<'a> {
        self.texture.slice_owned(&TextureSliceDesc {
            offset: crate::Offset3D::ZERO,
            extent: self.extent,
            base_array_layer: self.base_array_layer,
            array_layers: self.array_layers,
            base_mip_level: self.base_mip_level,
            mip_levels: self.mip_levels,
        })
    }
}

impl Drop for TextureView {
    fn drop(&mut self) {
        unsafe {
            self.device.wait_idle().unwrap();
            let framebuffers = Md::take(&mut self.framebuffers);
            #[cfg(feature = "parking_lot")]
            if let Ok(framebuffers) = Arc::try_unwrap(framebuffers) {
                for key in framebuffers.into_inner().drain(..) {
                    if let Some(framebuffer) = self.device.framebuffers.write().remove(&key) {
                        self.device.destroy_framebuffer(framebuffer, None);
                    }
                }
            }
            #[cfg(not(feature = "parking_lot"))]
            if let Ok(framebuffers) = Arc::try_unwrap(framebuffers) {
                for key in framebuffers.into_inner().drain(..) {
                    if let Some(framebuffer) = self.device.framebuffers.write().remove(&key) {
                        self.device.destroy_framebuffer(framebuffer, None);
                    }
                }
            }

            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_image_view(raw, None);
            }
        }
    }
}

/// Describes a region of a image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureSliceDesc {
    /// 3d offset from origin
    pub offset: crate::Offset3D,
    /// 3d dimensions of regine
    pub extent: crate::Extent3D,
    /// first image in array
    pub base_array_layer: u32,
    /// number of array layers, use !0 for remaining layers
    pub array_layers: u32,
    /// the mip level
    pub base_mip_level: u32,
    /// the number of mip levels
    pub mip_levels: u32,
}

/// Describes a region of a image
///
/// This is used to indicate what parts of an image to use in a copy
#[derive(Clone, Debug)]
pub struct TextureSlice<'a> {
    pub(crate) texture: Cow<'a, Texture>,
    pub(crate) offset: crate::Offset3D,
    pub(crate) extent: crate::Extent3D,
    pub(crate) base_array_layer: u32,
    pub(crate) array_layers: u32,
    pub(crate) base_mip_level: u32,
    pub(crate) mip_levels: u32,
}

impl std::cmp::PartialEq for TextureSlice<'_> {
    fn eq(&self, other: &TextureSlice<'_>) -> bool {
        // don't know how the derive and cow behave so manual implementation
        self.texture.as_ref().eq(&other.texture)
            && self.offset == other.offset
            && self.extent == other.extent
            && self.base_array_layer == other.base_array_layer
            && self.array_layers == other.array_layers
            && self.base_mip_level == other.base_mip_level
            && self.mip_levels == other.mip_levels
    }
}

impl std::cmp::Eq for TextureSlice<'_> {}

impl std::hash::Hash for TextureSlice<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.texture.as_ref().hash(state);
        self.offset.hash(state);
        self.extent.hash(state);
        self.base_array_layer.hash(state);
        self.array_layers.hash(state);
        self.base_mip_level.hash(state);
        self.mip_levels.hash(state);
    }
}

impl<'a> TextureSlice<'a> {
    /// Take ownership of the texture by consuming the inner value
    /// If the slice is already owned then this should be a nop but in reality probably won't be
    pub fn into_owned(self) -> TextureSlice<'static> {
        TextureSlice {
            texture: Cow::Owned(self.texture.into_owned()),
            offset: self.offset,
            extent: self.extent,
            base_array_layer: self.base_array_layer,
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            base_mip_level: self.base_mip_level,
        }
    }

    /// Take ownership of the texture by cloning the inner value leaving the original in place
    pub fn as_owned(&self) -> TextureSlice<'static> {
        TextureSlice {
            texture: Cow::Owned(self.texture.as_ref().clone()),
            offset: self.offset,
            extent: self.extent,
            base_array_layer: self.base_array_layer,
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            base_mip_level: self.base_mip_level,
        }
    }

    /// Get the texture of the slice
    pub fn texture(&'a self) -> &'a Texture {
        self.texture.as_ref()
    }

    /// Get the texture of the slice
    pub fn cow_texture<'b>(&'b self) -> &'b Cow<'a, Texture> {
        &self.texture
    }

    /// Write binary data to the slice
    ///
    /// If the Texture has memory type Device this will return a error
    /// If the texture is a swapchain texture then this will panic
    pub fn write(&self, data: &[u8]) -> Result<(), Error> {
        let offset = (self.offset.x * self.offset.y * self.offset.z) as usize;
        let size = self.texture.format.size()
            * (self.extent.width * self.extent.height * self.extent.depth) as usize;
        if self.texture.mem_ty == crate::MemoryType::Device {
            panic!("ERROR: Attempt to write to TextureSlice with memory type not visible to host");
        }

        if data.len() < size {
            panic!("ERROR: Attempt to write to TextureSlice with data of size less than required");
        }

        unsafe {
            let p_result = self.texture.device.map_memory(
                **self.texture.memory.as_ref().unwrap(),
                offset as u64,
                size as u64,
                vk::MemoryMapFlags::empty(),
            );

            let p = match p_result {
                Ok(p) => p,
                Err(e) => return Err(e.into()),
            };

            self.texture.device.check_errors()?;

            p.copy_from_nonoverlapping(data.as_ptr() as *const _, size as usize);

            self.texture
                .device
                .unmap_memory(**self.texture.memory.as_ref().unwrap());
        }

        Ok(())
    }

    /// Read binary data from the Texture
    ///
    /// If the Texture has memory type Device this will return an error
    /// If the Texture is a swapchain texture then this will panic
    pub fn read(&self, data: &mut [u8]) -> Result<(), Error> {
        let offset = (self.offset.x * self.offset.y * self.offset.z) as usize;
        let size = self.texture.format.size()
            * (self.extent.width * self.extent.height * self.extent.depth) as usize;
        if self.texture.mem_ty == crate::MemoryType::Device {
            panic!("ERROR: Attempt to read from TextureSlice with memory type not visible to host");
        }

        if data.len() < size {
            panic!("ERROR: Attempt to read from TextureSlice with data of size less than required");
        }

        unsafe {
            let p_result = self.texture.device.map_memory(
                **self.texture.memory.as_ref().unwrap(),
                offset as u64,
                size as u64,
                vk::MemoryMapFlags::empty(),
            );

            let p = match p_result {
                Ok(p) => p,
                Err(e) => return Err(e.into()),
            };

            self.texture.device.check_errors()?;

            data.as_mut_ptr()
                .copy_from_nonoverlapping(p as *const _, size as usize);

            self.texture
                .device
                .unmap_memory(**self.texture.memory.as_ref().unwrap());
        }

        Ok(())
    }

    /// Get the base array layer in the slice
    pub fn base_array_layer(&self) -> u32 {
        self.base_array_layer
    }

    /// Get the number of array layers in the slice
    pub fn array_layers(&self) -> u32 {
        self.array_layers
    }

    /// Get the base mip levels in the texture
    pub fn base_mip_level(&self) -> u32 {
        self.base_mip_level
    }

    /// Get the number of mip levels in the texture
    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    /// Get the extent of the slice
    pub fn extent(&self) -> crate::Extent3D {
        self.extent
    }

    /// Get the offset of the slice
    pub fn offset(&self) -> crate::Offset3D {
        self.offset
    }
}

/// Describes how a texture has been accessed by previous commands in the command buffer
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TextureAccessInfo<'a> {
    /// The texture being accessed
    pub texture: Cow<'a, Texture>,
    /// The first mip level that was accessed
    pub base_mip_level: u32,
    /// The number of mip levels accessed
    pub mip_levels: u32,
    /// The base array layer accessed
    pub base_array_layer: u32,
    /// The number of array layers accessed
    pub array_layers: u32,
    /// How the texture has accessed before
    pub src_access: crate::AccessFlags,
    /// How the texture will be accessed after
    pub dst_access: crate::AccessFlags,
    /// The layout that the texture is in
    pub src_layout: crate::TextureLayout,
    /// The layout that the texture will be in after
    pub dst_layout: crate::TextureLayout,
}
