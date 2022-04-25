//! Sampler + description

use std::{mem::ManuallyDrop as Md, ptr, sync::Arc};

use ash::vk;

use crate::error::*;

/// Describes a sampler
///
/// Note that if you want use the default with mipmaps then you need to set the
/// max lod to be the mipmap levels that the image you are sampling from will have
/// and if you want to use this as a shadow sampler set the compare to Some
#[derive(Debug, Clone, PartialEq)]
pub struct SamplerDesc {
    /// the name of the sampler
    pub name: Option<String>,
    /// what to do when sampling outside the image coordinate range on the x axis
    /// sometimes also called wrap_u
    pub wrap_x: crate::WrapMode,
    /// what to do when sampling outside the image coordinate range on the y axis
    /// sometimes alos called wrap_v
    pub wrap_y: crate::WrapMode,
    /// what to do when sampoing outside the image coordinate range on teh z axis
    /// sometimes alos called wrap_w
    pub wrap_z: crate::WrapMode,
    /// how to filter when too large
    pub mag_filter: crate::FilterMode,
    /// how to filter when too small
    pub min_filter: crate::FilterMode,
    /// how to filter when between mipmap levels
    pub mipmap_filter: crate::FilterMode,
    /// added to mipmap level for sampling
    pub mipmap_bias: f32,
    /// minimum level of detail to be used for mipmap levels
    pub min_lod: f32,
    /// maximum level of detail to be used for mipmap levels
    pub max_lod: f32,
    /// The maximum anisotropy to use
    pub max_anisotropy: Option<f32>,
    /// how compare depth values, if this is some it indicates that this should be a depth sampler
    pub compare: Option<crate::CompareOp>,
    /// the color to be used if any wrap mode is ClampToBorder
    pub border: crate::BorderColor,
}

impl SamplerDesc {
    /// A Description with linear sampling and wrap repeat
    pub const LINEAR: Self = Self {
        name: None,
        wrap_x: crate::WrapMode::MirroredRepeat,
        wrap_y: crate::WrapMode::MirroredRepeat,
        wrap_z: crate::WrapMode::MirroredRepeat,
        mag_filter: crate::FilterMode::Linear,
        min_filter: crate::FilterMode::Linear,
        mipmap_filter: crate::FilterMode::Linear,
        mipmap_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        compare: None,
        max_anisotropy: None,
        border: crate::BorderColor::OpaqueBlack,
    };

    /// A Description with nearest sampling and wrap repeat
    pub const NEAREST: Self = Self {
        name: None,
        wrap_x: crate::WrapMode::MirroredRepeat,
        wrap_y: crate::WrapMode::MirroredRepeat,
        wrap_z: crate::WrapMode::MirroredRepeat,
        mag_filter: crate::FilterMode::Nearest,
        min_filter: crate::FilterMode::Nearest,
        mipmap_filter: crate::FilterMode::Nearest,
        mipmap_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        compare: None,
        max_anisotropy: None,
        border: crate::BorderColor::OpaqueBlack,
    };

    /// A Description with linear sampling and wrap clamp to edge
    pub const CLAMP_EDGE: Self = Self {
        name: None,
        wrap_x: crate::WrapMode::ClampToEdge,
        wrap_y: crate::WrapMode::ClampToEdge,
        wrap_z: crate::WrapMode::ClampToEdge,
        mag_filter: crate::FilterMode::Linear,
        min_filter: crate::FilterMode::Linear,
        mipmap_filter: crate::FilterMode::Linear,
        mipmap_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        compare: None,
        max_anisotropy: None,
        border: crate::BorderColor::OpaqueBlack,
    };

    /// A Description with linear sampling and wrap clamp to border
    pub const CLAMP_BORDER: Self = Self {
        name: None,
        wrap_x: crate::WrapMode::ClampToBorder,
        wrap_y: crate::WrapMode::ClampToBorder,
        wrap_z: crate::WrapMode::ClampToBorder,
        mag_filter: crate::FilterMode::Linear,
        min_filter: crate::FilterMode::Linear,
        mipmap_filter: crate::FilterMode::Linear,
        mipmap_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        compare: None,
        max_anisotropy: None,
        border: crate::BorderColor::OpaqueBlack,
    };
}

impl Default for SamplerDesc {
    fn default() -> Self {
        Self {
            name: None,
            wrap_x: crate::WrapMode::MirroredRepeat,
            wrap_y: crate::WrapMode::MirroredRepeat,
            wrap_z: crate::WrapMode::MirroredRepeat,
            mag_filter: crate::FilterMode::Linear,
            min_filter: crate::FilterMode::Linear,
            mipmap_filter: crate::FilterMode::Linear,
            mipmap_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            compare: None,
            max_anisotropy: None,
            border: crate::BorderColor::OpaqueBlack,
        }
    }
}

/// A Sampler
///
/// Describes how sampling from textures works
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSampler.html>
pub struct Sampler {
    pub(crate) name: Option<String>,
    pub(crate) raw: Md<Arc<vk::Sampler>>,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl PartialEq for Sampler {
    fn eq(&self, other: &Sampler) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for Sampler {}

impl std::hash::Hash for Sampler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl Clone for Sampler {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for Sampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sampler id: {:?} name: {:?}", **self.raw, self.name)
    }
}

impl Sampler {
    /// Create a new Sampler
    pub fn new(device: &crate::Device, desc: &SamplerDesc) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create Sampler, name {:?}", desc.name);

        let create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            address_mode_u: desc.wrap_x.into(),
            address_mode_v: desc.wrap_y.into(),
            address_mode_w: desc.wrap_z.into(),
            mag_filter: desc.mag_filter.into(),
            min_filter: desc.min_filter.into(),
            mipmap_mode: desc.mipmap_filter.into(),
            max_lod: desc.max_lod,
            min_lod: desc.min_lod,
            mip_lod_bias: desc.mipmap_bias,
            anisotropy_enable: if desc.max_anisotropy.is_some() {
                vk::TRUE
            } else {
                vk::FALSE
            },
            max_anisotropy: desc.max_anisotropy.unwrap_or(16.0),
            compare_enable: if desc.compare.is_some() {
                vk::TRUE
            } else {
                vk::FALSE
            },
            compare_op: desc
                .compare
                .map(|d| d.into())
                .unwrap_or(vk::CompareOp::ALWAYS),
            border_color: desc.border.into(),
            unnormalized_coordinates: vk::FALSE,
        };

        let raw_result = unsafe { device.raw.create_sampler(&create_info, None) };

        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(ExplicitError(e).into()),
        };

        let s = Self {
            name: desc.name.as_ref().map(|s| s.to_string()),
            raw: Md::new(Arc::new(raw)),
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &desc.name {
            device.raw.set_sampler_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get the id of the sampler
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.raw) }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.wait_idle().unwrap();
                self.device.destroy_sampler(raw, None);
            }
        }
    }
}