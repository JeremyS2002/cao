//! Uniform buffer utilities

/// A uniform that is used to send T into shaders
/// as well as being Pod and Zeroable T should be repr(C) for the binary data
/// to be interpreted correctly in the shaders
#[derive(Debug, Clone)]
pub struct Uniform<U: bytemuck::Pod> {
    /// the uniform buffer, default usage: COPY_SRC COPY_DST UNIFORM
    pub buffer: gpu::Buffer,
    /// the data of the uniform
    pub data: U,
}

impl<U: bytemuck::Pod> PartialEq for Uniform<U> {
    fn eq(&self, other: &Uniform<U>) -> bool {
        self.buffer == other.buffer
    }
}

impl<U: bytemuck::Pod> Eq for Uniform<U> {}

impl<U: bytemuck::Pod> std::hash::Hash for Uniform<U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
    }
}

impl<U: bytemuck::Pod + Default> Uniform<U> {
    /// Create a new uniform using the default value of U
    /// The data on the gpu will on ly be correct when the encoder is submitted
    pub fn default(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::new(encoder, device, U::default(), name)
    }
}

impl<U: bytemuck::Pod> Uniform<U> {
    /// Create a new uniform from usage
    /// The data on the gpu will only be correct when the encoder is submitted
    pub fn from_usage(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        data: U,
        usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let uniform_name = if let Some(name) = name {
            Some(format!("{}_buffer", name))
        } else {
            None
        };

        let buffer = device.create_buffer(&gpu::BufferDesc {
            size: std::mem::size_of::<U>() as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::UNIFORM
                | usage,
            memory: gpu::MemoryType::Device,
            name: uniform_name,
        })?;

        let bytes = bytemuck::bytes_of(&data).to_vec();

        encoder.update_buffer_owned(buffer.clone(), 0, bytes);

        Ok(Self { data, buffer })
    }

    /// Create a new uniform
    /// The data on the gpu will only be correct when the encoder is submitted
    pub fn new(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        data: U,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage(encoder, device, data, gpu::BufferUsage::empty(), name)
    }

    /// Update the data on the gpu
    /// --------------------------
    ///
    /// The update will only be complete when the command encoder is submitted
    /// if the encoder is dropped before being submitted then no update will occur
    pub fn update_gpu_ref<'a>(&'a self, encoder: &mut crate::CommandEncoder<'a>) {
        encoder.update_buffer_ref(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }

    /// Update the data on the gpu
    /// --------------------------
    ///
    /// The update will only be complete when the command encoder is submitted
    /// if the encoder is dropped before being submitted then no update will occur
    pub fn update_gpu_owned<'a>(&self, encoder: &mut crate::CommandEncoder<'a>) {
        encoder.update_buffer_owned(
            self.buffer.clone(),
            0,
            bytemuck::bytes_of(&self.data).to_vec(),
        );
    }

    /// Update the data on the cpu
    pub fn update_cpu(
        &mut self,
        device: &gpu::Device,
        buffer: &mut gpu::CommandBuffer,
    ) -> Result<(), gpu::Error> {
        let staging_buffer = device.create_buffer(&gpu::BufferDesc {
            name: None,
            size: std::mem::size_of::<U>() as u64,
            memory: gpu::MemoryType::Host,
            usage: gpu::BufferUsage::COPY_DST,
        })?;
        let mut encoder = crate::CommandEncoder::new(&device);
        encoder.copy_buffer_to_buffer(self.buffer.slice_ref(..), staging_buffer.slice_ref(..));

        buffer.wait(!0)?;
        encoder.submit(buffer, true)?;
        buffer.wait(!0)?;

        staging_buffer
            .slice_ref(..)
            .read(bytemuck::bytes_of_mut(&mut self.data))?;

        Ok(())
    }
}
