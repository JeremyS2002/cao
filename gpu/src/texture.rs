
use crate::DescType;
use gpu_derive::DescType;

use std::sync::Arc;
use std::ptr;
use std::num::NonZeroU32;

use ash::vk::Handle;
use ash::vk;

#[derive(Debug, Clone, DescType)]
pub struct TextureDesc {
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
    /// The name of the texture
    pub name: Option<String>,
}

pub(crate) enum TextureBacking {
    Memory(vk::DeviceMemory),
    Swapchain(Arc<crate::SwapchainInner>),
}

pub(crate) struct TextureInner {
    pub(crate) info: TextureInfo,
    pub(crate) backing: TextureBacking,
    pub(crate) raw: vk::Image,
    pub(crate) device: Arc<crate::DeviceInner>,
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        unsafe {
            // textures backed by the swapchain get destroyed when the swapchain is
            if let TextureBacking::Memory(memory) = self.backing {
                self.device.logical.destroy_image(self.raw, None);
                self.device.logical.free_memory(memory, None);
            }
        }   
    }
}

/// Represents an image on the gpu
///
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImage.html>
pub struct Texture {
	pub(crate) inner: Arc<TextureInner>,
}

impl Texture {
    pub fn create_default_view(&self) -> Result<crate::TextureView, crate::Error> {
        self.create_view(&crate::TextureViewDesc {
            dimension: self.inner.info.dimension.into(),
            base_array_layer: 0,
            base_mip_level: 0,
            mip_levels: self.inner.info.mip_levels.into(),
            format_change: None,
            name: self.inner.info.name.as_ref().map(|n| format!("{}_default_view", n))
        })
    }

    pub fn create_view(&self, desc: &crate::TextureViewDesc) -> Result<crate::TextureView, crate::Error> {
        
        let create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: self.inner.raw,
            format: if let Some(format) = desc.format_change {
                format.into()
            } else {
                self.inner.info.format.into()
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            view_type: desc.dimension.into(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: self.inner.info.format.aspects().into(),
                base_mip_level: desc.base_mip_level,
                level_count: desc.mip_levels,
                base_array_layer: desc.base_array_layer,
                layer_count: desc.dimension.layers(),
            },
            ..Default::default()
        };

        let raw = unsafe { self.inner.device.logical.create_image_view(&create_info, None)? };

        let view = crate::TextureView {
            inner: Arc::new(crate::TextureViewInner {
                info: desc.to_info(),
                raw,
                texture: Arc::clone(&self.inner),
            })
        };

        if let Some(name) = &desc.name {
            self.inner.device.set_name(raw.as_raw(), vk::ObjectType::IMAGE_VIEW, name)?;
        }

        Ok(view)
    }
}