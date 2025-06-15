
use crate::DescType;
use gpu_derive::DescType;

use std::sync::Arc;

// use ash::vk;

/// Infomation about a device - normally represents a gpu or integrated graphics
#[derive(Debug)]
pub struct PhysDeviceInfo {
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
    /// the extensions that the device supports
    pub extensions: Vec<String>,
}

#[derive(Clone, Debug, DescType)]
pub struct DeviceDesc<'a> {
    /// Optional surface that the device should support presenting to
	#[skip_info]
	pub compatible_surfaces: &'a [&'a crate::Surface],
    /// Features that the device should have
	pub features: crate::DeviceFeatures,
	pub phys_device: u8,
}

pub(crate) struct DeviceInner {

}

pub struct Device {
	pub(crate) inner: Arc<DeviceInner>,
	pub(crate) instance: Arc<crate::InstanceInner>,
}