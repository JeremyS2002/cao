use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::RawWindowHandle;

use crate::error::*;

/// Infomation about a surface
#[derive(Debug)]
pub struct SurfaceInfo {
    /// The minimum number of frames that the surface supports in a swapchain
    pub min_images: u32,
    /// The maximum number of frames that the surface supports in a swapchain
    pub max_images: u32,
    /// The minimum extent of a swapchain
    pub min_extent: crate::Extent2D,
    /// The maximum extent of a swapchain
    pub max_extent: crate::Extent2D,
    /// The current extent of the surface
    pub current_extent: crate::Extent2D,
    /// The supported formats of the surface (if empty then all are supported)
    pub formats: Vec<crate::Format>,
    /// The supported present modes of the surface (if empty then all are supported)
    pub present_modes: Vec<crate::PresentMode>,
}

/// A Surface
///
/// provides a bridge between the window and the swapchain
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSurfaceKHR.html>
pub struct Surface {
    pub(crate) raw: Md<Arc<vk::SurfaceKHR>>,
    pub(crate) loader: Arc<khr::Surface>,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Surface id {:?}", **self.raw)
    }
}

impl Surface {
    /// Create a new Surface from a window
    pub fn new<W: HasRawWindowHandle>(
        instance: &crate::Instance,
        window: &W,
    ) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create Surface");

