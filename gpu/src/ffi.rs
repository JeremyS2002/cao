use ash::vk;
use std::ffi::{c_void, CStr};

#[allow(unused_variables)]
pub(crate) unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void,
) -> vk::Bool32 {
    let raw_device = unsafe { &*(p_user_data as *const crate::RawDevice) };
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
            log::error!("GPU VALIDATION {:?}", message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {:?}", message);

            let mut error = raw_device.error.write();
            error.push(message.to_string());
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            #[cfg(feature = "logging")]
            log::trace!("GPU VALIDATION {} {:?}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {:?}", ty, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            #[cfg(feature = "logging")]
            log::warn!("GPU VALIDATION {} {:?}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {:?}", ty, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            #[cfg(feature = "logging")]
            log::info!("GPU VALIDATION {} {:?}", ty, message);
            #[cfg(not(feature = "logging"))]
            eprintln!("GPU VALIDATION {} {:?}", ty, message);
        }
        _ => (),
    }

    //println!("[Debug]{:?}{}{:?}", message_severity, ty, message);

    vk::FALSE
}

#[cfg(target_os = "macos")]
fn required_extension_names() -> Vec<&'static CStr> {
    vec![ash::extensions::mvk::MacOSSurface::name()]
}

#[cfg(windows)]
fn required_extension_names() -> Vec<&'static CStr> {
    vec![ash::extensions::khr::Win32Surface::name()]
}

#[cfg(target_os = "linux")]
fn required_extension_names() -> Vec<&'static CStr> {
    vec![
        // ash::extensions::khr::XlibSurface::name(),
        // ash::extensions::khr::XcbSurface::name(),
        // ash::extensions::khr::WaylandSurface::name(),
        ash::khr::xlib_surface::NAME,
        ash::khr::xcb_surface::NAME,
        ash::khr::wayland_surface::NAME,
    ]
}

#[cfg(target_os = "android")]
fn required_extension_names() -> Vec<&'static CStr> {
    vec![ash::extensions::khr::AndroidSurface::name()]
}

pub(crate) fn extension_names() -> Vec<&'static CStr> {
    let mut v = required_extension_names();
    v.push(ash::khr::surface::NAME);
    v.push(ash::khr::swapchain::NAME);
    // v.push(ash::extensions::ext::DebugUtils::name());
    // #[cfg(feature = "ray")]
    // v.push(ash::extensions::khr::ray_tracing::name());
    // #[cfg(feature = "mesh")]
    // v.push(ash::extensions::nv::MeshShader::name());
    v
}
