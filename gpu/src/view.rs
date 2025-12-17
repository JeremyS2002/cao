
use crate::DescType;
use gpu_derive::DescType;

use std::sync::Arc;
// use std::ptr;
// use std::num::NonZeroU32;

use ash::vk;


#[derive(Clone, DescType)]
pub struct TextureViewDesc {
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
    /// The name of the texture view
	pub name: Option<String>,
}

pub struct TextureViewInner {
	pub(crate) info: TextureViewInfo,
	pub(crate) raw: vk::ImageView,
	pub(crate) texture: Arc<crate::TextureInner>,
}

impl Drop for TextureViewInner {
	fn drop(&mut self) {
	    unsafe {
	    	self.texture.device.logical.destroy_image_view(self.raw, None);
	    }
	}
}

pub struct TextureView {
	pub(crate) inner: Arc<TextureViewInner>
}