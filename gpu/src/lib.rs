//! Vulkan wrapper
//!
//! Manages descruction of types via [`Arc`] and aims to remove hide all unsafe code.
//!
//! Any errors caught by validation layers are returned as [`Result`] as [`Error::Validation`] variants contining the error message/
//!
//! [`DescriptorSet`] creation has be abstracted to be more similar to wgpu implementation
//!
//! Syncronisation logic is abstracted only exposing methods to wait on results to complete
//!
//! Surface creation is abstracted thanks to [`raw_window_handle`]
//!
//! This is not at all tested well enough to use for anything serious, do not use this!
//!
//! This is still lacking support for ray tracing, variable shading rates, mesh shading as well as many other extensions.
//!
//! Basic example of creating a window and not drawing anthing to it
//! ```
//! use winit::{
//!     event::{Event, WindowEvent},
//!     event_loop::{ControlFlow, EventLoop},
//!     window::WindowBuilder,
//! };
//!
//! fn main() {
//!     let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();
//!     let event_loop = EventLoop::new();
//!     let window = WindowBuilder::new().build(&event_loop).unwrap();
//!     let surface = instance.create_surface(&window).unwrap();
//!     let device = instance.create_device(&gpu::DeviceDesc {
//!         combatible_surfaces: &[&surface],
//!         ..Default::default()
//!     }).unwrap();
//!     
//!     let mut swapchain = device.create_swapchain(
//!         &surface,
//!         &gpu::SwapchainDesc::from_surface(&surface, &device).unwrap(),
//!     ).unwrap();
//!     
//!     let mut resized = false;
//!     
//!     event_loop.run(move |event, _, control_flow| {
//!         *control_flow = ControlFlow::Poll;
//!
//!         match event {
//!             Event::WindowEvent {
//!                 event: WindowEvent::CloseRequested,
//!                 ..
//!             } => *control_flow = ControlFlow::Exit,
//!             Event::WindowEvent {
//!                 event: WindowEvent::Resized(_),
//!                 ..
//!             } => {
//!                 resized = true;
//!             }
//!             Event::RedrawRequested(_) => {
//!                 if resized {
//!                     swapchain.recreate(&device).unwrap();
//!                     resized = false;
//!                 }
//!
//!                 let view = match swapchain.acquire(!0) {
//!                     Ok((view, _)) => view,
//!                     Err(e) => {
//!                         if e.can_continue() {
//!                             resized = true;
//!                             return;
//!                         } else {
//!                             panic!("{}", e)
//!                         }
//!                     }
//!                 };
//!                 match swapchain.present(view) {
//!                     Ok(_) => (),
//!                     Err(e) => {
//!                         if e.can_continue() {
//!                             resized = true;
//!                             return;
//!                         } else {
//!                             panic!("{}", e);
//!                         }
//!                     }
//!                 }
//!             }
//!             _ => (),
//!         }
//!     });
//! }
//! ```
//!
//! Although the example uses winit to create a window alternatives can be used or none at all.

use ash::{vk, Entry};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;
use vk::Handle;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub mod binding;
pub mod buffer;
pub mod command;
pub mod data;
pub mod device;
pub mod error;
mod ffi;
pub mod format;
pub mod pass;
pub mod pipeline;
pub mod query;
pub mod sampler;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod texture;

pub use binding::*;
pub use buffer::*;
pub use command::*;
pub use data::*;
pub use device::*;
pub use error::*;
use ffi::*;
pub use format::*;
pub use pass::*;
pub use pipeline::*;
pub use query::*;
pub use sampler::*;
pub use shader::*;
pub use surface::*;
pub use swapchain::*;
pub use texture::*;

