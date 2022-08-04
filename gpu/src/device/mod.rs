//! Vulkan Device
//!
//! Represents a device that the system can execute commands on. One computer can have multiple devices (eg. discrete and integrated graphics)
//!
//! The device is used to create almost all other objects

use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::sync::Arc;
use std::mem::ManuallyDrop as Md;
use std::sync::Mutex;

use ash::extensions::ext;
use ash::vk;
use vk::Handle;

use crate::error::*;

pub(crate) mod raw;

pub(crate) use raw::*;

/// Infomation about a device
#[derive(Debug)]
pub struct DeviceInfo {
    /// the id of the physical device
    pub id: u64,
    /// the vulkan api version that the device supports
    pub api_version: (u32, u32, u32),
    /// the version of the driver for the device
    pub driver_version: u32,
    /// the id of the vendor of the device (who made it)
    pub vendor_id: u32,
    /// the type of the device
    pub device_type: crate::DeviceType,
    /// the name of the device
    pub name: String,
    /// the properties of the device memory
    pub mem_properties: crate::MemoryProperties,
    /// the limits of the device
    pub limits: crate::DeviceLimits,
}

pub struct DeviceDesc<'a, F: Fn(&DeviceInfo, &DeviceInfo) -> Ordering> {
    /// Optional surface that the device should support presenting to
    pub compatible_surfaces: &'a [&'a crate::Surface],
    /// Features that the device should have
    pub features: crate::DeviceFeatures,
    /// How to choose the device the device
    /// The device with the greatest ordering will be chosen
    pub predicate: F,
}

fn default_device_ordering(l: &DeviceInfo, r: &DeviceInfo) -> Ordering {
    let l_s = l.device_type as u8;
    let r_s = r.device_type as u8;
    l_s.cmp(&r_s)
}

impl Default for DeviceDesc<'static, fn(&DeviceInfo, &DeviceInfo) -> Ordering> {
    fn default() -> Self {
        Self {
            compatible_surfaces: &[],
            features: crate::DeviceFeatures::BASE,
            predicate: default_device_ordering,
        }
    }
}

/// A Device
///
/// all resources are created through the device
/// This actually encompases two concepts in vulkan a physical and logical device
/// The physical device represents a physical gpu
/// The logical device provides access to the physical gpu
/// for simplicity both have been combined into the device struct
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDevice.html>
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDevice.html>
pub struct Device {
    pub(crate) physical: vk::PhysicalDevice,
    pub(crate) queue_family: u32,
    pub(crate) queue: vk::Queue,
    pub(crate) info: DeviceInfo,
    // a command objects used for under the hood initialization
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) command_buffer: vk::CommandBuffer,
    pub(crate) semaphore: Md<Arc<vk::Semaphore>>,
    pub(crate) fence: vk::Fence,
    pub(crate) waiting_on_semaphore: Mutex<Option<Arc<vk::Semaphore>>>,
    // for debugging + error catching
    pub(crate) debug_utils: Option<ext::DebugUtils>,
    pub(crate) debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    // drop the raw last
    pub(crate) raw: Arc<RawDevice>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Device: Physical {:?}", self.physical)
    }
}

impl Device {
    pub unsafe fn raw_device<'a>(&'a self) -> &'a ash::Device {
        &self.raw.device
    }

    pub unsafe fn raw_debug<'a>(&'a self) -> Option<&'a ash::extensions::ext::DebugUtils> {
        self.raw.debug_loader.as_ref()
    }
}

