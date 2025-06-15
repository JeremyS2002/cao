
pub(crate) mod utils;
pub(crate) mod ffi;

pub mod error;
pub mod desc;
pub mod data;
pub mod device;
pub mod surface;

pub use error::*;
pub use desc::*;
pub use data::*;
pub use device::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
pub use surface::*;

use std::collections::HashSet;
use std::mem::ManuallyDrop as Md;
use std::sync::Arc;
use std::ffi::{c_void, CStr, CString};
use std::ptr;
use std::sync::OnceLock;

use parking_lot::RwLock;

use ash::{ext, vk};
use vk::Handle;

lazy_static::lazy_static! {
    pub(crate) static ref VK_ENTRY: ash::Entry = unsafe { ash::Entry::load().expect("Failed to create vulkan entry")};
}

/// Describes an instance of the vulkan api
#[derive(Debug, Clone, gpu_derive::DescType)]
pub struct InstanceDesc<'a> {
    /// name of the application
    pub app_name: &'a str,
    /// version of the application (variant, major, minor, patch)
    pub app_version: (u32, u32, u32, u32),
    /// name of the engine
    pub engine_name: &'a str,
    /// version of the engine (variant, major, minor, patch)
    pub engine_version: (u32, u32, u32, u32),
    /// version of the vulkan api (variant, major, minor, patch)
    pub api_version: (u32, u32, u32, u32),
    /// validation layers for the api to use
    /// will be ignored on release builds
    pub validation_layers: &'a [&'a str],
    /// additional extension names, extensions required by this library
    /// will be automatically added additional functionality can be added by
    /// using the the raw_ methods on structs and the ash crate to
    /// create the extension required
    pub extension_names: &'a [&'a str],
}

impl Default for InstanceDesc<'static> {
    fn default() -> Self {
        Self {
            app_name: "",
            app_version: (1, 0, 0, 0),
            engine_name: "",
            engine_version: (1, 0, 0, 0),
            api_version: (0, 1, 0, 0),
            validation_layers: &[],
            extension_names: &[],
        }
    }
}

pub(crate) struct InstanceInner {
	pub raw: Md<ash::Instance>,
	pub debug_utils: OnceLock<utils::DebugUtils>,
	pub validation_errors: RwLock<Vec<String>>,
}

impl InstanceInner {
    pub fn check_errors(&self) -> Result<(), crate::Error> {
        if self.debug_utils.get().is_some() {
            let mut errors = self.validation_errors.write();
            if errors.len() == 0 {
                Ok(())
            } else {
                let mut new = Vec::new();
                std::mem::swap(&mut *errors, &mut new);
                Err(crate::Error::Validation(new))
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for InstanceInner {
	fn drop(&mut self) {
		unsafe {
			if let Some(utils) = self.debug_utils.take() {
				utils.instance.destroy_debug_utils_messenger(utils.messenger, None);
			}
			let raw = Md::take(&mut self.raw);
			raw.destroy_instance(None);
		}
	}
}

pub struct Instance {
	pub(crate) inner: Arc<InstanceInner>,
	pub(crate) info: InstanceInfo,
}

impl Instance {
    pub fn get_info<'a>(&'a self) -> &'a InstanceInfo {
        &self.info
    }

    pub fn get_raw<'a>(&'a self) -> &'a ash::Instance {
        &self.inner.raw
    }

    pub fn get_debug_instance<'a>(&'a self) -> Option<&'a ext::debug_utils::Instance> {
        self.inner.debug_utils.get().map(|d| &d.instance)
    }

    pub fn get_debug_messenger<'a>(&'a self) -> Option<&'a vk::DebugUtilsMessengerEXT> {
        self.inner.debug_utils.get().map(|d| &d.messenger)
    }
}

