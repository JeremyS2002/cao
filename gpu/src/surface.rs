
use std::mem::ManuallyDrop as Md;
use std::sync::Arc;
use std::ptr;

use ash::vk;
use ash::khr;

use raw_window_handle::{ HasWindowHandle, HasDisplayHandle, RawWindowHandle, RawDisplayHandle };

pub(crate) struct SurfaceInner {
	pub raw: Md<vk::SurfaceKHR>,
	pub instance: khr::surface::Instance,
}

impl Drop for SurfaceInner {
	fn drop(&mut self) {
		unsafe {
			let raw = Md::take(&mut self.raw);
			self.instance.destroy_surface(raw, None);
		}
	}
}

#[derive(Clone)]
pub struct Surface {
	pub(crate) inner: Arc<SurfaceInner>,
	pub(crate) instance: Arc<crate::InstanceInner>,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Surface({:?})", *self.inner.raw)
    }
}

impl Surface {
	/// Create a new Surface from a window
    pub fn new<W: HasWindowHandle + HasDisplayHandle>(
        instance: &crate::Instance,
        window: &W,
    ) -> Result<Self, crate::Error> {
        let wh = window.window_handle().unwrap();
        let dh = window.display_handle().unwrap();

        #[cfg(feature = "logging")]
        log::trace!("gpu::Surface::new() - window {:?} display {:?}", wh, dh);

        match wh.as_raw() {
            #[cfg(target_os = "linux")]
            RawWindowHandle::Xlib(rwh) => unsafe { 
                if let RawDisplayHandle::Xlib(rdh) = dh.as_raw() {
                    Self::create_surface_from_xlib(instance, rwh, rdh) 
                } else {
                    panic!("mismatched window and display handles {:?} and {:?}", wh, dh);
                }
            },
            // #[cfg(target_os = "linux")]
            // RawWindowHandle::Xcb(h) => unsafe { Self::create_surface_from_xcb(instance, h) },
            // #[cfg(target_os = "linux")]
            // RawWindowHandle::Wayland(h) => unsafe {
            //     Self::create_surface_from_wayland(instance, h)
            // },
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

    #[cfg(target_os = "linux")]
    unsafe fn create_surface_from_xlib(
        instance: &crate::Instance,
        h: raw_window_handle::XlibWindowHandle,
        d: raw_window_handle::XlibDisplayHandle,
    ) -> Result<Self, crate::Error> {
        let xlib_loader = khr::xlib_surface::Instance::new(&*crate::VK_ENTRY, &instance.inner.raw);
        let info = vk::XlibSurfaceCreateInfoKHR {
            s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
            window: h.window as _,
            dpy: d.display.map(|x| x.as_ptr()).unwrap_or(ptr::null_mut()) as _,
            ..Default::default()
        };
        let raw = unsafe { xlib_loader.create_xlib_surface(&info, None)? };
        let surface_instance = khr::surface::Instance::new(&*crate::VK_ENTRY, &instance.inner.raw);
        Ok(Self {
            inner: Arc::new(SurfaceInner {
            	raw: Md::new(raw),
            	instance: surface_instance,
            }),
            instance: Arc::clone(&instance.inner),
        })
    }
}