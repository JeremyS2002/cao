//! A [`Swapchain`] is used to present images to a window
//!
//! A Swapchain is really a series of images so that while one is being shown on screen another can be drawn to

use std::cell::Cell;
// use std::ffi::c_void;
use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;

#[cfg(feature = "logging")]
use std::time::SystemTime;

use parking_lot::Mutex;

use ash::khr;
use ash::vk;
use vk::Handle;

use crate::error::*;
use crate::utils;

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
    /// the name of the swapchain, used for debugging
    pub name: Option<String>,
}

impl SwapchainDesc {
    /// Create a SwapchainDesc from a surface to match dimensions
    /// and pick a valid present_mode/format/image_count
    pub fn from_surface(surface: &crate::Surface, device: &crate::Device, vsync: bool) -> Result<Self, Error> {
        let info = surface.info(device)?;
        let mut texture_count = if info.min_images > 3 {
            info.min_images + 1
        } else {
            3
        };
        if info.max_images > 0 {
            texture_count = texture_count.min(info.max_images);
        }
        let mut present_mode = info.present_modes[0];
        if (!vsync) {
            for mode in info.present_modes {
                if mode == crate::PresentMode::Mailbox {
                    present_mode = mode;
                    break;
                }
                if mode == crate::PresentMode::Immediate {
                    present_mode = mode;
                }
            }
        }
        Ok(Self {
            format: info.formats[0],
            present_mode,
            texture_count,
            texture_usage: crate::TextureUsage::COLOR_OUTPUT,
            frames_in_flight: texture_count.min(2) as _,
            name: None,
        })
    }
}

/// TODO: consider making view field public?
pub struct SwapchainView<'a> {
    /// The inner from the swapchain this view is from
    pub(crate) inner: &'a SwapchainInner,
    /// The texture view that is currently acquired
    pub(crate) view: &'a crate::TextureView,
    pub(crate) fence_idx: usize,
    pub(crate) semaphore_idx:usize,
    /// The index of the view
    pub(crate) index: u32,
    /// Flags to store if the view has been rendered to
    /// and therefore if the semaphore should be waited on
    pub(crate) drawn: Cell<bool>,
}

impl<'a> std::fmt::Debug for SwapchainView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.view)?;
        writeln!(f, "Swapchain Index: {}", self.index)?;

        Ok(())
    }
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

#[derive(Debug, Clone)]
pub(crate) struct SwapchainSync {
    pub present_complete_semaphores: Vec<Arc<vk::Semaphore>>,
    pub rendering_complete_semaphores: Vec<Arc<vk::Semaphore>>,
    // need all the fences because user could choose not to render or present some frames??
    // pub acquire_complete_fences: Vec<Arc<vk::Fence>>,
    pub rendering_complete_fences: Vec<Arc<vk::Fence>>,
    // pub present_complete_fences: Vec<Arc<vk::Fence>>,
}

// impl std::clone::Clone for SwapchainSync {
//     fn clone(&self) -> Self {
//         Self {
//             present_complete_semaphores: self.present_complete_semaphores.clone(),
//             rendering_complete_semaphores: self.rendering_complete_semaphores.clone(),
//             acquire_complete_fences: self.acquire_complete_fences.clone(),
//             rendering_complete_fences: self.rendering_complete_fences.clone(),
//             present_complete_fences: self.present_complete_fences.clone(),
//         }
//     }
// }

pub(crate) struct SwapchainInner {
    pub utils: crate::utils::SwapchainUtils,
    pub raw: Md<Arc<Cell<vk::SwapchainKHR>>>,

    pub sync: SwapchainSync,

    pub surface: Md<Arc<vk::SurfaceKHR>>,
    pub surface_loader: khr::surface::Instance,

    pub device: Arc<crate::RawDevice>,
}