impl Instance {
    /// Create a new Instance with the KHRONOS_VALIDATION layer enabled
    ///
    /// This is the entry point to the api and will be the first object created
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkInstance.html>
    ///
    /// Panics if VK_LAYER_KHRONOS_validation is unavailable
    /// use [`Instance::no_validation`] to create an instance without validation for realease builds
    pub fn new(desc: &InstanceDesc<'_>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("gpu::Instance::new()");

        let mut validation_layers = desc.validation_layers.to_owned();
        validation_layers.push(ffi::KHRONOS_VALIDATION_NAME);
        let mut desc = (*desc).clone();
        desc.validation_layers = &validation_layers;
        let (s, validation) = unsafe { Self::raw(&desc)? };
        // TODO return error not panic
        if !validation {
            panic!("Validation layer {} not supported\nConsider using gpu::Instance::no_validation(..) instead", ffi::KHRONOS_VALIDATION_NAME)
        } else {
            Ok(s)
        }
    }

    /// Create a new Instance without the KHRONOS_VALIDATION layer
    /// This removes the requirement that the system has the khro validation layers
    /// Installed but consequenctly makes the entire rest of the api unsafe as almost all
    /// checking is performed by validation layers.
    ///
    /// This is the entry point to the api and will be the first object created
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkInstance.html>
    pub unsafe fn no_validation(desc: &InstanceDesc<'_>) -> Result<Self, Error> {
        return unsafe { Self::raw(desc).map(|(s, _)| s) };
    }

    /// returns (Self, VK_LAYER_KHRONOS_validation available)
    unsafe fn raw(desc: &InstanceDesc<'_>) -> Result<(Self, bool), Error> {
        let app_name = CString::new(desc.app_name).unwrap();
        let app_version = vk::make_api_version(
            desc.app_version.0,
            desc.app_version.1,
            desc.app_version.2,
            desc.app_version.3,
        );
        let engine_name = CString::new(desc.engine_name).unwrap();
        let engine_version = vk::make_api_version(
            desc.engine_version.0,
            desc.engine_version.1,
            desc.engine_version.2,
            desc.engine_version.3,
        );
        let api_version = vk::make_api_version(
            desc.api_version.0,
            desc.api_version.1,
            desc.api_version.2,
            desc.api_version.3,
        );
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: app_version,
            p_engine_name: engine_name.as_ptr(),
            engine_version,
            api_version,
            _marker: std::marker::PhantomData,
        };

        let extension_names = Self::extension_names(desc)?;
        let (validation_names, khronos_enabled) = Self::validation_names(desc)?;

