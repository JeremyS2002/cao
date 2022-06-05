use std::cell::Cell;
use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;

use parking_lot::Mutex;

use ash::extensions::khr;
use ash::vk;

use crate::error::*;

/// Describes a swapchain
#[derive(Debug, Clone)]
pub struct SwapchainDesc {
    /// the format of the images in the swapchain
    pub format: crate::Format,
    /// the present mode of the swapchain
    pub present_mode: crate::PresentMode,
    /// the number of images in the swapchain
    pub texture_count: u32,
    /// the usage of the textures
    pub texture_usage: crate::TextureUsage,
    /// the maximum number of frames that are allowed
    /// to be being computed simultaniously
    pub frames_in_flight: usize,
}

impl SwapchainDesc {
    /// Create a SwapchainDesc from a surface to match dimensions
    /// and pick a valid present_mode/format/image_count
    pub fn from_surface(surface: &crate::Surface, device: &crate::Device) -> Result<Self, Error> {
        let info = surface.info(device)?;
        Ok(Self {
            format: info.formats[0],
            present_mode: info.present_modes[0],
            texture_count: if info.min_images < 3 {
                info.min_images
            } else {
                3
            },
            texture_usage: crate::TextureUsage::COLOR_OUTPUT,
            frames_in_flight: 1,
        })
    }
}

/// TODO: consider making view field public?
#[derive(Debug)]
pub struct SwapchainView<'a> {
    /// The texture view that is currently acquired
    pub(crate) view: &'a crate::TextureView,
    /// Semaphore to wait on before rendering to this frame
    pub(crate) wait_semaphore: vk::Semaphore,
    /// Semaphore to signal when rendering to this frame is complete
    pub(crate) signal_semaphore: vk::Semaphore,
    /// The index of the view
    pub(crate) index: u32,
    /// Flags to store if the view has been rendered to
    /// and therefore if the semaphore should be waited on
    pub(crate) drawn: Cell<bool>,
}

impl<'a> PartialEq for SwapchainView<'a> {
    fn eq(&self, _: &Self) -> bool {
        // swapchain views are always unique
        false
    }
}

#[derive(Debug)]
pub struct SwapchainInfo {
    /// The extent of the swapchain
    pub extent: crate::Extent2D,
    /// The format of the swapchain
    pub format: crate::Format,
}

/// A Swapchain
///
/// A series of Textures that can be presented to the window
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSwapchainKHR.html>
pub struct Swapchain {
    pub(crate) loader: khr::Swapchain,
    pub(crate) raw: vk::SwapchainKHR,

    pub(crate) textures: Vec<crate::Texture>,
    pub(crate) views: Vec<crate::TextureView>,
    pub(crate) framebuffers: Mutex<Vec<crate::FramebufferKey>>,

    pub(crate) surface: Md<Arc<vk::SurfaceKHR>>,
    pub(crate) surface_loader: Arc<khr::Surface>,

    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) version: u64,
    pub(crate) queue: vk::Queue,

    pub(crate) format: vk::SurfaceFormatKHR,
    pub(crate) extent: vk::Extent2D,
    pub(crate) pre_transform: vk::SurfaceTransformFlagsKHR,
    pub(crate) present_mode: vk::PresentModeKHR,
    pub(crate) image_count: u32,

    pub(crate) frames_in_flight: usize,
    pub(crate) frame: Cell<usize>,

    pub(crate) rendering_complete_semaphores: Vec<vk::Semaphore>,
    pub(crate) acquire_complete_semaphores: Vec<vk::Semaphore>,
}

impl std::fmt::Debug for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Swapchain id: {:?}", self.raw)
    }
}

impl std::cmp::PartialEq for Swapchain {
    fn eq(&self, other: &Swapchain) -> bool {
        self.raw == other.raw
    }
}

impl std::cmp::Eq for Swapchain {}

impl std::hash::Hash for Swapchain {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl Swapchain {
    /// Create a new swapchain for the surface
    ///
    /// NOTE: If the swapchain desc is invalid the properties of the swwapchain wlll be modified so that the creation can still take place
    pub fn new(
        device: &crate::Device,
        surface: &crate::Surface,
        desc: &SwapchainDesc,
    ) -> Result<Self, Error> {
        let loader = khr::Swapchain::new(&**device.raw.instance, &**device.raw);
        let (raw, format, extent, pre_transform) =
            Self::create_raw(device, surface, desc, &loader)?;
        let (textures, views) = Self::create_frames(device, &loader, &raw, format, extent)?;
        let (rendering_complete_semaphores, acquire_complete_semaphores) =
            Self::create_sync(device, desc.frames_in_flight)?;

        let image_count = textures.len() as u32;

        device.raw.check_errors()?;

        Ok(Self {
            loader,
            raw,

            textures,
            views,
            framebuffers: Mutex::new(Vec::new()),

            surface: Md::new(Arc::clone(&surface.raw)),
            surface_loader: Arc::clone(&surface.loader),

            format,
            extent,
            pre_transform,
            present_mode: desc.present_mode.into(),
            image_count,

            device: Arc::clone(&device.raw),
            version: 0,
            queue: device.queue,

            frames_in_flight: desc.frames_in_flight,
            frame: Cell::new(0),

            rendering_complete_semaphores,
            acquire_complete_semaphores,
        })
    }

