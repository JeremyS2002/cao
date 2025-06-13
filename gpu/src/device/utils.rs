
use ash::ext;
use ash::khr;

#[derive(Clone)]
pub(crate) struct DebugUtils {
	pub(crate) instance: ext::debug_utils::Instance,
	pub(crate) device: ext::debug_utils::Device,
}

impl DebugUtils {
	pub fn new(instance: &crate::Instance, raw_device: &ash::Device) -> Self {
		let raw_utils_instance = ext::debug_utils::Instance::new(&*crate::VK_ENTRY, &**instance.raw);
		let raw_utils_device = ext::debug_utils::Device::new(&**instance.raw, raw_device);
		Self {
			instance: raw_utils_instance,
			device: raw_utils_device,
		}
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