/// Makes `&[u8]` into `&[u32]` ensuring correct alignment
///
/// returns error if the length isn't a multiple of 4 or if the magic number is missing
///
/// <https://www.reddit.com/r/ProgrammerHumor/comments/99y3ez/smooth_criminal/?utm_source=share&utm_medium=web2x&context=3>
pub fn make_spirv<'a>(data: &'a [u8]) -> Result<Cow<'a, [u32]>, error::MakeSpirvError> {
    // so most of the time this worked
    // let spirv = include_bytes!("path/to/file.spv");
    // let shader_module = device.create_shader_module(&gpu::ShaderModuleDesc {
    //    spirv: &bytemuck::cast_slice(spirv),
    //    entries: &[..]
    // })?;
    // but sometimes it would cause an alignment error so i looked it up and
    // there is some alignment issue so i stole wpgu code to convert pointer alignment
    if data.len() % 4 != 0 {
        return Err(error::MakeSpirvError::NotMultipleOfFour);
    }

    let result = if data.as_ptr().align_offset(std::mem::align_of::<u32>()) == 0 {
        let (_, result, _) = unsafe { data.align_to::<u32>() };
        Cow::from(result)
    } else {
        let mut result = vec![0u32; data.len() / std::mem::size_of::<u32>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                result.as_mut_ptr() as *mut u8,
                data.len(),
            );
        }
        Cow::from(result)
    };

    // <https://www.khronos.org/registry/spir-v/specs/1.0/SPIRV.html#_a_id_magic_a_magic_number>
    const MAGIC_NUMBER: u32 = 0x07230203;

    if result[0] != MAGIC_NUMBER {
        return Err(error::MakeSpirvError::MissingMagicNumber);
    }

    Ok(result)
}

/// Include spirv data from a file directly into the binary
///
/// This ensures correct alignment of bytes and magic number
///
/// <https://www.reddit.com/r/ProgrammerHumor/comments/99y3ez/smooth_criminal/?utm_source=share&utm_medium=web2x&context=3>
#[macro_export]
macro_rules! include_spirv {
    ($($token:tt)*) => {
        $crate::make_spirv(include_bytes!($($token)*)).expect("Failed to convert binary to spirv")
    };
}

lazy_static::lazy_static! {
    pub(crate) static ref VK_ENTRY: ash::Entry = unsafe { Entry::load().expect("Failed to create vulkan entry")};
}

pub const KHRONOS_VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation";

/// Describes an instance of the vulkan api
#[derive(Debug, Clone, Copy)]
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

pub struct Instance {
    pub(crate) raw: Md<Arc<ash::Instance>>,

