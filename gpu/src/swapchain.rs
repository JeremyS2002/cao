
use crate::DescType;
use gpu_derive::DescType;

use std::sync::Arc;
use std::ptr;
use std::cell::Cell;

use ash::vk::Handle;
use ash::{ vk, khr };


/// Describes a swapchain
#[derive(Debug, Clone, DescType)]
pub struct SwapchainDesc<'a> {
	#[skip_info]
	pub surface: &'a crate::Surface,
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

pub struct SwapchainTexture {
	pub(crate) view: crate::TextureView,
	pub(crate) texture: crate::Texture,
}

pub struct SwapchainSync {
	pub(crate) acquire: vk::Semaphore,
	pub(crate) render: vk::Semaphore,
	pub(crate) in_flight: vk::Fence,
}

pub(crate) struct SwapchainLoader {
	pub(crate) instance: khr::swapchain::Instance,
	pub(crate) device: khr::swapchain::Device,
}

// TODO - if Cell then use Rc if Arc then use Mutex
pub(crate) struct SwapchainInner {
	pub(crate) info: SwapchainInfo,
	pub(crate) loader: SwapchainLoader,
	pub(crate) raw: Cell<vk::SwapchainKHR>,

	pub(crate) present_queue: vk::Queue,

	pub(crate) acquired: Cell<bool>,
	pub(crate) frame: Cell<usize>,
	pub(crate) texture: Cell<usize>,
	pub(crate) sync: Vec<SwapchainSync>,

	pub(crate) surface: Arc<crate::SurfaceInner>,
	pub(crate) device: Arc<crate::DeviceInner>,
}

impl Drop for SwapchainInner {
	fn drop(&mut self) {
		unsafe { 
			self.device.logical.device_wait_idle().unwrap();
		}

		unsafe {
			self.loader.device.destroy_swapchain(self.raw.get(), None);
		}
	}
}

pub struct SwapchainFrame<'a> {
	pub(crate) swapchain: &'a Swapchain,
	pub(crate) frame_idx: usize,
	pub(crate) texture_idx: u32,
}

impl<'a> SwapchainFrame<'a> {
	pub(crate) fn get_texture(&self) -> &'a crate::Texture {
		&self.swapchain.textures.get(self.texture_idx as usize).unwrap().texture
	}

	pub(crate) fn get_view(&self) -> &'a crate::TextureView {
		&self.swapchain.textures.get(self.texture_idx as usize).unwrap().view
	}

	pub(crate) fn get_sync(&self) -> &'a SwapchainSync {
		self.swapchain.inner.sync.get(self.frame_idx).unwrap()
	}
}

pub struct Swapchain {
	pub(crate) textures: Vec<SwapchainTexture>,
	pub(crate) inner: Arc<SwapchainInner>,
}

impl Swapchain {
	pub fn new(device: &crate::Device, desc: &SwapchainDesc<'_>) -> Result<Self, crate::Error> {
		let mut extent = crate::Extent2D { width: 0, height: 0 };
		let loader = Self::create_loader(device);		
		let raw = Self::create_raw(device, desc, &mut extent, &loader)?;
		let present_queue = Self::select_queue(device, desc)?;
		let info = desc.to_info();
		let sync = Self::create_sync(device, &info)?;

		let inner = Arc::new(SwapchainInner {
			info,
			loader,
			raw: Cell::new(raw),
			present_queue,
			frame: Cell::new(0),
			texture: Cell::new(0),
			sync,
			surface: Arc::clone(&desc.surface.inner),
			device: Arc::clone(&device.inner),
		});

		let textures = Self::create_textures(device, &inner, extent)?;

		let swapchain = Swapchain {
			textures,
			inner,
		};

		device.check_errors()?;

		Ok(swapchain)
	}

	pub(crate) fn create_loader(device: &crate::Device) -> SwapchainLoader {
		let instance = khr::swapchain::Instance::new(&*&crate::VK_ENTRY, &*device.inner.instance.raw);
		let device = khr::swapchain::Device::new(&*device.inner.instance.raw, &device.inner.logical);

		SwapchainLoader {
			instance,
			device,
		}
	}

	pub(crate) fn create_raw(device: &crate::Device, desc: &SwapchainDesc, extent: &mut crate::Extent2D, loader: &SwapchainLoader) -> Result<vk::SwapchainKHR, crate::Error> {

		let raw_format: vk::Format = desc.format.into();

		let supported_formats = unsafe { desc.surface.inner.loader.get_physical_device_surface_formats(device.inner.physical, desc.surface.inner.raw)? };	
		let Some(format) = supported_formats.iter().find(|&f| f.format == raw_format) else {
			panic!("ERROR: Attempt to create swapchain with unsupported format {:?}", desc.format);
		};

		let caps = unsafe { desc.surface.inner.loader.get_physical_device_surface_capabilities(device.inner.physical, desc.surface.inner.raw)? };

		let pre_transform = if caps.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
			vk::SurfaceTransformFlagsKHR::IDENTITY
		} else {
			caps.current_transform
		};

		let mut composite_alpha = vk::CompositeAlphaFlagsKHR::empty();
		let desired_alpha_list = [
            vk::CompositeAlphaFlagsKHR::OPAQUE, 
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::INHERIT
        ];
		for &desired_alpha in &desired_alpha_list {
			if caps.supported_composite_alpha.contains(desired_alpha) {
				composite_alpha = desired_alpha;
				break;
			}
		}