impl std::clone::Clone for SwapchainInner {
    fn clone(&self) -> Self {
        Self {
            utils: self.utils.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            sync: self.sync.clone(),
            surface: Md::new(Arc::clone(&self.surface)),
            surface_loader: self.surface_loader.clone(),
            device: Arc::clone(&self.device),
        }
    }
}

/// A Swapchain
///
/// A series of Textures that can be presented to the window
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSwapchainKHR.html>
pub struct Swapchain {
    pub(crate) inner: SwapchainInner,

    pub(crate) textures: Vec<crate::Texture>,
    pub(crate) views: Vec<crate::TextureView>,
    pub(crate) framebuffers: Mutex<Vec<crate::FramebufferKey>>,

    pub(crate) version: u64,
    pub(crate) queue: vk::Queue,

    // pub(crate) format: vk::SurfaceFormatKHR,
    // pub(crate) extent: vk::Extent2D,
    // pub(crate) pre_transform: vk::SurfaceTransformFlagsKHR,
    // pub(crate) present_mode: vk::PresentModeKHR,
    // pub(crate) image_count: u32,
    pub(crate) desc: SwapchainDesc,

    pub(crate) frames_in_flight: usize,
    pub(crate) fence_idx: Cell<usize>,
    pub(crate) semaphore_idx: Cell<usize>,
}

impl std::fmt::Debug for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Swapchain id: {:?}", self.inner.raw)
    }
}

impl std::cmp::PartialEq for Swapchain {
    fn eq(&self, other: &Swapchain) -> bool {
        self.inner.raw == other.inner.raw
    }
}

impl std::cmp::Eq for Swapchain {}

impl std::hash::Hash for Swapchain {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.raw.get().hash(state)
    }
}

