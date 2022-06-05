fn main() {
    env_logger::init();

    let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();

    let device = instance
        .create_device(&gpu::DeviceDesc {
            ..Default::default()
        })
        .unwrap();

    let mut line = String::new();
    println!("Enter how many to compute:");
    std::io::stdin().read_line(&mut line).unwrap();

    let len = line[..(line.len() - 1)].parse().unwrap();

    let mut data = vec![0u32; len].into_boxed_slice();

    let buffer = device
        .create_buffer(&gpu::BufferDesc {
            name: None,
            size: (len * std::mem::size_of::<u32>()) as u64,
            usage: gpu::BufferUsage::STORAGE,
            memory: gpu::MemoryType::Host,
        })
        .unwrap();

    let spv = gpu::include_spirv!("comp.spv");
    let shader = device
        .create_shader_module(&gpu::ShaderModuleDesc {
            name: None,
            entries: &[(gpu::ShaderStages::COMPUTE, "main")],
            spirv: &spv,
        })
        .unwrap();

    let descriptor_layout = device
        .create_descriptor_layout(&gpu::DescriptorLayoutDesc {
            name: None,
            entries: &[gpu::DescriptorLayoutEntry {
                ty: gpu::DescriptorLayoutEntryType::StorageBuffer { read_only: false },
                stage: gpu::ShaderStages::COMPUTE,
                count: std::num::NonZeroU32::new(1).unwrap(),
            }],
        })
        .unwrap();

    let descriptor_set = device
        .create_descriptor_set(&gpu::DescriptorSetDesc {
            name: None,
            layout: &descriptor_layout,
            entries: &[gpu::DescriptorSetEntry::Buffer(buffer.slice_ref(..))],
        })
        .unwrap();

    let layout = device
        .create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: None,
            descriptor_sets: &[&descriptor_layout],
            push_constants: &[],
        })
        .unwrap();

    let pipeline = device
        .create_compute_pipeline(&gpu::ComputePipelineDesc {
            name: None,
            layout: &layout,
            shader: &shader,
        })
        .unwrap();

    let mut command = device.create_command_buffer(None).unwrap();

    command.begin(true).unwrap();

    command.begin_compute_pass(&pipeline).unwrap();

    command
        .bind_descriptor(0, &descriptor_set, gpu::PipelineBindPoint::Compute, &layout)
        .unwrap();

    command.dispatch(len as _, 1, 1).unwrap();

    command.end().unwrap();

    command.submit().unwrap();

    command.wait(!0).unwrap();

    buffer
        .slice_ref(..)
        .read(bytemuck::cast_slice_mut(&mut data))
        .unwrap();

    for (i, v) in data.iter().enumerate() {
        println!("{}, {}", i, v);
    }
}
