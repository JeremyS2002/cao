//! Buffer, BufferSlice + descriptions

use std::{
    borrow::Cow,
    mem::ManuallyDrop as Md,
    ops::{Bound, RangeBounds},
    ptr,
    sync::Arc,
};

use ash::vk;

use crate::error::*;

pub(crate) fn find_memory_type(
    req: vk::MemoryRequirements,
    ty: crate::MemoryType,
    physical: vk::PhysicalDeviceMemoryProperties,
) -> Result<u32, Error> {
    let mem_properties = ty.into();
    for (i, ty) in physical.memory_types.iter().enumerate() {
        if (req.memory_type_bits & (1 << i)) > 0 && ty.property_flags.contains(mem_properties) {
            return Ok(i as u32);
        }
    }
    panic!("ERROR Memory type requested is unavailable")
}

/// Describes a buffer
#[derive(Debug)]
pub struct BufferDesc {
    /// The name of the Buffer
    pub name: Option<String>,
    /// the size of the buffer
    pub size: u64,
    /// the usage of the buffer
    pub usage: crate::BufferUsage,
    /// the type of memory of the buffer
    pub memory: crate::MemoryType,
}

/// A Buffer
///
/// Contains arbitrary data on the gpu
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkBuffer.html>
pub struct Buffer {
    pub(crate) raw: Md<Arc<vk::Buffer>>,
    pub(crate) memory: Md<Arc<vk::DeviceMemory>>,
    pub(crate) size: u64,
    pub(crate) usage: crate::BufferUsage,
    pub(crate) mem_ty: crate::MemoryType,
    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) name: Option<String>,
}

impl std::hash::Hash for Buffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Buffer) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for Buffer {}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            raw: Md::new(Arc::clone(&self.raw)),
            memory: Md::new(Arc::clone(&self.memory)),
            size: self.size,
            usage: self.usage,
            mem_ty: self.mem_ty,
            device: Arc::clone(&self.device),
            name: self.name.clone(),
        }
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buffer id: {:?} name: {:?}", **self.raw, self.name)
    }
}

impl Buffer {
    pub unsafe fn raw_buffer(&self) -> vk::Buffer {
        **self.raw
    }

    pub unsafe fn raw_memory(&self) -> vk::DeviceMemory {
        **self.memory
    }
}

impl Buffer {
    /// Create a new Buffer
    pub fn new(device: &crate::Device, desc: &BufferDesc) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::trace!("GPU: Create Buffer, name {:?}", desc.name);

        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: desc.size,
            usage: desc.usage.into(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        let raw_result = unsafe { device.raw.create_buffer(&create_info, None) };

        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        let mem_req = unsafe { device.raw.get_buffer_memory_requirements(raw) };

        let mem_type = find_memory_type(mem_req, desc.memory, device.info.mem_properties)?;

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_req.size,
            memory_type_index: mem_type,
        };

        let memory_result = unsafe { device.raw.allocate_memory(&allocate_info, None) };

        let memory = match memory_result {
            Ok(m) => m,
            Err(e) => return Err(e.into()),
        };

        let bind_result = unsafe { device.raw.bind_buffer_memory(raw, memory, 0) };

        match bind_result {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }

        let s = Self {
            raw: Md::new(Arc::new(raw)),
            memory: Md::new(Arc::new(memory)),
            size: desc.size,
            usage: desc.usage,
            mem_ty: desc.memory,
            device: Arc::clone(&device.raw),
            name: desc.name.clone().map(|s| s.to_string()),
        };
        if let Some(name) = &desc.name {
            device.raw.set_buffer_name(&s, name)?;
        }
        device.raw.check_errors()?;
        Ok(s)
    }

    /// Create a new BufferSlice referencing self
    #[inline]
    pub fn slice_ref<'a, R: RangeBounds<u64>>(&'a self, range: R) -> BufferSlice<'a> {
        let offset = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };
        let size = match range.end_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v - 1,
            Bound::Unbounded => self.size - offset,
        };
        if size + offset > self.size {
            panic!("ERROR: Buffer slice size out of bounds")
        }
        BufferSlice {
            buffer: Cow::Borrowed(self),
            offset,
            size,
        }
    }

    /// Create a new BufferSlice containing a clone of self
    pub fn slice_owned<'a, 'b, R: RangeBounds<u64>>(&'b self, range: R) -> BufferSlice<'a> {
        let offset = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };
        let size = match range.end_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v - 1,
            Bound::Unbounded => self.size - offset,
        };
        if size + offset > self.size {
            panic!("ERROR: Buffer slice size out of bounds")
        }
        BufferSlice {
            buffer: Cow::Owned(self.clone()),
            offset,
            size,
        }
    }

    /// Create a new BufferSlice containting self
    pub fn into_slice<'a, R: RangeBounds<u64>>(self, range: R) -> BufferSlice<'a> {
        let offset = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };
        let size = match range.end_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v - 1,
            Bound::Unbounded => self.size - offset,
        };
        if size + offset > self.size {
            panic!("ERROR: Buffer slice size out of bounds")
        }
        BufferSlice {
            buffer: Cow::Owned(self),
            offset,
            size,
        }
    }

    /// Get the usage of the buffer
    pub fn usage(&self) -> crate::BufferUsage {
        self.usage
    }

    /// Get the memory type of the buffer
    pub fn mem_ty(&self) -> crate::MemoryType {
        self.mem_ty
    }

    /// get the size of the buffer
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the id of the buffer
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(**self.raw) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_buffer(raw, None);
            }
            let memory = Md::take(&mut self.memory);
            if let Ok(memory) = Arc::try_unwrap(memory) {
                self.device.free_memory(memory, None);
            }
        }
    }
}