impl Swapchain {
    pub unsafe fn raw_loader<'a>(&'a self) -> &'a khr::swapchain::Instance {
        &self.inner.utils.instance
    }

    pub unsafe fn raw_device<'a>(&'a self) -> &'a khr::swapchain::Device {
        &self.inner.utils.device
    }

    pub unsafe fn raw_swapchain(&self) -> vk::SwapchainKHR {
        self.inner.raw.get()
    }

    pub unsafe fn raw_surface(&self) -> vk::SurfaceKHR {
        **self.inner.surface
    }

    pub unsafe fn raw_surface_loader<'a>(&'a self) -> &'a khr::surface::Instance {
        &self.inner.surface_loader
    }

    pub fn textures<'a>(&'a self) -> &'a [crate::Texture] {
        &self.textures
    }

    pub fn textures_views<'a>(&'a self) -> &'a [crate::TextureView] {
        &self.views
    }

    pub unsafe fn raw_queue(&self) -> vk::Queue {
        self.queue
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
        #[cfg(feature = "logging")]
        log::trace!("gpu::Swapchain::new");

        let utils = crate::utils::SwapchainUtils::new(&device.raw);
        let raw= Self::create_raw(device, surface, desc, &utils)?;
        let (textures, views) = Self::create_frames(device, &utils, &raw, format, extent, desc.name.as_ref())?;
        let sync = Self::create_sync(device, desc.frames_in_flight, textures.len(), desc.name.as_ref())?;

        let s = Self {
            inner: SwapchainInner {
                utils,
                raw: Md::new(Arc::new(Cell::new(raw))),

                sync,

                surface: Md::new(Arc::clone(&surface.raw)),
                surface_loader: surface.loader.clone(),

                device: Arc::clone(&device.raw),
            },

            textures,
            views,
            framebuffers: Mutex::new(Vec::new()),

            // format,
            // extent,
            // pre_transform,
            // present_mode: desc.present_mode.into(),
            // image_count,
            desc: desc.clone(),

            version: 0,
            queue: device.queue,

            frames_in_flight: desc.frames_in_flight,
            fence_idx: Cell::new(0),
            semaphore_idx: Cell::new(0),
        };

        if let Some(name) = &s.desc.name {
            device.raw.set_swapchain_name(&s, name)?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    fn create_raw(
        device: &crate::Device,
        surface: &vk::SurfaceKHR,
        desc: &SwapchainDesc,
        utils: &utils::SwapchainUtils,
    ) -> Result<vk::SwapchainKHR, crate::Error> {
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

        let mut composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        
        let desired_alpha_flags = [
            vk::CompositeAlphaFlagsKHR::OPAQUE, 
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::INHERIT
        ];
        for &desired_alpha in &desired_alpha_flags {
            if caps.supported_composite_alpha.contains(desired_alpha) {
                composite_alpha = desired_alpha;
                break;
            }
        }

        let mut image_extent = caps.current_extent;
        image_extent.width = image_extent
            .width
            .min(caps.max_image_extent.width)
            .max(caps.min_image_extent.width);
        image_extent.height = image_extent
            .height
            .min(caps.max_image_extent.height)
            .max(caps.min_image_extent.height);

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            surface: **surface.raw,
            old_swapchain: vk::SwapchainKHR::null(),
            min_image_count: desc.texture_count.min(caps.max_image_count).max(caps.min_image_count),
            image_extent,
            image_format: format.format,
            image_color_space: format.color_space,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: pre_transform,
            composite_alpha,
            present_mode: desc.present_mode.into(),
            clipped: vk::TRUE,
            image_array_layers: 1,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            ..Default::default()
        };

        let swapchain_result = unsafe { utils.device.create_swapchain(&create_info, None) };

        let swapchain = match swapchain_result {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        return Ok(swapchain);
    }

    fn create_frames(
        device: &crate::Device,
        utils: &utils::SwapchainUtils,
        swapchain: &vk::SwapchainKHR,
        format: vk::SurfaceFormatKHR,
        extent: vk::Extent2D,
        swapchain_name: Option<&String>,
    ) -> Result<(Vec<crate::Texture>, Vec<crate::TextureView>), Error> {
        let raw_images_result = unsafe { utils.device.get_swapchain_images(*swapchain) };
        let raw_images = match raw_images_result {
            Ok(i) => i,
            Err(e) => return Err(e.into()),
        };

        let textures: Vec<crate::Texture> = raw_images
            .into_iter()
            .enumerate()
            .map(|(idx, i)| {
                let t = crate::Texture {
                    name: swapchain_name.map(|n| format!("{}.texture[{}]", n, idx)),
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

                if let Some(n) = &t.name {
                    device.raw.set_texture_name(&t, n)?;
                }

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
        swapchain_images: usize,
        swapchain_name: Option<&String>,
    ) -> Result<SwapchainSync, crate::Error> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            ..Default::default()
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut present_complete_semaphores = Vec::new();
        let mut rendering_complete_semaphores = Vec::new();
        // let mut acquire_complete_fences = Vec::new();
        let mut rendering_complete_fences = Vec::new();
        // let mut present_complete_fences = Vec::new();


        for idx in 0..swapchain_images {
            let present_complete_semaphore = unsafe { device.raw.create_semaphore(&semaphore_create_info, None)? };
            if let Some(n) = swapchain_name {
                device.raw.set_name(present_complete_semaphore.as_raw(), vk::ObjectType::SEMAPHORE, &format!("{}.present_complete_semaphore[{}]", n, idx))?;
            }
            present_complete_semaphores.push(Arc::new(present_complete_semaphore));

            let rendering_complete_semaphore = unsafe { device.raw.create_semaphore(&semaphore_create_info, None)? };
            if let Some(n) = swapchain_name {
                device.raw.set_name(rendering_complete_semaphore.as_raw(), vk::ObjectType::SEMAPHORE, &format!("{}.rendering_complete_semaphore[{}]", n, idx))?;
            }
            rendering_complete_semaphores.push(Arc::new(rendering_complete_semaphore));
        }

        for idx in 0..frames_in_flight {
            // let acquire_complete_fence = unsafe { device.raw.create_fence(&fence_create_info, None)? };
            // if let Some(n) = swapchain_name {
            //     device.raw.set_name(acquire_complete_fence.as_raw(), vk::ObjectType::FENCE, &format!("{}.acquire_complete_fence[{}]", n, idx))?;
            // }
            // acquire_complete_fences.push(Arc::new(acquire_complete_fence));

            let rendering_complete_fence = unsafe { device.raw.create_fence(&fence_create_info, None)? };
            if let Some(n) = swapchain_name {
                device.raw.set_name(rendering_complete_fence.as_raw(), vk::ObjectType::FENCE, &format!("{}.rendering_complete_fence[{}]", n, idx))?;
            }
            rendering_complete_fences.push(Arc::new(rendering_complete_fence));

            // let present_complete_fence = unsafe { device.raw.create_fence(&fence_create_info, None)? };
            // if let Some(n) = swapchain_name {
            //     device.raw.set_name(present_complete_fence.as_raw(), vk::ObjectType::FENCE, &format!("{}.present_complete_fence[{}]", n, idx))?;
            // }
            // present_complete_fences.push(Arc::new(present_complete_fence));
        }

        Ok(SwapchainSync {
            present_complete_semaphores, 
            rendering_complete_semaphores,
            // acquire_complete_fences,
            rendering_complete_fences,
            // present_complete_fences,
        })
    }

    pub fn recreate(&mut self, device: &crate::Device) -> Result<(), crate::Error> {
        #[cfg(feature = "logging")]
        log::trace!("Swapchain::recreate");

        // let mut start = SystemTime::now();

        self.inner.device.wait_idle()?;
        // let wait_result = unsafe {
        //     let fences = self.inner.sync.acquire_complete_fences
        //         .iter()
        //         .chain(&self.inner.sync.rendering_complete_fences)
        //         .chain(&self.inner.sync.present_complete_fences)
        //         .map(|f| **f)
        //         .collect::<Vec<_>>();
        //     self.inner.device.wait_for_fences(&fences, true, !0)
        // };
        // match wait_result {
        //     Ok(_) => (),
        //     Err(e) => return Err(e.into()),
        // }

        // log::info!("wait took                 : {:?}", start.elapsed().unwrap());
        // start = SystemTime::now();

        // log::info!("destroy images took       : {:?}", start.elapsed().unwrap());
        // start = SystemTime::now();

        let caps_result = unsafe {
            self.inner
                .surface_loader
                .get_physical_device_surface_capabilities(device.physical, **self.inner.surface)
        };

        let caps = match caps_result {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };

        // #[cfg(feature = "logging")]
        // log::trace!("Swapchain::recreate got extent {:?}", caps.current_extent);

        // let create_info = vk::SwapchainCreateInfoKHR {
        //     s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        //     p_next: ptr::null(),
        //     surface: **self.inner.surface,
        //     old_swapchain: self.inner.raw.get(),
        //     min_image_count: self.image_count,
        //     image_extent: caps.current_extent,
        //     image_format: self.format.format,
        //     image_color_space: self.format.color_space,
        //     image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        //     image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        //     pre_transform: self.pre_transform,
        //     composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        //     present_mode: self.present_mode,
        //     clipped: vk::TRUE,
        //     image_array_layers: 1,
        //     queue_family_index_count: 0,
        //     p_queue_family_indices: ptr::null(),
        //     flags: vk::SwapchainCreateFlagsKHR::empty(),
        //     ..Default::default()
        // };

        // let swapchain_result = unsafe { self.inner.utils.device.create_swapchain(&create_info, None) };

        // let swapchain = match swapchain_result {
        //     Ok(s) => s,
        //     Err(e) => return Err(e.into()),
        // };

        let (swapcahin, ) = Self::create_raw()        

        // log::info!("create new swapchain took : {:?}", start.elapsed().unwrap());
        // start = SystemTime::now();

        unsafe { 
            for texture in self.textures.drain(..) {
                drop(texture)
            }

            for view in self.views.drain(..) {
                drop(view)
            }

            for key in self.framebuffers.lock().drain(..) {
                    if let Some(framebuffer) = self.inner.device.framebuffers.write().remove(&key) {
                    if let Ok(framebuffer) = Arc::try_unwrap(framebuffer) {
                        self.inner.device.destroy_framebuffer(framebuffer, None);
                    }
                }
            }
            self.inner.utils.device.destroy_swapchain(self.inner.raw.get(), None); 
        }        
        
        // log::info!("destory swapchain took    : {:?}", start.elapsed().unwrap());
        // start = SystemTime::now();

        self.extent = caps.current_extent;

        let (textures, views) = Self::create_frames(
            device,
            &self.inner.utils,
            &swapchain,
            self.format,
            self.extent,
            self.name.as_ref()
        )?;

        // log::info!("create new textures took  : {:?}", start.elapsed().unwrap());
        // start = SystemTime::now();

        self.inner.raw.set(swapchain);
        self.textures = textures;
        self.views = views;
        self.version += 1;

        for fence  in self.inner.sync.rendering_complete_fences.drain(..) {
            if let Ok(fence) = Arc::try_unwrap(fence) {
                unsafe { self.inner.device.destroy_fence(fence, None) };
            }
        }

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        for idx in 0..self.frames_in_flight {
            let rendering_complete_fence = unsafe { device.raw.create_fence(&fence_create_info, None)? };
            if let Some(n) = self.name.as_ref() {
                device.raw.set_name(rendering_complete_fence.as_raw(), vk::ObjectType::FENCE, &format!("{}.rendering_complete_fence[{}]", n, idx))?;
            }
            self.inner.sync.rendering_complete_fences.push(Arc::new(rendering_complete_fence));
        }

        device.raw.check_errors()?;

        // log::info!("check errors took         : {:?}", start.elapsed().unwrap());
        // log::info!("");

        Ok(())
    }

    /// Get the frame at the index supplied
    ///
    /// Note: the frame won't be presentable, it hasn't been acquired this is just a reference to it
    /// Will panic if the index is out of bounds of the number of frames
    ///
    /// Valid Usage:
    /// frame must be less than the swapchains frames_in_flight
    /// The view must be acqured before submitting command buffers that draw to the frame
    /// The view returned hasn't been acquired so can't be presented
    /// Note: If implementing multiple frames in flight then the frame index must be different for each view otherwise they
    /// could cause errors about semaphores being used without waiting on them
    pub fn frame<'a>(&'a self, index: usize, fence_idx: usize, semaphore_idx: usize) -> SwapchainView<'a> {
        SwapchainView {
            inner: &self.inner,
            view: self.views.get(index).unwrap(),
            index: index as _,
            fence_idx,
            semaphore_idx,
            drawn: Cell::new(false),
        }
    }

    /// Acquire the next frame in the swapchain to be presented
    ///
    /// Returns Ok((frame, suboptimal)) or Err(e)
    pub fn acquire<'a>(&'a self, timeout: u64) -> Result<(SwapchainView<'a>, bool), crate::Error> {
        #[cfg(feature = "logging")]
        log::trace!("Swapchain::acquire fence_idx {}, semaphore_idx {}", self.fence_idx.get(), self.semaphore_idx.get());

        #[cfg(feature = "logging")]
        let mut start = SystemTime::now();

        //let start = std::time::Instant::now();
        // let frame = self.fence_idx.get();
        let fence_idx = self.fence_idx.get();
        let semaphore_idx = self.semaphore_idx.get();

        // wait for the previous acquisition of this frame to complete
        // wait for the previous rendering to this frame to complete
        unsafe {
            self.inner.device.wait_for_fences(&[
                // *self.inner.sync.acquire_complete_fences[fence_idx],
                *self.inner.sync.rendering_complete_fences[fence_idx],
                // *self.inner.sync.present_complete_fences[fence_idx],
            ], true, !0)?;
        };

        #[cfg(feature = "logging")]
        {
            log::info!("wait for fences took {:?}", start.elapsed().unwrap());
            start = SystemTime::now();
        }

        // reset as it will be signaled when we have acquired a fence
        unsafe { self.inner.device.reset_fences(&[*self.inner.sync.rendering_complete_fences[fence_idx]])?; }

        #[cfg(feature = "logging")]
        {
            log::info!("reset fence took {:?}", start.elapsed().unwrap());
            start = SystemTime::now();
        }

        let result = unsafe {
            self.inner.utils.device.acquire_next_image(
                self.inner.raw.get(),
                timeout,
                *self.inner.sync.present_complete_semaphores[semaphore_idx],
                // *self.inner.sync.acquire_complete_fences[fence_idx],
                vk::Fence::null(),
            )
        };

        #[cfg(feature = "logging")]
        {
            log::info!("acquire took {:?}", start.elapsed().unwrap());
            start = SystemTime::now();
        }

        let (index, suboptimal) = match result {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        #[cfg(feature = "logging")]
        log::trace!("Swapchain::acquire - got index {} suboptimal {}", index, suboptimal);

        self.inner.device.check_errors()?;

        #[cfg(feature = "logging")]
        log::info!("check errors took {:?}", start.elapsed().unwrap());
        #[cfg(feature = "logging")]
        log::info!("");

        Ok((
            SwapchainView {
                inner: &self.inner,
                view: self.views.get(index as usize).unwrap(),
                index: index as _,
                fence_idx,
                semaphore_idx,
                drawn: Cell::new(false),
            },
            suboptimal,
        ))
    }

    pub fn present(&self, view: SwapchainView<'_>) -> Result<bool, crate::Error> {
        #[cfg(feature = "logging")]
        let mut start = SystemTime::now();

        #[cfg(feature = "logging")]
        log::trace!("Swapchain::present - index {} fence_idx {}, semaphore_idx {}", view.index, view.fence_idx, view.semaphore_idx);

        if !view.drawn.get() {
            // unsafe { self.inner.device.reset_fences(&[*self.inner.sync.rendering_complete_fences[view.]])? };

            #[cfg(feature = "logging")]
            log::trace!("Swapchain::present - view hasn't been drawn");
            // why submit nothing?
            // the rest of the synchronisation logic for view expects
            // that the wait semaphore will be waited on and therefore reset
            // and the signal semaphore will be signaled so if the view is just acquired
            // and not drawn to this fixes that
            // TODO: Think about what happens if you draw to the same view twice
            // seems fine
            let stage = vk::PipelineStageFlags::BOTTOM_OF_PIPE;
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: Arc::as_ptr(
                    view.inner
                        .sync
                        .present_complete_semaphores
                        .get(view.semaphore_idx)
                        .unwrap(),
                ),
                p_wait_dst_stage_mask: &stage,
                command_buffer_count: 0,
                p_command_buffers: ptr::null(),
                signal_semaphore_count: 1,
                p_signal_semaphores: Arc::as_ptr(
                    view.inner
                        .sync
                        .rendering_complete_semaphores
                        .get(view.semaphore_idx)
                        .unwrap(),
                ),
                ..Default::default()
            };

            let submit_result = unsafe {
                self.inner
                    .device
                    // .queue_submit(self.queue, &[submit_info], **self.inner.fence)
                    .queue_submit(self.queue, &[submit_info], *self.inner.sync.rendering_complete_fences[view.fence_idx])
            };

            match submit_result {
                Ok(_) => (),
                Err(e) => return Err(e.into()),
            }

            #[cfg(feature = "logging")]
            {
                log::info!("fake render took : {:?}", start.elapsed().unwrap());
                start = SystemTime::now();
            }
        }

        #[cfg(feature = "logging")]
        log::trace!("present swapchain image {} semaphore_idx {}", view.index, view.semaphore_idx);

        // unsafe { self.inner.device.reset_fences(&[*self.inner.sync.present_complete_fences[view.sync_index]])? }

        // let fence_info = vk::SwapchainPresentFenceInfoEXT {
        //     s_type: vk::StructureType::SWAPCHAIN_PRESENT_FENCE_INFO_EXT,
        //     p_next: ptr::null(),
        //     swapchain_count: 1,
        //     p_fences: Arc::as_ptr(&self.inner.sync.present_complete_fences[view.sync_index]),
        //     ..Default::default()
        // };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            // p_next: &fence_info as *const _ as *const c_void,
            p_next: ptr::null(),
            p_image_indices: &view.index as _,
            p_swapchains: self.inner.raw.as_ptr(),
            swapchain_count: 1,
            p_wait_semaphores: Arc::as_ptr(
                &self.inner.sync.rendering_complete_semaphores[view.semaphore_idx],
            ),
            wait_semaphore_count: 1,
            p_results: ptr::null_mut(),
            ..Default::default()
        };

        let result = unsafe { self.inner.utils.device.queue_present(self.queue, &present_info) };

        #[cfg(feature = "logging")]
        {
            log::info!("queue_present took {:?}", start.elapsed().unwrap());
            log::info!("");
        }

        let fence_idx = (self.fence_idx.get() + 1) % self.frames_in_flight;
        self.fence_idx.set(fence_idx);

        let semaphore_idx = (self.semaphore_idx.get() + 1) % self.textures.len();
        self.semaphore_idx.set(semaphore_idx);

        match result {
            Ok(b) => {
                self.inner.device.check_errors()?;
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

impl Drop for SwapchainInner {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();
        unsafe {
            // let fences = self.sync.acquire_complete_fences
            //     .iter()
            //     .chain(&self.sync.rendering_complete_fences)
            //     .chain(&self.sync.present_complete_fences)
            //     .map(|f| **f)
            //     .collect::<Vec<_>>();
            let fences = self.sync.rendering_complete_fences.iter().map(|f| **f).collect::<Vec<_>>();
            self.device.wait_for_fences(&fences, true, !0).unwrap();
        }

        // for fence in self.sync.acquire_complete_fences.drain(..) {
        //     if let Ok(fence) = Arc::try_unwrap(fence) {
        //         unsafe {
        //             self.device.destroy_fence(fence, None);
        //         }
        //     }
        // }

        for fence in self.sync.rendering_complete_fences.drain(..) {
            if let Ok(fence) = Arc::try_unwrap(fence) {
                unsafe {
                    self.device.destroy_fence(fence, None);
                }
            }
        }

        // for fence in self.sync.present_complete_fences.drain(..) {
        //     if let Ok(fence) = Arc::try_unwrap(fence) {
        //         unsafe {
        //             self.device.destroy_fence(fence, None);
        //         }
        //     }
        // }

        let swapchain = unsafe { Md::take(&mut self.raw) };
        if let Ok(swapchain) = Arc::try_unwrap(swapchain) {
            unsafe { self.utils.device.destroy_swapchain(swapchain.get(), None) }
        }        

        for semaphore in self.sync.present_complete_semaphores.drain(..) {
            if let Ok(semaphore) = Arc::try_unwrap(semaphore) {
                unsafe {
                    self.device.destroy_semaphore(semaphore, None);
                }
            }
        }

        for semaphore in self.sync.rendering_complete_semaphores.drain(..) {
            if let Ok(semaphore) = Arc::try_unwrap(semaphore) {
                unsafe {
                    self.device.destroy_semaphore(semaphore, None);
                }
            }
        }

        let surface = unsafe { Md::take(&mut self.surface) };
        if let Ok(surface) = Arc::try_unwrap(surface) {
            unsafe {
                self.surface_loader.destroy_surface(surface, None);
            }
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        for texture in self.textures.drain(..) {
            drop(texture);
        }

        for view in self.views.drain(..) {
            drop(view);
        }
    }
}