    pub(crate) extension_names: Vec<&'static CStr>,
    pub(crate) validation_layers: Vec<CString>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "vulkan instance")?;
        writeln!(f, "Validation layers: [")?;
        for layer in &self.validation_layers {
            writeln!(f, "    {:?}", layer)?;
        }
        writeln!(f, "]")?;
        writeln!(f, "Extensions: [")?;
        for ext in &self.extension_names {
            writeln!(f, "    {:?}", ext)?;
        }
        writeln!(f, "]")?;
        Ok(())
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
        validation_layers.push(KHRONOS_VALIDATION);
        let mut desc = (*desc).clone();
        desc.validation_layers = &validation_layers;
        let (s, validation) = unsafe { Self::raw(&desc)? };
        // TODO return error not panic
        if !validation {
            panic!("Validation layer VK_LAYER_KHRONOS_validation not supported\nConsider using gpu::Instance::no_validation(..) instead")
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
            ..Default::default()
        };

        let mut extension_names = instance_extension_names();
        if desc.validation_layers.len() != 0 {
            extension_names.push(ash::ext::debug_utils::NAME);
        }

        let available_extensions_result = unsafe { VK_ENTRY.enumerate_instance_extension_properties(None) };
        let available_extensions = match available_extensions_result {
            Ok(e) => e,
            Err(e) => return Err(e.into()),
        };
        let available_extension_set = available_extensions
            .iter()
            .map(|e| unsafe { CStr::from_ptr(&e.extension_name[0]).to_str().unwrap() })
            .collect::<HashSet<_>>();
        // TODO check this with user supplied extensions
        let pp_enabled_extension_names = extension_names
            .iter()
            .map(|&n| n.to_str().unwrap())
            .chain(desc.extension_names.iter().map(|&l| l))
            .filter_map(|n| {
                if available_extension_set.contains(n) {
                    Some(n.as_ptr() as _)
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!("gpu::Instance - create vkInstance extension {:?} not present", n);
                    None
                }
            })
            .collect::<Vec<*const i8>>();

        let available_validation_result = unsafe { VK_ENTRY.enumerate_instance_layer_properties() };
        let available_validation = match available_validation_result {
            Ok(e) => e,
            Err(e) => return Err(e.into()),
        };

        let available_validation_set = available_validation
            .iter()
            .map(|e| unsafe { CStr::from_ptr(&e.layer_name[0]).to_str().unwrap() })
            .collect::<HashSet<_>>();

        let validation_available = available_validation_set.contains("VK_LAYER_KHRONOS_validation");

        let validation_layers = desc
            .validation_layers
            .iter()
            .filter_map(|&l| {
                if available_validation_set.contains(l) {
                    Some(CString::new(l))
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!("Validation Layer {:?} not present", l);
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let pp_enabled_layer_names = validation_layers
            .iter()
            .map(|l| l.as_ptr())
            .collect::<Vec<_>>();
        let enabled_layer_count = pp_enabled_layer_names.len() as u32;
        let validation = enabled_layer_count != 0;

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if validation {
                pp_enabled_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count,
            pp_enabled_extension_names: pp_enabled_extension_names.as_ptr(),
            enabled_extension_count: pp_enabled_extension_names.len() as u32,
            ..Default::default()
        };
        let raw_result = unsafe { VK_ENTRY.create_instance(&create_info, None) };
        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok((
            Self {
                raw: Md::new(Arc::new(raw)),

                extension_names,
                validation_layers,
            },
            validation_available,
        ))
    }

    /// Get infomation about all the devices that are available
    pub fn devices(&self) -> Result<Vec<crate::DeviceInfo>, Error> {
        let devices_result = unsafe { self.raw.enumerate_physical_devices() };
        let devices = match devices_result {
            Ok(d) => d,
            Err(e) => return Err(e.into()),
        };
        let info = devices
            .iter()
            .map(|&physical_device| self.device_info(physical_device))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(info)
    }

    /// Get the names of all supported validation layers
    pub fn validation_layers() -> Result<Vec<String>, crate::Error> {
        let available_validation_result = unsafe { VK_ENTRY.enumerate_instance_layer_properties() };
        let available_validation = match available_validation_result {
            Ok(e) => e,
            Err(e) => return Err(e.into()),
        };

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
        let available_extensions_result = unsafe { VK_ENTRY.enumerate_instance_extension_properties(None) };
        let available_extensions = match available_extensions_result {
            Ok(e) => e,
            Err(e) => return Err(e.into()),
        };

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

    pub(crate) fn device_info(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<DeviceInfo, Error> {
        let properties = unsafe { self.raw.get_physical_device_properties(physical_device) };
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
            self.raw
                .get_physical_device_memory_properties(physical_device)
        };
        let limits = properties.limits;

        let raw_extensions = unsafe { self.raw.enumerate_device_extension_properties(physical_device)? };
        let extensions = raw_extensions
            .into_iter()
            .map(|e| unsafe { CStr::from_ptr(&e.extension_name[0]).to_str().unwrap().to_string() })
            .collect();

        Ok(crate::DeviceInfo {
            id: physical_device.as_raw(),
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

    /// create a new surface
    pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<crate::Surface, Error> {
        crate::Surface::new(self, window)
    }

    /// create a new device
    pub fn create_device<F: Fn(&DeviceInfo, &DeviceInfo) -> Ordering>(
        &self,
        desc: &crate::DeviceDesc<'_, F>,
    ) -> Result<crate::Device, Error> {
        crate::Device::new(self, desc)
    }

    /// create a new device from id of physical device
    pub fn create_device_from_id(
        &self,
        id: u64,
        features: crate::DeviceFeatures,
        compatible_surfaces: &'_ [&'_ crate::Surface],
    ) -> Result<crate::Device, Error> {
        crate::Device::from_id(self, id, features, compatible_surfaces)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                raw.destroy_instance(None);
            }
        }
    }
}