impl Device {
    /// Internal function, create Device from vk::PhysicalDevice and other required info
    fn from_raw(
        instance: &crate::Instance,
        physical: vk::PhysicalDevice,
        info: DeviceInfo,
        features: crate::DeviceFeatures,
        compatible_surfaces: &'_ [&'_ crate::Surface],
    ) -> Result<Self, Error> {
        let queue_info = Self::get_queue_info(instance, features, compatible_surfaces, physical);
        let validation = instance.validation_layers.len() == 0;
        let (enabled_layer_names, enabled_extensions) =
            Self::enabled_layers_extension(instance, physical)?;

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: if validation {
                instance.validation_layers.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if validation {
                enabled_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            p_enabled_features: &features.into(),
        };

        let raw_result = unsafe { instance.raw.create_device(physical, &create_info, None) };
        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        let queue = unsafe { raw.get_device_queue(queue_info.queue_family_index, 0) };

        let (command_pool, command_buffer, fence, semaphore) =
            Self::create_command(&raw, queue_info.queue_family_index)?;

        let debug_utils = if instance.validation_layers.len() != 0 {
            Some(ext::DebugUtils::new(&*crate::VK_ENTRY, &**instance.raw))
        } else {
            None
        };

        let mut raw = Arc::new(RawDevice::new(
            raw,
            Arc::clone(&instance.raw),
            features,
            info.limits,
            debug_utils.clone(),
        ));

        // TODO: not this, it works but there's no way this is defined behaviour
        let p_user_data = Arc::get_mut(&mut raw).unwrap() as *mut RawDevice as *mut c_void;

        let debug_messenger = if let Some(utils) = &debug_utils {
            let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
                s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                p_next: ptr::null(),
                flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                pfn_user_callback: Some(crate::ffi::vulkan_debug_utils_callback),
                p_user_data,
            };

            let result = unsafe { utils.create_debug_utils_messenger(&debug_create_info, None) };

            let messenger = match result {
                Ok(m) => m,
                Err(e) => return Err(e.into()),
            };
            Some(messenger)
        } else {
            None
        };

        Ok(Self {
            raw,
            info,
            physical,
            queue,
            queue_family: queue_info.queue_family_index,
            command_pool,
            command_buffer,
            semaphore: Md::new(Arc::new(semaphore)),
            fence,
            waiting_on_semaphore: Mutex::new(None),
            debug_utils,
            debug_messenger,
        })
    }

    /// Create a new Device from the id of the physical device
    pub fn from_id(
        instance: &crate::Instance,
        id: u64,
        features: crate::DeviceFeatures,
        compatible_surfaces: &'_ [&'_ crate::Surface],
    ) -> Result<Self, Error> {
        let physical = vk::PhysicalDevice::from_raw(id);
        let info = match instance.device_info(physical) {
            Ok(i) => i,
            Err(e) => return Err(e.into()),
        };

        Self::from_raw(instance, physical, info, features, compatible_surfaces)
    }

    /// Create a new Device
    pub fn new<F: Fn(&DeviceInfo, &DeviceInfo) -> Ordering>(
        instance: &crate::Instance,
        desc: &DeviceDesc<'_, F>,
    ) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create Device");

        let (physical, info) = Self::get_physical_device(instance, desc)?;

        Self::from_raw(
            instance,
            physical,
            info,
            desc.features,
            desc.compatible_surfaces,
        )
    }

    fn create_command(
        raw: &ash::Device,
        queue_family: u32,
    ) -> Result<(vk::CommandPool, vk::CommandBuffer, vk::Fence, vk::Semaphore), Error> {
        let pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_family,
        };

        let pool_result = unsafe { raw.create_command_pool(&pool_create_info, None) };

        let pool = match pool_result {
            Ok(p) => p,
            Err(e) => return Err(e.into()),
        };

        let buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let buffer_result = unsafe { raw.allocate_command_buffers(&buffer_alloc_info) };

        let buffer = match buffer_result {
            Ok(b) => b[0],
            Err(e) => return Err(e.into()),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::empty(),
        };

        let fence_result = unsafe { raw.create_fence(&fence_create_info, None) };

        let fence = match fence_result {
            Ok(f) => f,
            Err(e) => return Err(e.into()),
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let semaphore_result = unsafe { raw.create_semaphore(&semaphore_create_info, None) };

        let semaphore = match semaphore_result {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        Ok((pool, buffer, fence, semaphore))
    }

    fn get_physical_device<F>(
        instance: &crate::Instance,
        desc: &DeviceDesc<'_, F>,
    ) -> Result<(vk::PhysicalDevice, crate::DeviceInfo), Error>
    where
        F: Fn(&DeviceInfo, &DeviceInfo) -> Ordering,
    {
        let physical_devices_result = unsafe { instance.raw.enumerate_physical_devices() };
        let physical_devices = match physical_devices_result {
            Ok(d) => d,
            Err(e) => return Err(e.into()),
        };

        let physical_device = physical_devices
            .iter()
            .filter_map(|&physical_device| unsafe {
                let mut supported = false;
                for (i, _) in instance
                    .raw
                    .get_physical_device_queue_family_properties(physical_device)
                    .iter()
                    .enumerate()
                {
                    let mut tmp = true;
                    for &surface in desc.compatible_surfaces {
                        tmp =
                            tmp && Self::queue_supports_surface(physical_device, i as u32, surface)
                    }
                    supported = supported || tmp;
                    if supported {
                        break;
                    }
                }
                if supported {
                    match instance.device_info(physical_device) {
                        Ok(i) => Some((physical_device, i)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .max_by(|l, r| (desc.predicate)(&l.1, &r.1));
        if let Some(device) = physical_device {
            Ok(device)
        } else {
            panic!("ERROR: No device matches the description found")
        }
    }

    fn queue_supports_surface(
        physical: vk::PhysicalDevice,
        queue: u32,
        surface: &crate::Surface,
    ) -> bool {
        let result = unsafe {
            surface
                .loader
                .get_physical_device_surface_support(physical, queue, **surface.raw)
        };
        result.unwrap_or(false)
    }

    fn get_queue_info(
        instance: &crate::Instance,
        features: crate::DeviceFeatures,
        compatible_surfaces: &'_ [&'_ crate::Surface],
        physical: vk::PhysicalDevice,
    ) -> vk::DeviceQueueCreateInfo {
        let mut queue_req = vk::QueueFlags::empty();
        if features.contains(crate::DeviceFeatures::GRAPHICS) {
            queue_req |= vk::QueueFlags::GRAPHICS;
        }
        if features.contains(crate::DeviceFeatures::COMPUTE) {
            queue_req |= vk::QueueFlags::COMPUTE;
        }
        if features.contains(crate::DeviceFeatures::TRANSFER) {
            queue_req |= vk::QueueFlags::TRANSFER;
        }
        let (index, _) = unsafe {
            instance
                .raw
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .enumerate()
                .find(|&(i, f)| {
                    let mut present = true;
                    for s in compatible_surfaces {
                        let ok = Self::queue_supports_surface(physical, i as u32, s);
                        present = present && ok;
                    }
                    f.queue_flags.contains(queue_req) && present
                })
                .unwrap()
        };

        vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: index as u32,
            p_queue_priorities: &1.0,
            queue_count: 1,
        }
    }

    fn enabled_layers_extension(
        instance: &crate::Instance,
        physical: vk::PhysicalDevice,
    ) -> Result<(Vec<*const i8>, Vec<*const i8>), Error> {
        let enabled_layer_names = instance
            .validation_layers
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let available_extension_names_result =
            unsafe { instance.raw.enumerate_device_extension_properties(physical) };
        let available_extension_names = match available_extension_names_result {
            Ok(n) => n,
            Err(e) => return Err(e.into()),
        };
        let available_extension_set = available_extension_names
            .iter()
            .map(|e| unsafe { CStr::from_ptr(&e.extension_name[0]) })
            .collect::<HashSet<_>>();
        let extension_names = &instance.extension_names;
        let enabled_extensions = extension_names
            .iter()
            .filter_map(|&n| {
                if available_extension_set.contains(n) {
                    Some(n.as_ptr())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok((enabled_layer_names, enabled_extensions))
    }

    /// Get infomation about the device
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// wait for the device to be idle
    pub fn wait_idle(&self) -> Result<(), Error> {
        self.raw.wait_idle()
    }

    /// returns the limits of the device
    pub fn limits(&self) -> crate::DeviceLimits {
        self.raw.limits
    }

    /// returns the features of the device
    pub fn features(&self) -> crate::DeviceFeatures {
        self.raw.features
    }

    /// returns limits that apply to textures created with the format kind and usage supplied
    pub fn texture_properties(
        &self,
        format: crate::Format,
        kind: crate::TextureKind,
        usage: crate::TextureUsage,
    ) -> Result<crate::TextureFormatProperties, crate::Error> {
        let raw = unsafe {
            self.raw
                .instance
                .get_physical_device_image_format_properties(
                    self.physical,
                    format.into(),
                    kind.into(),
                    vk::ImageTiling::OPTIMAL,
                    usage.into(),
                    usage.into(),
                )
        };

        match raw {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(e.into()),
        }
    }

    /// create a new swapchain to present to the surface supplied
    pub fn create_swapchain(
        &self,
        surface: &crate::Surface,
        desc: &crate::SwapchainDesc,
    ) -> Result<crate::Swapchain, crate::Error> {
        crate::Swapchain::new(self, surface, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateRenderPass.html>
    pub fn create_render_pass(
        &self,
        desc: &crate::RenderPassDesc,
    ) -> Result<crate::RenderPass, crate::Error> {
        crate::RenderPass::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateCommandPool.html>
    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkAllocateCommandBuffers.html>
    pub fn create_command_buffer(
        &self,
        name: Option<String>,
    ) -> Result<crate::CommandBuffer, crate::Error> {
        crate::CommandBuffer::new(self, name)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateShaderModule.html>
    pub fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDesc,
    ) -> Result<crate::ShaderModule, crate::Error> {
        crate::ShaderModule::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html>    
    pub fn create_buffer(&self, desc: &crate::BufferDesc) -> Result<crate::Buffer, crate::Error> {
        crate::Buffer::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateImage.html>
    pub fn create_texture(
        &self,
        desc: &crate::TextureDesc,
    ) -> Result<crate::Texture, crate::Error> {
        crate::Texture::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateSampler.html>
    pub fn create_sampler(
        &self,
        desc: &crate::SamplerDesc,
    ) -> Result<crate::Sampler, crate::Error> {
        crate::Sampler::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreatePipelineLayout.html>
    pub fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDesc,
    ) -> Result<crate::PipelineLayout, crate::Error> {
        crate::PipelineLayout::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateGraphicsPipelines.html>
    pub fn create_graphics_pipeline(
        &self,
        desc: &crate::GraphicsPipelineDesc,
    ) -> Result<crate::GraphicsPipeline, crate::Error> {
        crate::GraphicsPipeline::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateComputePipelines.html>
    pub fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDesc,
    ) -> Result<crate::ComputePipeline, crate::Error> {
        crate::ComputePipeline::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateDescriptorSetLayout.html>
    pub fn create_descriptor_layout(
        &self,
        desc: &crate::DescriptorLayoutDesc,
    ) -> Result<crate::DescriptorLayout, crate::Error> {
        crate::DescriptorLayout::new(self, desc)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkCreateDescriptorPool.html>
    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkAllocateDescriptorSets.html>
    pub fn create_descriptor_set(
        &self,
        desc: &crate::DescriptorSetDesc,
    ) -> Result<crate::DescriptorSet, crate::Error> {
        crate::DescriptorSet::new(self, desc)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            if let Some(utils) = self.debug_utils.take() {
                utils.destroy_debug_utils_messenger(self.debug_messenger.unwrap(), None);
            }
            self.raw.destroy_command_pool(self.command_pool, None);
            let semaphore = Md::take(&mut self.semaphore);
            if let Ok(semaphore) = Arc::try_unwrap(semaphore) {
                self.raw.destroy_semaphore(semaphore, None);
            }
            self.raw.destroy_fence(self.fence, None);
            if let Some(semaphore) = self.waiting_on_semaphore.lock().unwrap().take() {
                if let Ok(semaphore) = Arc::try_unwrap(semaphore) {
                    self.raw.destroy_semaphore(semaphore, None);
                }
            }
        }
    }
}
