
use std::ffi::{CStr, c_void};

use ash::vk;

use parking_lot::RwLock;

pub const KHRONOS_VALIDATION_NAME: &'static str = "VK_LAYER_KHRONOS_validation";

#[cfg(target_os = "macos")]
fn platform_instance_extensions() -> Vec<&'static CStr> {
    vec![ash::mvk::macos_surface::NAME]
}

#[cfg(windows)]
fn platform_instance_extensions() -> Vec<&'static CStr> {
    vec![ash::khr::win32_surface::NAME]
}

#[cfg(target_os = "linux")]
fn platform_instance_extensions() -> Vec<&'static CStr> {
    vec![
        ash::khr::xlib_surface::NAME,
        ash::khr::xcb_surface::NAME,
        ash::khr::wayland_surface::NAME,
    ]
}

#[cfg(target_os = "android")]
fn platform_instance_extensions() -> Vec<&'static CStr> {
    vec![ash::khr::AndroidSurface::NAME]
}

pub(crate) fn instance_extension_names(debug: bool) -> Vec<&'static CStr> {
	let mut v = platform_instance_extensions();
    v.push(ash::khr::surface::NAME);
    v.push(ash::khr::get_surface_capabilities2::NAME);
    v.push(ash::ext::surface_maintenance1::NAME);
    v.push(ash::khr::get_physical_device_properties2::NAME);
    if debug {
        v.push(ash::ext::debug_utils::NAME);    
    }
    v
}

pub(crate) fn device_extension_names(features: crate::DeviceFeatures) -> Vec<&'static CStr> {
    let mut v = vec![];
    if features.contains(crate::DeviceFeatures::SWAPCHAIN) {
        v.push(ash::khr::swapchain::NAME);
        v.push(ash::ext::swapchain_maintenance1::NAME);
    }
    v
}

#[allow(unused_variables)]
pub(crate) unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void,
) -> vk::Bool32 {
    let validation_errors = unsafe { &*(p_user_data as *const RwLock<Vec<String>>) };
    let message = unsafe { CStr::from_ptr((*p_callback_data).p_message) }
        .to_str()
        .unwrap();
    let ty = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            #[cfg(feature = "logging")]
            log::error!("GPU VALIDATION {}", message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {}", message);

            let mut error = validation_errors.write();
            error.push(message.to_string());
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            #[cfg(feature = "logging")]
            log::trace!("GPU VALIDATION {} {}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {}", ty, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            #[cfg(feature = "logging")]
            log::warn!("GPU VALIDATION {} {}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {}", ty, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            #[cfg(feature = "logging")]
            log::info!("GPU VALIDATION {} {:?}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {}", ty, message);
        }
        _ => (),
    }

    //println!("[Debug]{:?}{}{:?}", message_severity, ty, message);

    vk::FALSE
}