		let mut image_extent = caps.current_extent;
		image_extent.width = image_extent.width
            .min(caps.max_image_extent.width)
            .max(caps.min_image_extent.width);
        image_extent.height = image_extent.height
            .min(caps.max_image_extent.height)
            .max(caps.min_image_extent.height);

        *extent = image_extent.into();

        let min_images = desc.texture_count.min(caps.max_image_count).max(caps.min_image_count);

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            surface: desc.surface.inner.raw,
            old_swapchain: vk::SwapchainKHR::null(),
            min_image_count: min_images,
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

        let raw = unsafe { loader.device.create_swapchain(&create_info, None)? };

        if let Some(name) = &desc.name {
        	device.set_name(raw.as_raw(), vk::ObjectType::SWAPCHAIN_KHR, name)?;
        }

        Ok(raw)
	}

	pub(crate) fn create_textures(device: &crate::Device, inner: &Arc<SwapchainInner>, extent: crate::Extent2D) -> Result<Vec<SwapchainTexture>, crate::Error> {
		let images = unsafe { inner.loader.device.get_swapchain_images(inner.raw.get())? };

		let mut frames = Vec::with_capacity(images.len());

		for (index, &image) in images.iter().enumerate() {
			let texture_name = inner.info.name.as_ref().map(|n| format!("{}_texture_{}", n, index));

			if let Some(name) = &texture_name {
				device.set_name(image.as_raw(), vk::ObjectType::IMAGE, name)?;
			}

			let texture = crate::Texture {
				inner: Arc::new(crate::TextureInner {
					info: crate::TextureInfo {
						format: inner.info.format,
						usage: crate::TextureUsage::empty(),
						dimension: crate::TextureDimension::D2(extent.width, extent.height, crate::Samples::S1),
						mip_levels: std::num::NonZeroU32::new(1).unwrap(),
						memory: crate::MemoryType::Device,
						layout: crate::TextureLayout::Undefined,
						name: texture_name,
					},
					backing: crate::TextureBacking::Swapchain(Arc::clone(inner)),
					raw: image,
					device: Arc::clone(&device.inner),
				})
			};

			let view = texture.create_default_view()?;

			frames.push(SwapchainTexture {
				view,
				texture,
			})
		}

		Ok(frames)
	}

	pub(crate) fn create_sync(device: &crate::Device, info: &SwapchainInfo) -> Result<Vec<SwapchainSync>, crate::Error> {
		let mut sync = Vec::with_capacity(info.frames_in_flight);

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

		for _ in 0..info.frames_in_flight {
			let acquire = unsafe { device.inner.logical.create_semaphore(&semaphore_create_info, None)? };
			let render = unsafe { device.inner.logical.create_semaphore(&semaphore_create_info, None)? };
			let in_flight = unsafe { device.inner.logical.create_fence(&fence_create_info, None)? };

			sync.push(SwapchainSync {
				acquire,
				render,
				in_flight,
			});
		}

		Ok(sync)
	}

	pub(crate) fn select_queue(device: &crate::Device, desc: &SwapchainDesc) -> Result<vk::Queue, crate::Error> {
		let family = device.select_queue_family_for_surface(desc.surface)?;
		let queue = unsafe { device.inner.logical.get_device_queue(family as u32, 0) };
		Ok(queue)
	}

	pub fn acquire<'a>(&'a self, timeout: u64) -> Result<(SwapchainFrame<'a>, bool), crate::Error> {
		let frame_idx = self.inner.frame.get();
		let sync = self.inner.sync.get(frame_idx as usize).unwrap();
		
		// wait until the previous in flight frame is finished
		unsafe { self.inner.device.logical.wait_for_fences(&[sync.in_flight], true, timeout)? };

		let (texture_idx, suboptimal) = unsafe { self.inner.loader.device.acquire_next_image(self.inner.raw.get(), timeout, sync.acquire, vk::Fence::null())? };

		let frame = SwapchainFrame {
			swapchain: self,
			frame_idx,
			texture_idx: texture_idx,
		};

		self.inner.device.check_errors()?;

		Ok((frame, suboptimal))
	}

	pub fn present(&self) -> Result<bool, crate::Error> {

		// let sync = view.get_sync();

		let present_info = vk::PresentInfoKHR {
			s_type: vk::StructureType::PRESENT_INFO_KHR,
			p_next: ptr::null(),
			p_image_indices: &view.texture_idx as _,
			swapchain_count: 1,
			p_swapchains: self.inner.raw.as_ptr() as _,
			wait_semaphore_count: 1,
			p_wait_semaphores: &sync.render as _,
			p_results: ptr::null_mut(),
			..Default::default()
		};

		let suboptimal = unsafe { self.inner.loader.device.queue_present(self.inner.present_queue, &present_info)? };

		self.inner.frame.set((self.inner.frame.get() + 1) % self.inner.info.frames_in_flight);

		self.inner.device.check_errors()?;

		Ok(suboptimal)
	}

	pub fn recreate(&self, device: &crate::Device) -> Result<(), crate::Error> {


		todo!();
	}
}