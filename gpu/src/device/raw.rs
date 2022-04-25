
use std::ffi::CString;
use std::{collections::HashMap, mem::ManuallyDrop as Md, ptr, sync::Arc};

use ash::extensions::ext;
use ash::vk;

use parking_lot::RwLock;
use parking_lot::Mutex;

use crate::error::*;

pub(crate) struct RawDevice {
    pub framebuffers: RwLock<HashMap<crate::FramebufferKey, vk::Framebuffer>>,
    pub descriptor_set_layouts:
        RwLock<HashMap<crate::DescriptorLayoutKey, vk::DescriptorSetLayout>>,

    pub device: ash::Device,
    pub features: crate::DeviceFeatures,
    pub limits: crate::DeviceLimits,
    pub instance: Md<Arc<ash::Instance>>,

    pub debug_loader: Option<ext::DebugUtils>,
    pub error: RwLock<Vec<String>>,

    pub semaphore: Mutex<Option<vk::Semaphore>>,
}

impl std::ops::Deref for RawDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl RawDevice {
    #[inline]
    pub fn check_errors(&self) -> Result<(), ValidationError> {
        if self.debug_loader.is_some() {
            #[cfg(feature = "parking_lot")]
            let mut errors = self.error.write();
            #[cfg(not(feature = "parking_lot"))]
            let mut errors = self.error.write();
            if errors.len() == 0 {
                return Ok(());
            } else {
                let mut new = Vec::new();
                std::mem::swap(&mut *errors, &mut new);
                return Err(ValidationError(new));
            }
        } else {
            Ok(())
        }
    }

    pub fn wait_idle(&self) -> Result<(), Error> {
        let result = unsafe { self.device_wait_idle() };
        match result {
            Ok(_) => Ok(()),
            Err(e) => return Err(ExplicitError(e).into()),
        }
    }

    pub fn new(
        raw: ash::Device,
        instance: Arc<ash::Instance>,
        features: crate::DeviceFeatures,
        limits: crate::DeviceLimits,
        debug_loader: Option<ext::DebugUtils>,
    ) -> Self {
        Self {
            descriptor_set_layouts: RwLock::new(HashMap::new()),
            framebuffers: RwLock::new(HashMap::new()),

            device: raw,
            features,
            limits,
            instance: Md::new(instance),

            debug_loader,
            error: RwLock::new(Vec::new()),

            semaphore: Mutex::new(None),
        }
    }

    fn match_result(result: Result<(), vk::Result>) -> Result<(), Error> {
        match result {
            Ok(_) => Ok(()),
            Err(e) => return Err(ExplicitError(e).into()),
        }
    }

    fn set_name(&self, obj: u64, ty: vk::ObjectType, name: &str) -> Result<(), Error> {
        let c = CString::new(name.to_string()).unwrap();
        unsafe { 
            if let Some(loader) = &self.debug_loader {
                let result = loader.debug_utils_set_object_name(
                    self.device.handle(),
                    &vk::DebugUtilsObjectNameInfoEXT {
                        s_type: vk::StructureType::DEBUG_UTILS_OBJECT_NAME_INFO_EXT,
                        p_next: ptr::null(),
                        object_type: ty,
                        object_handle: obj,
                        p_object_name: c.as_ptr(),
                    }
                );
                Self::match_result(result)?;
            }
        }
        Ok(())
    }

    pub fn set_shader_module_name(
        &self, 
        module: &crate::ShaderModule,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(**module.raw) }, vk::ObjectType::SHADER_MODULE, name)
    }

    pub fn set_buffer_name(
        &self,
        buffer: &crate::Buffer,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(**buffer.raw) }, vk::ObjectType::BUFFER, name)
    }

    pub fn set_texture_name(
        &self,
        texture: &crate::Texture,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(**texture.raw) }, vk::ObjectType::IMAGE, name)
    }

    pub fn set_texture_view_name(
        &self,
        view: &crate::TextureView,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(**view.raw) }, vk::ObjectType::IMAGE_VIEW, name)
    }

    pub fn set_command_buffer_name(
        &self,
        buffer: &crate::CommandBuffer,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(buffer.pool) }, vk::ObjectType::COMMAND_POOL, name)?;
        self.set_name(unsafe { std::mem::transmute(buffer.buffer) }, vk::ObjectType::COMMAND_BUFFER, name)
    }

    pub fn set_sampler_name(
        &self,
        sampler: &crate::Sampler,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**sampler.raw) }, vk::ObjectType::SAMPLER, name)
    }

    pub fn set_descriptor_set_name(
        &self,
        set: &crate::DescriptorSet,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**set.pool) }, vk::ObjectType::DESCRIPTOR_POOL, name)?;
        self.set_name(unsafe { std::mem::transmute(&**set.set) }, vk::ObjectType::DESCRIPTOR_SET, name)
    }

    pub fn set_descriptor_layout_name(
        &self,
        layout: &crate::DescriptorLayout,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(layout.raw) }, vk::ObjectType::DESCRIPTOR_SET_LAYOUT, name)
    }

    pub fn set_pipeline_layout_name(
        &self,
        layout: &crate::PipelineLayout,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**layout.raw) }, vk::ObjectType::PIPELINE_LAYOUT, name)
    }

    pub fn set_render_pass_name(
        &self,
        pass: &crate::RenderPass,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**pass.raw) }, vk::ObjectType::RENDER_PASS, name)
    }

    pub fn set_graphics_pipeline_name(
        &self,
        pipeline: &crate::GraphicsPipeline,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**pipeline.raw) }, vk::ObjectType::PIPELINE, name)
    }

    pub fn set_compute_pipeline_name(
        &self,
        pipeline: &crate::ComputePipeline,
        name: &str,
    ) -> Result<(), Error> {
        self.set_name(unsafe { std::mem::transmute(&**pipeline.raw) }, vk::ObjectType::PIPELINE, name)
    }
}

impl Drop for RawDevice {
    fn drop(&mut self) {
        unsafe {
            self.wait_idle().unwrap();

            for (_, framebuffer) in self.framebuffers.write().drain() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            for (_, layout) in self.descriptor_set_layouts.write().drain() {
                self.device.destroy_descriptor_set_layout(layout, None);
            }

            self.device.destroy_device(None);
            let instance = Md::take(&mut self.instance);
            if let Ok(instance) = Arc::try_unwrap(instance) {
                instance.destroy_instance(None);
            }
        }
    }
}
