//! Storage buffer utilities

/// A Wrapper for a storage buffer
#[derive(Debug, Clone)]
pub struct Storage<U: bytemuck::Pod> {
    /// the storage buffer, default usage COPY_SRC COPY_DST STORAGE
    pub buffer: gpu::Buffer,
    /// the number of elements in the buffer
    pub length: usize,
    /// marks the type of the buffer
    pub _marker: std::marker::PhantomData<U>,
}

impl<U: bytemuck::Pod> PartialEq for Storage<U> {
    fn eq(&self, other: &Storage<U>) -> bool {
        self.buffer == other.buffer
    }
}

impl<U: bytemuck::Pod> Eq for Storage<U> {}

impl<U: bytemuck::Pod> std::hash::Hash for Storage<U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
    }
}

impl<U: bytemuck::Pod> Storage<U> {
    /// Create a new storage from specific usage of buffer from Vec
    /// The data on the gpu won't be correct until the encoder is submitted
    pub fn from_vec_usage<'a>(
        encoder: &mut crate::CommandEncoder<'a>,
        device: &gpu::Device,
        data: Vec<U>,
        usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let storage_name = if let Some(name) = name {
            Some(format!("{}_buffer", name))
        } else {
            None
        };

        let buffer = device.create_buffer(&gpu::BufferDesc {
            size: std::mem::size_of::<U>() as u64 * data.len() as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::STORAGE
                | gpu::BufferUsage::UNIFORM
                | usage,
            memory: gpu::MemoryType::Device,
            name: storage_name,
        })?;

        let length = data.len();

        // max limit for update buffer
        if std::mem::size_of::<U>() * data.len() >= 65536 {
            println!("staging buffer");
            let staging_buffer = device.create_buffer(&gpu::BufferDesc {
                size: std::mem::size_of::<U>() as u64 * data.len() as u64,
                usage: gpu::BufferUsage::COPY_SRC,
                memory: gpu::MemoryType::Host,
                name: None,
            })?;

            staging_buffer
                .slice_ref(..)
                .write(bytemuck::cast_slice(&data))?;

            encoder.copy_buffer_to_buffer(staging_buffer.into_slice(..), buffer.slice_owned(..));
            println!("done");
        } else {
            let (ptr, len, cap) = data.into_raw_parts();
            let t = unsafe { Vec::from_raw_parts(ptr as *mut u8, len * 4, cap * 4) };
            encoder.push_command(crate::encoder::Command::UpdateBuffer {
                buffer: std::borrow::Cow::Owned(buffer.clone()),
                offset: 0,
                data: std::borrow::Cow::Owned(t),
            });
            println!("done");
        }

        Ok(Self {
            buffer,
            length,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a new storage from specific usage of buffer
    /// The data on the gpu won't be correct until the encoder is submitted
    pub fn from_usage<'a>(
        encoder: &mut crate::CommandEncoder<'a>,
        device: &gpu::Device,
        data: &'a [U],
        usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let storage_name = if let Some(name) = name {
            Some(format!("{}_buffer", name))
        } else {
            None
        };

        let buffer = device.create_buffer(&gpu::BufferDesc {
            size: std::mem::size_of::<U>() as u64 * data.len() as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::STORAGE
                | usage,
            memory: gpu::MemoryType::Device,
            name: storage_name,
        })?;

        // max limit for update buffer
        if std::mem::size_of::<U>() * data.len() >= 65536 {
            let staging_buffer = device.create_buffer(&gpu::BufferDesc {
                size: std::mem::size_of::<U>() as u64 * data.len() as u64,
                usage: gpu::BufferUsage::COPY_SRC,
                memory: gpu::MemoryType::Host,
                name: None,
            })?;

            staging_buffer
                .slice_ref(..)
                .write(bytemuck::cast_slice(data))?;

            encoder.copy_buffer_to_buffer(staging_buffer.into_slice(..), buffer.slice_owned(..));
        } else {
            encoder.push_command(crate::encoder::Command::UpdateBuffer {
                buffer: std::borrow::Cow::Owned(buffer.clone()),
                offset: 0,
                data: std::borrow::Cow::Borrowed(bytemuck::cast_slice(data)),
            });
        }

        Ok(Self {
            buffer,
            length: data.len(),
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a new storage
    /// The data on the gpu will only be correct when the encoder is submitted
    pub fn new<'a>(
        encoder: &mut crate::CommandEncoder<'a>,
        device: &gpu::Device,
        data: &'a [U],
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage(encoder, device, data, gpu::BufferUsage::empty(), name)
    }

    /// Create a new storage from Vec
    /// The data on the gpu will only be correct when the encoder is submitted
    pub fn from_vec<'a>(
        encoder: &mut crate::CommandEncoder<'a>,
        device: &gpu::Device,
        data: Vec<U>,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::from_vec_usage(encoder, device, data, gpu::BufferUsage::empty(), name)
    }

    /// Update the data on the gpu
    /// --------------------------
    ///
    /// The update will only be complete when the command encoder is submitted
    /// if the encoder is dropped before being submitted then no update will occur
    /// the data should have length >= self.length or this will return an error
    pub fn update_gpu<'a>(&'a self, encoder: &mut crate::CommandEncoder<'a>, data: &'a [U]) {
        encoder.update_buffer_ref(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    /// Update one index of the data on the gpu
    /// --------------------------
    ///
    /// The update will only be complete when the command encoder is submitted
    /// if the encoder is dropped before being submitted then no update will occur
    /// the data should have length >= self.length or this will return an error
    pub fn update_one_gpu<'a>(
        &'a self,
        encoder: &mut crate::CommandEncoder<'a>,
        index: usize,
        data: &'a U,
    ) {
        encoder.update_buffer_ref(
            &self.buffer,
            (index * std::mem::size_of::<U>()) as _,
            bytemuck::bytes_of(data),
        );
    }

    /// Update the data on the cpu
    /// after this the data will contain the data from the storage buffer
    /// the buffer should have length >= self.length or this will return and error
    pub fn update_cpu(
        &mut self,
        device: &gpu::Device,
        buffer: &mut gpu::CommandBuffer,
        data: &mut [U],
    ) -> Result<(), gpu::Error> {
        // if less than self then will write beyond valid memory of data
        if data.len() < self.length {
            panic!("ERROR: Cannot update cpu to data of length less than storage")
        }

        let staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: std::mem::size_of::<U>() as u64 * self.length as u64,
            usage: gpu::BufferUsage::COPY_SRC | gpu::BufferUsage::COPY_DST,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;

        let mut encoder = crate::CommandEncoder::new();
        encoder.copy_buffer_to_buffer(self.buffer.slice_ref(..), staging_buffer.slice_ref(..));

        encoder.submit(buffer, true)?;
        buffer.wait(!0)?;

        staging_buffer
            .into_slice(..)
            .read(bytemuck::cast_slice_mut(data))?;

        Ok(())
    }
}

impl<U: bytemuck::Pod> std::ops::Deref for Storage<U> {
    type Target = gpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