/// A BufferSlice
///
/// Describes a region of a buffer represented by a size and offest
#[derive(Clone, Debug)]
pub struct BufferSlice<'a> {
    pub(crate) buffer: Cow<'a, Buffer>,
    pub(crate) offset: u64,
    pub(crate) size: u64,
}

impl std::cmp::PartialEq for BufferSlice<'_> {
    fn eq(&self, other: &BufferSlice<'_>) -> bool {
        // idk how the derive and Cow behave together so this might be the same as #[derive(PartialEq)]
        self.buffer.as_ref().eq(&other.buffer)
            && self.offset == other.offset
            && self.size == other.size
    }
}

impl std::cmp::Eq for BufferSlice<'_> {}

impl std::hash::Hash for BufferSlice<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.as_ref().hash(state);
        self.offset.hash(state);
        self.size.hash(state);
    }
}

impl<'a> BufferSlice<'a> {
    /// Take ownership of the slice by consuming the inner value
    /// If the slice is already owned then this should be a nop but in reality probably won't be
    pub fn into_owned(self) -> BufferSlice<'static> {
        BufferSlice {
            buffer: Cow::Owned(self.buffer.into_owned()),
            offset: self.offset,
            size: self.size,
        }
    }

    /// Take ownership of the slice by cloning the inner value and leaving the original in place
    pub fn as_owned(&self) -> BufferSlice<'static> {
        BufferSlice {
            buffer: Cow::Owned(self.buffer.as_ref().clone()),
            offset: self.offset,
            size: self.size,
        }
    }

    /// Get the buffer being sliced
    pub fn buffer(&'a self) -> &'a Buffer {
        self.buffer.as_ref()
    }

    /// Get the buffer being sliced
    pub fn cow_buffer<'b>(&'b self) -> &'b Cow<'a, Buffer> {
        &self.buffer
    }

    /// Get the offset of the slice
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the size of the slice
    pub fn size(&self) -> u64 {
        self.size
    }

    /// write binary data to the buffer
    /// the write happens instantly so there is no need for command buffers
    /// but the buffer must have been created with memory type host
    pub fn write(&self, data: &[u8]) -> Result<(), Error> {
        if self.buffer.mem_ty == crate::MemoryType::Device {
            panic!("ERROR: Can't write to buffer with memory type not visible to cpu")
        }

        if (data.len() as u64) < self.size {
            panic!("ERROR: Can't write to buffer with size less that slice size");
        }

        unsafe {
            let p_result = self.buffer.device.map_memory(
                **self.buffer.memory,
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            );

            let p = match p_result {
                Ok(p) => p,
                Err(e) => return Err(e.into()),
            };

            self.buffer.device.check_errors()?;

            p.copy_from_nonoverlapping(data.as_ptr() as *const _, self.size as usize);

            self.buffer.device.unmap_memory(**self.buffer.memory);
        }

        Ok(())
    }

    /// read binary data from the buffer
    /// the read happens instantly so there is no need for command buffers
    /// but the buffer must have been created with memory type Host
    pub fn read(&self, data: &mut [u8]) -> Result<(), Error> {
        if self.buffer.mem_ty == crate::MemoryType::Device {
            panic!("ERROR: Can't write to buffer with memory type not visible to cpu")
        }

        if (data.len() as u64) < self.size {
            panic!("ERROR: Can't read from buffer with size less that slice size");
        }

        unsafe {
            let p_result = self.buffer.device.map_memory(
                **self.buffer.memory,
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            );

            let p = match p_result {
                Ok(p) => p,
                Err(e) => return Err(e.into()),
            };

            self.buffer.device.check_errors()?;

            data.as_mut_ptr()
                .copy_from_nonoverlapping(p as *const _, self.size as usize);

            self.buffer.device.unmap_memory(**self.buffer.memory);
        }

        Ok(())
    }
}

/// Buffer Access
/// Describes how a buffer is accessed between cpu commands
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferAccessInfo<'a> {
    /// The buffer being accessed
    pub buffer: BufferSlice<'a>,
    /// How the buffer was accessed before
    pub src_access: crate::AccessFlags,
    /// How the buffer will be accessed after
    pub dst_access: crate::AccessFlags,
}
