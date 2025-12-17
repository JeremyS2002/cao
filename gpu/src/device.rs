
use std::ffi::CString;
use crate::DescType;
use gpu_derive::DescType;

use std::collections::HashSet;
use std::ffi::CStr;
use std::ptr;
use std::cmp::Ordering;
use std::sync::Arc;

use ash::vk;
use ash::ext;

use parking_lot::RwLock;

/// Infomation about a device - normally represents a gpu or integrated graphics
#[derive(Debug, Clone)]
pub struct PhysDeviceInfo {
    /// the id of the physical device
    pub id: vk::PhysicalDevice,
    /// the vulkan api version that the device supports
    pub api_version: (u32, u32, u32),
    /// the version of the driver for the device
    pub driver_version: u32,
    /// the id of the vendor of the device (who made it)
    pub vendor_id: u32,
    /// the index of this devices as returned by phys_devices
    pub index: usize,
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
	pub phys_device: vk::PhysicalDevice,
}

pub(crate) struct DeviceInner {
    pub(crate) info: DeviceInfo,

    pub(crate) physical: vk::PhysicalDevice,
    pub(crate) logical: ash::Device,
    pub(crate) debug_device: Option<ext::debug_utils::Device>,

    pub(crate) general_queue: vk::Queue,

    pub(crate) errors: RwLock<Vec<String>>,

    pub(crate) instance: Arc<crate::InstanceInner>,
}

impl DeviceInner {
    pub fn check_errors(&self) -> Result<(), crate::Error> {
        if self.debug_device.is_some() {
            let mut errors = self.errors.write();
            if errors.len() == 0 {
                Ok(())
            } else {
                let mut res = Vec::new();
                std::mem::swap(&mut *errors, &mut res);
                Err(crate::Error::Validation(res))
            }
        } else {
            Ok(())
        }
    }

    pub fn wait_idle(&self) -> Result<(), crate::Error> {
        let result = unsafe { self.logical.device_wait_idle() };
        match result {
            Ok(_) => Ok(()),
            Err(e) => return Err(e.into())
        }
    }

    pub fn get_info(&self) -> &DeviceInfo {
        &self.info
    }

    pub(crate) fn set_name(&self, obj: u64, ty: vk::ObjectType, name: &str) -> Result<(), crate::Error> {
        if let Some(debugger) = &self.debug_device {
            let c = CString::new(name.to_string()).unwrap();
            unsafe {
                debugger.set_debug_utils_object_name(&vk::DebugUtilsObjectNameInfoEXT {
                    s_type: vk::StructureType::DEBUG_UTILS_OBJECT_NAME_INFO_EXT,
                    p_next: ptr::null(),
                    object_type: ty,
                    object_handle: obj,
                    p_object_name: c.as_ptr(),
                    ..Default::default()
                })?;
            }
        }

        Ok(())
    }
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        unsafe { 
            self.logical.device_wait_idle().unwrap() 
        }

        unsafe {
            self.logical.destroy_device(None);
        }
    }
}

pub struct Device {
	pub(crate) inner: Arc<DeviceInner>,
}