    fn create_raw(
        device: &crate::Device,
        surface: &crate::Surface,
        desc: &SwapchainDesc,
        loader: &khr::Swapchain,
    ) -> Result<
        (
            vk::SwapchainKHR,
            vk::SurfaceFormatKHR,
            vk::Extent2D,
            vk::SurfaceTransformFlagsKHR,
        ),
        crate::Error,
    > {
        let raw_format = desc.format.into();

        let supported_formats_result = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device.physical, **surface.raw)
        };
        let supported_formats = match supported_formats_result {
            Ok(f) => f,
            Err(e) => return Err(e.into()),
        };

        let format_available = supported_formats.iter().find(|&f| f.format == raw_format);

        let format = if let Some(&f) = format_available {
            f
        } else {
            panic!("ERROR: Attempt to create swapchain with unsupported format")
        };

        let caps_result = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, **surface.raw)
        };

        let caps = match caps_result {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };

        let pre_transform = if caps
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            caps.current_transform
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            surface: **surface.raw,
            old_swapchain: vk::SwapchainKHR::null(),
            min_image_count: desc.texture_count,
            image_extent: caps.current_extent,
            image_format: format.format,
            image_color_space: format.color_space,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: pre_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: desc.present_mode.into(),
            clipped: vk::TRUE,
            image_array_layers: 1,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
        };

        let swapchain_result = unsafe { loader.create_swapchain(&create_info, None) };

        let swapchain = match swapchain_result {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        return Ok((swapchain, format, caps.current_extent, pre_transform));
    }

    fn create_frames(
        device: &crate::Device,
        loader: &khr::Swapchain,
        swapchain: &vk::SwapchainKHR,
        format: vk::SurfaceFormatKHR,
        extent: vk::Extent2D,
    ) -> Result<(Vec<crate::Texture>, Vec<crate::TextureView>), Error> {
        let raw_images_result = unsafe { loader.get_swapchain_images(*swapchain) };
        let raw_images = match raw_images_result {
            Ok(i) => i,
            Err(e) => return Err(e.into()),
        };

        let textures: Vec<crate::Texture> = raw_images
            .into_iter()
            .map(|i| {
                let t = crate::Texture {
                    name: None,
                    device: Arc::clone(&device.raw),
                    raw: Md::new(Arc::new(i)),
                    memory: None,
                    usage: crate::TextureUsage::empty(),
                    format: format.format.into(),
                    mem_ty: crate::MemoryType::Device,
                    mip_levels: 1,
                    initial_layout: crate::TextureLayout::SwapchainPresent,
                    dimension: crate::TextureDimension::D2(
                        extent.width,
                        extent.height,
                        crate::Samples::S1,
                    ),
                };

                crate::init_image_layout(&device, &t, crate::TextureLayout::SwapchainPresent)?;

                Ok(t)
            })
            .collect::<Result<_, crate::Error>>()?;

        let views = textures
            .iter()
            .map(|t| t.create_default_view())
            .collect::<Result<_, crate::Error>>()?;

        Ok((textures, views))
    }

    fn create_sync(
        device: &crate::Device,
        frames_in_flight: usize,
    ) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>), crate::Error> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let mut semaphores_1 = Vec::new();
        let mut semaphores_2 = Vec::new();
        // let mut fences = Vec::new();

        for _ in 0..frames_in_flight {
            let semaphore_1_res =
                unsafe { device.raw.create_semaphore(&semaphore_create_info, None) };

            let semaphore_1 = match semaphore_1_res {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            };

            semaphores_1.push(semaphore_1);

            let semaphore_2_res =
                unsafe { device.raw.create_semaphore(&semaphore_create_info, None) };

            let semaphore_2 = match semaphore_2_res {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            };

            semaphores_2.push(semaphore_2);

            // let fence_create_info = vk::FenceCreateInfo {
            //     s_type: vk::StructureType::FENCE_CREATE_INFO,
            //     p_next: ptr::null(),
            //     flags: vk::FenceCreateFlags::empty(),
            // };

            // let fence_res = unsafe {
            //     device.raw.create_fence(&fence_create_info, None)
            // };

            // let fence = match fence_res {
            //     Ok(f) => f,
            //     Err(e) => return Err(e.into())
            // };

            // fences.push(fence);
        }

        Ok((semaphores_1, semaphores_2))
    }

    pub fn recreate(&mut self, device: &crate::Device) -> Result<(), crate::Error> {
        self.device.wait_idle()?;

        // destroy previous resources
        for texture in self.textures.drain(..) {
            drop(texture)
        }

        for view in self.views.drain(..) {
            drop(view)
        }

        for key in self.framebuffers.lock().drain(..) {
            unsafe {
                if let Some(framebuffer) = self.device.framebuffers.write().remove(&key) {
                    self.device.destroy_framebuffer(framebuffer, None);
                }
            }
        }

        let caps_result = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(device.physical, **self.surface)
        };

        let caps = match caps_result {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            surface: **self.surface,
            old_swapchain: self.raw,
            min_image_count: self.image_count,
            image_extent: caps.current_extent,
            image_format: self.format.format,
            image_color_space: self.format.color_space,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: self.pre_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: self.present_mode,
            clipped: vk::TRUE,
            image_array_layers: 1,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
        };

        let swapchain_result = unsafe { self.loader.create_swapchain(&create_info, None) };

        let swapchain = match swapchain_result {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        self.extent = caps.current_extent;

        let (textures, views) =
            Self::create_frames(device, &self.loader, &swapchain, self.format, self.extent)?;

        self.raw = swapchain;
        self.textures = textures;
        self.views = views;
        self.version += 1;

        device.raw.check_errors()?;

        Ok(())
    }

    pub fn acquire<'a>(&'a self, timeout: u64) -> Result<(SwapchainView<'a>, bool), crate::Error> {
        //let start = std::time::Instant::now();
        let frame = self.frame.get();

        let result = unsafe {
            self.loader.acquire_next_image(
                self.raw,
                timeout,
                self.acquire_complete_semaphores[frame],
                // vk::Semaphore::null(),
                vk::Fence::null(),
                //self.acquire_complete_fences[frame],
            )
        };

        let (index, suboptimal) = match result {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        // let elapsed = start.elapsed().as_nanos();
        // let remaining = timeout.saturating_sub(elapsed as _);

        // let wait_result = unsafe {
        //     self.device.wait_for_fences(
        //         &[self.acquire_complete_fences[frame]],
        //         true,
        //         remaining
        //     )
        // };

        // match wait_result {
        //     Ok(_) => (),
        //     Err(e) => return Err(e.into())
        // }

        // let reset_result = unsafe {
        //     self.device.reset_fences(&[self.acquire_complete_fences[frame]])
        // };

        // match reset_result {
        //     Ok(_) => (),
        //     Err(e) => return Err(e.into())
        // }

        self.device.check_errors()?;

        Ok((
            SwapchainView {
                view: self.views.get(index as usize).unwrap(),
                index: index as _,
                wait_semaphore: self.acquire_complete_semaphores[frame],
                signal_semaphore: self.rendering_complete_semaphores[frame],
                drawn: Cell::new(false),
            },
            suboptimal,
        ))
    }

    pub fn present(&self, view: SwapchainView<'_>) -> Result<bool, crate::Error> {
        if !view.drawn.get() {
            // why submit nothing?
            // the rest of the synchronisation logic for view expects
            // that the wait semaphore will be waited on and therefore reset
            // and the signal semaphore will be signaled so if the view is just acquired
            // and not drawn to this fixes that
            // TODO: Think about what happens if you draw to the same view twice
            let stage = vk::PipelineStageFlags::BOTTOM_OF_PIPE;
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: &view.wait_semaphore as _,
                p_wait_dst_stage_mask: &stage,
                command_buffer_count: 0,
                p_command_buffers: ptr::null(),
                signal_semaphore_count: 1,
                p_signal_semaphores: &view.signal_semaphore as _,
            };

            let submit_result = unsafe {
                self.device
                    .queue_submit(self.queue, &[submit_info], vk::Fence::null())
            };

            match submit_result {
                Ok(_) => (),
                Err(e) => return Err(e.into()),
            }
        }

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            p_image_indices: &view.index as _,
            p_swapchains: &self.raw,
            swapchain_count: 1,
            p_wait_semaphores: &view.signal_semaphore as _,
            wait_semaphore_count: 1,
            p_results: ptr::null_mut(),
        };

        let result = unsafe { self.loader.queue_present(self.queue, &present_info) };

        match result {
            Ok(b) => {
                self.device.check_errors()?;
                let frame = (self.frame.get() + 1) % self.frames_in_flight;
                self.frame.set(frame);
                Ok(b)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn extent(&self) -> crate::Extent2D {
        self.extent.into()
    }

    pub fn format(&self) -> crate::Format {
        self.format.format.into()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();

        for texture in self.textures.drain(..) {
            drop(texture);
        }

        for view in self.views.drain(..) {
            drop(view);
        }

        unsafe { self.loader.destroy_swapchain(self.raw, None) }

        let surface = unsafe { Md::take(&mut self.surface) };
        if let Ok(surface) = Arc::try_unwrap(surface) {
            unsafe {
                self.surface_loader.destroy_surface(surface, None);
            }
        }

        for semaphore in self.acquire_complete_semaphores.drain(..) {
            unsafe {
                self.device.destroy_semaphore(semaphore, None);
            }
        }

        for semaphore in self.rendering_complete_semaphores.drain(..) {
            unsafe {
                self.device.destroy_semaphore(semaphore, None);
            }
        }
    }
}