        match window.raw_window_handle() {
            #[cfg(target_os = "linux")]
            RawWindowHandle::Xlib(h) => unsafe { Self::create_surface_from_xlib(instance, h) },
            #[cfg(target_os = "linux")]
            RawWindowHandle::Xcb(h) => unsafe { Self::create_surface_from_xcb(instance, h) },
            #[cfg(target_os = "linux")]
            RawWindowHandle::Wayland(h) => unsafe {
                Self::create_surface_from_wayland(instance, h)
            },
            #[cfg(target_os = "android")]
            RawWindowHandle::Android(h) => unsafe {
                Self::create_surface_from_android(instance, h)
            },
            #[cfg(target_os = "windows")]
            RawWindowHandle::Windows(h) => unsafe {
                Self::create_surface_from_windows(instance, h)
            },
            #[cfg(target_os = "macos")]
            RawWindowHandle::MacOS(h) => {}
            h => panic!("ERROR: Can't create surface from window of type {:?}", h),
        }
    }

    /// Get infomation about the surface
    pub fn info(&self, device: &crate::Device) -> Result<SurfaceInfo, Error> {
        let raw_formats_result = unsafe {
            self.loader
                .get_physical_device_surface_formats(device.physical, **self.raw)
        };
        let raw_formats = match raw_formats_result {
            Ok(f) => f,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let formats = raw_formats
            .iter()
            .map(|f| f.format.into())
            .collect::<Vec<crate::Format>>();
        let raw_present_modes_result = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(device.physical, **self.raw)
        };

        let raw_present_modes = match raw_present_modes_result {
            Ok(p) => p,
            Err(e) => return Err(ExplicitError(e).into()),
        };

        let present_modes = raw_present_modes
            .iter()
            .map(|m| (*m).into())
            .collect::<Vec<crate::PresentMode>>();

        let caps_result = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(device.physical, **self.raw)
        };
        let caps = match caps_result {
            Ok(c) => c,
            Err(e) => return Err(ExplicitError(e).into()),
        };

        device.raw.check_errors()?;

        Ok(SurfaceInfo {
            current_extent: caps.current_extent.into(),
            min_extent: caps.min_image_extent.into(),
            max_extent: caps.max_image_extent.into(),
            min_images: caps.min_image_count,
            max_images: if caps.max_image_count == 0 {
                // there is no max
                std::u32::MAX
            } else {
                caps.max_image_count
            },
            formats,
            present_modes,
        })
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_surface_from_xlib(
        instance: &crate::Instance,
        h: raw_window_handle::unix::XlibHandle,
    ) -> Result<Self, Error> {
        let xlib_loader = khr::XlibSurface::new(&*crate::VK_ENTRY, &**instance.raw);
        let info = vk::XlibSurfaceCreateInfoKHR {
            s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
            window: h.window,
            dpy: h.display as *mut vk::Display,
        };
        let surface_result = xlib_loader.create_xlib_surface(&info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_surface_from_xcb(
        instance: &crate::Instance,
        h: raw_window_handle::unix::XcbHandle,
    ) -> Result<Self, Error> {
        let xcb_loader = khr::XcbSurface::new(&*crate::VK_ENTRY, &**instance.raw);
        let info = vk::XcbSurfaceCreateInfoKHR {
            s_type: vk::StructureType::XCB_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::XcbSurfaceCreateFlagsKHR::empty(),
            window: h.window,
            connection: h.connection,
        };
        let surface_result = xcb_loader.create_xcb_surface(&info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_surface_from_wayland(
        instance: &crate::Instance,
        h: raw_window_handle::unix::WaylandHandle,
    ) -> Result<Self, Error> {
        let wayland_loader = khr::WaylandSurface::new(&*crate::VK_ENTRY, &**instance.raw);
        let info = vk::WaylandSurfaceCreateInfoKHR {
            s_type: vk::StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::WaylandSurfaceCreateFlagsKHR::empty(),
            display: h.display,
            surface: h.surface,
        };
        let surface_result = wayland_loader.create_wayland_surface(&info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }

    #[cfg(target_os = "android")]
    unsafe fn create_surface_from_android(
        instance: &crate::Instance,
        h: raw_window_handle::android::AndroidHandle,
    ) -> Result<Self, Error> {
        let a_loader = khr::AndroidSurface::new(&*crate::VK_ENTRY, &**instance.raw);
        let info = vk::AndroidSurfaceCreateInfoKHR {
            s_type: vk::StructureType::ANDROID_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::AndroidSurfaceCreateFlagsKHR::empty(),
            window: h.a_native_window,
        };
        let surface_result = a_loader.create_android_surface(&info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }

    #[cfg(windows)]
    unsafe fn create_surface_from_windows(
        instance: &crate::Instance,
        h: raw_window_handle::windows::WindowsHandle,
    ) -> Result<Self, Error> {
        let win_loader = khr::Win32Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        let info = vk::Win32SurfaceCreateInfoKHR::builder()
            .flags(vk::Win32SurfaceCreateFlagsKHR::empty())
            .hinstance(h.hinstance)
            .hwnd(h.hwnd);
        let surface_result = win_loader.create_win32_surface(&info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }

    #[cfg(target_os = "macos")]
    unsafe fn create_surface_from_macos(
        instance: &crate::Instance,
        h: raw_window_handle::macos::MacOSHandle,
    ) -> Result<Self, error::SurfaceCreation> {
        use cocoa::appkit::{NSView, NSWindow};
        use cocoa::base::id as cocoa_id;
        use metal::CoreAnimationLayer;
        use objc::runtime::YES;
        use std::mem;
        use std::os::raw::c_void;

        let wnd: cocoa_id = mem::transmute(h.ns_window);
        let layer = CoreAnimationlayer::new();

        layer.set_edge_antialiasing_mask(0);
        layer.set_presents_with_transaction(false);
        layer.remove_all_animations();

        let view = wnd.contentView();

        layer.set_contents_scale(view.backingScaleFactor());
        view.setLayer(mem::transmute(layer.as_ref()));
        view.setWantsLayer(YES);

        let create_info = vk::MacOSSurfaceCreateInfoMVK {
            s_type: vk::StructureType::MACOS_SURFACE_CREATE_INFO_M,
            p_next: ptr::null(),
            flags: Default::default(),
            p_view: window.ns_view() as *const c_void,
        };

        let macos_surface_loader = MacOSSurface::new(entry, instance);
        let surface_result = macos_surface_loader.create_macos_surface_mvk(&create_info, None);
        let surface = match surface_result {
            Ok(s) => s,
            Err(e) => return Err(ExplicitError(e).into()),
        };
        let loader = khr::Surface::new(&*crate::VK_ENTRY, &**instance.raw);
        Ok(Self {
            raw: Md::new(Arc::new(surface)),
            loader: Arc::new(loader),
        })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.loader.destroy_surface(raw, None);
            }
        }
    }
}
