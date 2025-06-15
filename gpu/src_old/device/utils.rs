
use std::ffi::c_void;
use std::ptr;
use crate::Error;

use ash::vk;
use ash::ext;
use ash::khr;

#[derive(Clone)]
pub(crate) struct DebugUtils {
	pub(crate) instance: ext::debug_utils::Instance,
	// pub(crate) device: ext::debug_utils::Device,
	pub(crate) messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtils {
	pub fn new(instance: &crate::Instance, p_user_data: *mut c_void) -> Result<Self, Error> {
		let debug_instance = ext::debug_utils::Instance::new(&*crate::VK_ENTRY, &**instance.raw);
		// let raw_utils_device = ext::debug_utils::Device::new(&**instance.raw, raw_device);
		let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
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
            ..Default::default()
        };

        let result = unsafe { debug_instance.create_debug_utils_messenger(&messenger_create_info, None) };

        let messenger = match result {
            Ok(m) => m,
            Err(e) => return Err(e.into()),
        };
		Ok(Self {
			instance: debug_instance,
			// device: raw_utils_device,
			messenger
		})
	}
}

#[derive(Clone)]
pub(crate) struct SwapchainUtils {
	pub(crate) instance: khr::swapchain::Instance,
	pub(crate) device: khr::swapchain::Device,
}

impl SwapchainUtils {
	pub fn new(device: &crate::RawDevice) -> Self {
		let raw_utils_instance = khr::swapchain::Instance::new(&*crate::VK_ENTRY, &**device.instance);
		let raw_utils_device = khr::swapchain::Device::new(&**device.instance, &device.device);
		Self {
			instance: raw_utils_instance,
			device: raw_utils_device,
		}
	}
}