impl std::ops::Deref for Device {
    type Target = DeviceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Device {

    // =======================================================================
    // =======================================================================
    // # construction functions

    pub fn new(instance: &crate::Instance, desc: &crate::DeviceDesc) -> Result<Self, crate::Error> {

        let (logical_device, general_queue) = Self::create_logical_device(instance, desc)?;

        let debug_device = if instance.inner.debug_utils.get().is_some() {
            Some(ext::debug_utils::Device::new(&*instance.inner.raw, &logical_device))
        } else {
            None
        };

        let device = Self {
            inner: Arc::new(DeviceInner {
                info: desc.to_info(),

                physical: desc.phys_device,
                logical: logical_device,

                debug_device,

                general_queue,

                errors: RwLock::default(),

                instance: Arc::clone(&instance.inner),
            }),
        };

        device.check_errors()?;

        Ok(device)
    }

    pub fn default_phys_device_fn(lhs: &crate::PhysDeviceInfo, rhs: &crate::PhysDeviceInfo) -> Ordering {
        let lhs_val = lhs.device_type as u8;
        let rhs_val = rhs.device_type as u8;
        lhs_val.cmp(&rhs_val)
    }

    pub fn select_default_phys_device(instance: &crate::Instance, surfaces: &[&crate::Surface]) -> Result<crate::PhysDeviceInfo, crate::Error> {
        Self::select_phys_device(instance, surfaces, Self::default_phys_device_fn)
    }

    pub fn select_phys_device<F: Fn(&crate::PhysDeviceInfo, &PhysDeviceInfo) -> Ordering>(instance: &crate::Instance, surfaces: &[&crate::Surface], f: F) -> Result<crate::PhysDeviceInfo, crate::Error> {
        
        let supporting_devices = instance.phys_devices()?
            .into_iter()
            .filter_map(|info| match Self::phys_supports_surfaces(instance, &info, surfaces) {
                Ok(true) => Some(Ok(info)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(supporting_devices.into_iter().max_by(|l, r| f(l, r)).expect("ERROR: no physical devices support surfaces"))
    }

    pub fn phys_supports_surfaces(instance: &crate::Instance, phys: &crate::PhysDeviceInfo, surfaces: &[&crate::Surface]) -> Result<bool, crate::Error> {
        
        let mut supported_count = 0;
        let mut supported = vec![false; surfaces.len()];

        let queue_families = unsafe { instance.inner.raw.get_physical_device_queue_family_properties(phys.id) };
        for queue_family in 0..queue_families.len() {
            for (i, surface) in surfaces.iter().enumerate() {
                if supported[i] { continue }

                let this_supported = unsafe { surface.inner.loader.get_physical_device_surface_support(phys.id, queue_family as u32, surface.inner.raw)? };
                if !this_supported { continue; }

                supported_count += 1;
                supported[i] = true;
            }

            if supported_count == surfaces.len() { break; }
        }

        Ok(supported_count == surfaces.len())
    }

    fn select_queues(instance: &crate::Instance, desc: &DeviceDesc) -> Result<(Vec<vk::DeviceQueueCreateInfo<'static>>, usize), crate::Error> {
        
        let mut queue_flags = vk::QueueFlags::empty();
        if desc.features.contains(crate::DeviceFeatures::GRAPHICS) {
            queue_flags |= vk::QueueFlags::GRAPHICS;
        }
        if desc.features.contains(crate::DeviceFeatures::COMPUTE) {
            queue_flags |= vk::QueueFlags::COMPUTE;
        }
        if desc.features.contains(crate::DeviceFeatures::TRANSFER) {
            queue_flags |= vk::QueueFlags::TRANSFER;
        }

        let mut create_infos = Vec::new();

        let mut general_idx = -1;
        let mut supported_count = 0;
        let mut supported = vec![false; desc.compatible_surfaces.len()];

        let queue_families = unsafe { instance.inner.raw.get_physical_device_queue_family_properties(desc.phys_device) };

        for (family_idx, queue_family) in queue_families.iter().enumerate() {
            let mut use_this = false;
            if general_idx < 0 && queue_family.queue_flags.contains(queue_flags) {
                general_idx = family_idx as isize;
                use_this = true;
            }

            for (surface_idx, surface) in desc.compatible_surfaces.iter().enumerate() {
                if supported[surface_idx] { continue; }

                let this_supported = unsafe { surface.inner.loader.get_physical_device_surface_support(desc.phys_device, family_idx as u32, surface.inner.raw)? };
                if !this_supported { continue; }

                supported_count += 1;
                supported[surface_idx] = true;

                use_this = true;
            }

            if use_this {
                create_infos.push(vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: family_idx as u32,
                    p_queue_priorities: &1.0,
                    queue_count: 1,
                    ..Default::default()
                })
            }

            if general_idx >= 0 && supported_count == desc.compatible_surfaces.len() { break }
        }

        assert!(general_idx >= 0, "ERROR: Failed to find queue family that supports desc");

        Ok((create_infos, general_idx as usize))
    }

    fn get_extensions(instance: &crate::Instance, desc: &DeviceDesc) -> Result<Vec<*const i8>, crate::Error> {
        let available_extension_names = unsafe { instance.inner.raw.enumerate_device_extension_properties(desc.phys_device)? };
        let available_extension_names_set = available_extension_names
            .into_iter()
            .map(|ext| unsafe { CStr::from_ptr(&ext.extension_name[0]) })
            .collect::<HashSet<_>>();
        let needed_extension_names = crate::ffi::device_extension_names(desc.features);

        let requested_extension_names = needed_extension_names.iter().filter_map(|&n| {
                if available_extension_names_set.contains(n) {
                    Some(n.as_ptr())
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!("Requested device extension '{:?}' not present", n);
                    None
                }
            })
        .collect();

        Ok(requested_extension_names)
    }

    fn create_logical_device(instance: &crate::Instance, desc: &DeviceDesc) -> Result<(ash::Device, vk::Queue), crate::Error> {
        let (queues, general_queue_family) = Self::select_queues(instance, desc)?;

        let extensions = Self::get_extensions(instance, desc)?;

        let mut reset_features = vk::PhysicalDeviceHostQueryResetFeatures {
            s_type: vk::StructureType::PHYSICAL_DEVICE_HOST_QUERY_RESET_FEATURES,
            p_next: ptr::null_mut(),
            host_query_reset: vk::TRUE,
            ..Default::default()
        };

        let mut swapchain_features = vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT {
            s_type: vk::StructureType::PHYSICAL_DEVICE_SWAPCHAIN_MAINTENANCE_1_FEATURES_EXT,
            p_next: ptr::null_mut(),
            swapchain_maintenance1: vk::TRUE,
            ..Default::default()
        };

        let mut p_next = ptr::null_mut();

        if desc.features.contains(crate::DeviceFeatures::TIME_QUERIES) {
            reset_features.p_next = p_next;
            p_next = &mut reset_features as *mut _ as *mut _;
        }

        if desc.features.contains(crate::DeviceFeatures::SWAPCHAIN) {
            swapchain_features.p_next = p_next;
            p_next = &mut swapchain_features as *mut _ as *mut _;
        }

        let raw_features = desc.features.into();

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next,
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queues.len() as u32,
            p_queue_create_infos: queues.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            p_enabled_features: &raw_features,
            ..Default::default()
        };

        let logical_device = unsafe { instance.inner.raw.create_device(desc.phys_device, &create_info, None)? };

        let general_queue = unsafe { logical_device.get_device_queue(general_queue_family as u32, 0) };

        Ok((logical_device, general_queue))
    }

    pub(crate) fn select_queue_family_for_surface(&self, surface: &crate::Surface) -> Result<usize, crate::Error> {
        // this selects the first family that supports this surface
        // this is used when creating a swapchain to get the presentation queue
        // this use needs to be kept in sync with Self::select_queues which also selects the first queues that are needed
        // when creating the device since we need to create the queues first before getting them for the swapchain
        let queue_families = unsafe { self.inner.instance.raw.get_physical_device_queue_family_properties(self.inner.info.phys_device) };

        for family_idx in 0..queue_families.len() {
            let supported = unsafe { surface.inner.loader.get_physical_device_surface_support(self.inner.info.phys_device, family_idx as u32, surface.inner.raw)? };
            if supported { return Ok(family_idx) }
        }

        assert!(false, "ERROR: Device doesn't support surface, can't get presentation queue");
        Ok(0)
    }
}