        let raw_extension_names = extension_names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();
        let raw_validation_names = validation_names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: raw_validation_names.as_ptr(),
            enabled_layer_count: raw_validation_names.len() as u32,
            pp_enabled_extension_names: raw_extension_names.as_ptr(),
            enabled_extension_count: raw_extension_names.len() as u32,
            ..Default::default()
        };
        let raw = unsafe { VK_ENTRY.create_instance(&create_info, None)? };
        
        let inner = Arc::new(InstanceInner {
            raw: Md::new(raw),
            debug_utils: OnceLock::new(),
            validation_errors: RwLock::default(),
        });

        if validation_names.len() != 0 {
            let p_user_data = &inner.validation_errors as *const _ as *mut c_void;
            let utils = crate::utils::DebugUtils::new(&inner.raw, p_user_data)?;
            inner.debug_utils.set(utils).ok(); // ignore result, this is the only way to initialize
        }

        inner.check_errors()?; // is is possible to have caught any errors?? probably not?

        Ok((Self { inner, info: desc.to_info() }, khronos_enabled))
    }

    fn extension_names(desc: &crate::InstanceDesc<'_>) -> Result<Vec<CString>, crate::Error> {
        let desired_extensions = ffi::instance_extension_names(desc.validation_layers.len() != 0);

        let available_extensions_vec = unsafe { VK_ENTRY.enumerate_instance_extension_properties(None)? };
        let available_extension_set = available_extensions_vec
            .iter()
            .map(|e| unsafe { CStr::from_ptr(&e.extension_name[0]).to_str().unwrap() })
            .collect::<HashSet<_>>();

        let enabled_extensions = desired_extensions
            .into_iter()
            .map(|n| n.to_str().unwrap())
            .chain(desc.extension_names.iter().map(|&n| n))
            .filter_map(|n| if available_extension_set.contains(n) {
                Some(CString::new(n.to_string().into_bytes()))
            } else {
                #[cfg(feature = "logging")]
                log::warn!("Requested instance extension '{:?}' not present", n);
                None
            })
            .collect::<Result<Vec<_>, _>>().unwrap();

        Ok(enabled_extensions)
    }

    fn validation_names(desc: &crate::InstanceDesc<'_>) -> Result<(Vec<CString>, bool), crate::Error> {
        let desired_layers = desc.validation_layers;

        let available_layers_vec = unsafe { VK_ENTRY.enumerate_instance_layer_properties()? };
        let available_layers_set = available_layers_vec
            .iter()
            .map(|l| unsafe { CStr::from_ptr(&l.layer_name[0]).to_str().unwrap() })
            .collect::<HashSet<_>>();

        let enabled_layers = desired_layers
            .iter()
            .filter_map(|&n| if available_layers_set.contains(n) {
                Some(CString::new(n.to_string().into_bytes()))
            } else {
                #[cfg(feature = "logging")]
                log::warn!("Requested validation layer '{:?}' not present", n);
                None
            })
            .collect::<Result<Vec<_>, _>>().unwrap();

        Ok((enabled_layers, available_layers_set.contains(ffi::KHRONOS_VALIDATION_NAME)))
    }

    /// Get infomation about all the devices that are available
    pub fn phys_devices(&self) -> Result<Vec<crate::PhysDeviceInfo>, crate::Error> {
        let devices = unsafe { self.inner.raw.enumerate_physical_devices()? };
        devices.iter().map(|&d| self.get_phys_device_info(d)).collect::<Result<Vec<_>, _>>()
    }

    fn get_phys_device_info(&self, phys_device: vk::PhysicalDevice) -> Result<crate::PhysDeviceInfo, crate::Error> {
        let properties = unsafe { self.inner.raw.get_physical_device_properties(phys_device) };
        let api = properties.api_version;
        let major = vk::api_version_major(api);
        let minor = vk::api_version_minor(api);
        let patch = vk::api_version_patch(api);
        let ty = properties.device_type;
        let name = unsafe {
            CStr::from_ptr(&properties.device_name[0])
                .to_str()
                .unwrap()
                .to_string()
        };
        let mem_properties = unsafe {
            self.inner.raw.get_physical_device_memory_properties(phys_device)
        };
        let limits = properties.limits;

        let raw_extensions = unsafe { self.inner.raw.enumerate_device_extension_properties(phys_device)? };
        let extensions = raw_extensions
            .into_iter()
            .map(|e| unsafe { CStr::from_ptr(&e.extension_name[0]).to_str().unwrap().to_string() })
            .collect();

        Ok(crate::PhysDeviceInfo {
            id: phys_device.as_raw(),
            name,
            api_version: (major, minor, patch),
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_type: if ty == vk::PhysicalDeviceType::CPU {
                DeviceType::Cpu
            } else if ty == vk::PhysicalDeviceType::INTEGRATED_GPU {
                DeviceType::IntegratedGpu
            } else if ty == vk::PhysicalDeviceType::DISCRETE_GPU {
                DeviceType::DiscreteGpu
            } else if ty == vk::PhysicalDeviceType::VIRTUAL_GPU {
                DeviceType::VirtualGpu
            } else {
                DeviceType::Other
            },
            mem_properties,
            limits,
            extensions
        })
    }

    /// Get the names of all supported validation layers
    pub fn validation_layers() -> Result<Vec<String>, crate::Error> {
        let available_validation = unsafe { VK_ENTRY.enumerate_instance_layer_properties()? };

        let layers = available_validation
            .iter()
            .map(|l| {
                unsafe { CStr::from_ptr(&l.layer_name[0]) }
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();

        Ok(layers)
    }

    /// Get the names of all supported extensions
    pub fn extensions() -> Result<Vec<String>, crate::Error> {
        let available_extensions = unsafe { VK_ENTRY.enumerate_instance_extension_properties(None)? };

        let extensions = available_extensions
            .iter()
            .map(|e| {
                unsafe { CStr::from_ptr(&e.extension_name[0]) }
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();

        Ok(extensions)
    }

    /// create a new surface
    pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> Result<crate::Surface, crate::Error> {
        crate::Surface::new(self, window)
    }
}