//! ShaderModule + description

use std::{collections::HashMap, ffi::CString, mem::ManuallyDrop as Md, ptr, sync::Arc};

use ash::vk;

use crate::error::*;

/// Describes a shader module
#[derive(Debug)]
pub struct ShaderModuleDesc<'a, 'b> {
    /// the name of the shader module
    pub name: Option<String>,
    /// tuple containting shader stage -> entry point name
    pub entries: &'a [(crate::ShaderStages, &'b str)],
    /// pre compiled spirv data
    pub spirv: &'a [u32],
}

/// A ShaderModule
///
/// Provides a small program to be run on the gpu can have multiple entry points for seperate shader stages
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkShaderModule.html>
pub struct ShaderModule {
    pub(crate) name: Option<String>,
    pub(crate) raw: Md<Arc<vk::ShaderModule>>,
    pub(crate) map: HashMap<crate::ShaderStages, CString>,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl std::hash::Hash for ShaderModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for ShaderModule {
    fn eq(&self, other: &ShaderModule) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for ShaderModule {}

impl Clone for ShaderModule {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            map: self.map.clone(),
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for ShaderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ShaderModule id: {:?} name: {:?}", **self.raw, self.name)
    }
}

impl ShaderModule {
    pub unsafe fn raw_shader_module(&self) -> vk::ShaderModule {
        **self.raw
    }
}

impl ShaderModule {
    /// Create new shader module from description
    ///
    /// # Safety
    ///
    /// The spirv module must be valid and must not make use of features
    /// that that arn't declared in the device creation
    pub fn new(device: &crate::Device, desc: &ShaderModuleDesc<'_, '_>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create ShaderModule, name {:?}", desc.name);

        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: desc.spirv.len() * 4,
            p_code: desc.spirv.as_ptr(),
        };

        let raw_result = unsafe { device.raw.create_shader_module(&create_info, None) };

        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(ExplicitError(e).into()),
        };

        let map = desc
            .entries
            .iter()
            .map(|(t, e)| (*t, CString::new(e.to_string()).unwrap()))
            .collect::<HashMap<_, _>>();

        let s = Self {
            name: desc.name.clone(),
            raw: Md::new(Arc::new(raw)),
            map,
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &desc.name {
            device.raw.set_shader_module_name(&s, name)?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get the id of the shader module
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.raw) }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.wait_idle().unwrap();
                self.device.destroy_shader_module(raw, None);
            }
        }
    }
}
