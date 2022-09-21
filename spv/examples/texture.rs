
use std::borrow::Cow;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() {
    let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let surface = instance.create_surface(&window).unwrap();
    let device = instance
        .create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            ..Default::default()
        })
        .unwrap();

    let mut swapchain = device
        .create_swapchain(
            &surface,
            &gpu::SwapchainDesc::from_surface(&surface, &device).unwrap(),
        )
        .unwrap();

    let mut resized = false;

    let vertices = vec![
        Vertex {
            pos: [-0.5, -0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            pos: [-0.5, 0.5],
            uv: [0.0, 1.0],
        },
        Vertex {
            pos: [0.5, 0.5],
            uv: [1.0, 1.0],
        },
        Vertex {
            pos: [0.5, -0.5],
            uv: [1.0, 0.0],
        },
    ];

    let vertex_buffer = device
        .create_buffer(&gpu::BufferDesc {
            name: None,
            size: (std::mem::size_of::<Vertex>() * vertices.len()) as _,
            usage: gpu::BufferUsage::VERTEX,
            memory: gpu::MemoryType::Host,
        })
        .unwrap();

    vertex_buffer
        .slice_ref(..)
        .write(bytemuck::cast_slice(&vertices))
        .unwrap();

    let indices = vec![0u32, 1, 2, 2, 3, 0];

    let index_buffer = device
        .create_buffer(&gpu::BufferDesc {
            name: None,
            size: (std::mem::size_of::<u32>() * indices.len()) as _,
            usage: gpu::BufferUsage::INDEX,
            memory: gpu::MemoryType::Host,
        })
        .unwrap();

    index_buffer
        .slice_ref(..)
        .write(bytemuck::cast_slice(&indices))
        .unwrap();

    let vertex_spv = {
        let b = spv::Builder::new();

        let in_pos = b.in_vec2(0, "in_pos");
        let in_uv = b.in_vec2(1, "in_uv");

        let vk_pos = b.vk_position();
        let out_uv = b.out_vec2(0, "out_uv");

        b.entry(spv::Stage::Vertex, "main", || {
            let pos = in_pos.load();
            vk_pos.store(b.vec4(pos.x(), pos.y(), 0.0, 1.0));

            out_uv.store(in_uv.load());
        });
        
        b.compile()
    };

    let vertex_shader = device
        .create_shader_module(&gpu::ShaderModuleDesc {
            name: None,
            entries: &[(gpu::ShaderStages::VERTEX, "main")],
            spirv: &vertex_spv,
        })
        .unwrap();

    let fragment_spv = {
        let b = spv::Builder::new();

        let in_uv = b.in_vec2(0, "in_uv");

        let out_col = b.out_vec3(0, "out_color");

        let texture = b.texture2d(0, 0, Some("u_color"));
        let sampler = b.sampler(0, 1, Some("u_sampler"));

        b.entry(spv::Stage::Fragment, "main", || {
            let uv = in_uv.load();
            let combined = spv::combine(&texture, sampler);
            let col = spv::sample(&combined, uv);
            out_col.store(col.xyz());
        });

        b.compile()
    };

    let fragment_shader = device
        .create_shader_module(&gpu::ShaderModuleDesc {
            name: None,
            entries: &[(gpu::ShaderStages::FRAGMENT, "main")],
            spirv: &fragment_spv,
        })
        .unwrap();

    let render_pass = device
        .create_render_pass(&gpu::RenderPassDesc {
            name: None,
            colors: &[gpu::ColorAttachmentDesc {
                format: swapchain.format(),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
                initial_layout: gpu::TextureLayout::Undefined,
                final_layout: gpu::TextureLayout::SwapchainPresent,
            }],
            resolves: &[],
            depth: None,
            samples: gpu::Samples::S1,
        })
        .unwrap();

    let descriptor_layout = device
        .create_descriptor_layout(&gpu::DescriptorLayoutDesc {
            name: None,
            entries: &[
                // gpu::DescriptorLayoutEntry {
                //     ty: gpu::DescriptorLayoutEntryType::CombinedTextureSampler,
                //     stage: gpu::ShaderStages::FRAGMENT,
                //     count: std::num::NonZeroU32::new(1).unwrap(),
                // }
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::SampledTexture,
                    stage: gpu::ShaderStages::FRAGMENT,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::Sampler,
                    stage: gpu::ShaderStages::FRAGMENT,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
            ],
        })
        .unwrap();

    let layout = device
        .create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: None,
            descriptor_sets: &[&descriptor_layout],
            push_constants: &[],
        })
        .unwrap();

    let rasterizer = gpu::Rasterizer::default();

    let vertex_state = gpu::VertexState {
        stride: std::mem::size_of::<Vertex>() as _,
        input_rate: gpu::VertexInputRate::Vertex,
        attributes: &[
            // layout(location = 0) in vec2 in_pos;
            gpu::VertexAttribute {
                location: 0,
                format: gpu::VertexFormat::Vec2,
                offset: 0,
            },
            // layout(location = 1) in vec3 in_color;
            gpu::VertexAttribute {
                location: 1,
                format: gpu::VertexFormat::Vec2,
                offset: (2 * std::mem::size_of::<f32>()) as _,
            },
        ],
    };

    let blend_state = gpu::BlendState::REPLACE;

    let extent = swapchain.extent();

    let mut viewport = gpu::Viewport {
        x: 0,
        y: 0,
        width: extent.width,
        height: extent.height,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let mut pipeline = device
        .create_graphics_pipeline(&gpu::GraphicsPipelineDesc {
            name: None,
            layout: &layout,
            pass: &render_pass,
            vertex: (&vertex_shader, None),
            geometry: None,
            tessellation: None,
            fragment: Some((&fragment_shader, None)),
            rasterizer,
            vertex_states: &[vertex_state],
            blend_states: &[blend_state],
            depth_stencil: None,
            viewports: &[viewport],
            cache: None,
        })
        .unwrap();

    let mut command_buffer = device.create_command_buffer(None).unwrap();

    let sampler = device.create_sampler(&gpu::SamplerDesc::default()).unwrap();

    let i = image::open("../resources/images/rust.png").unwrap();
    let i_rgb = i.to_rgba8();
    let i_bytes = i_rgb.as_raw();
    let staging_buffer = device
        .create_buffer(&gpu::BufferDesc {
            name: None,
            size: i_bytes.len() as _,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
        })
        .unwrap();

    staging_buffer.slice_ref(..).write(i_bytes).unwrap();

    let texture = device
        .create_texture(&gpu::TextureDesc {
            name: None,
            format: gpu::Format::Rgba8Unorm,
            usage: gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COPY_DST,
            dimension: gpu::TextureDimension::D2(i_rgb.width(), i_rgb.height(), gpu::Samples::S1),
            mip_levels: std::num::NonZeroU32::new(1).unwrap(),
            memory: gpu::MemoryType::Device,
            layout: gpu::TextureLayout::ShaderReadOnlyOptimal,
        })
        .unwrap();

    let texture_view = texture.create_default_view().unwrap();

    let texture_slice = texture.slice_ref(&gpu::TextureSliceDesc {
        offset: gpu::Offset3D::ZERO,
        extent: texture.dimension().into(),
        base_array_layer: 0,
        array_layers: 1,
        base_mip_level: 0,
        mip_levels: 1,
    });

    command_buffer.begin(true).unwrap();

    command_buffer
        .pipeline_barrier(
            gpu::PipelineStageFlags::TOP_OF_PIPE,
            gpu::PipelineStageFlags::COPY,
            &[],
            &[gpu::TextureAccessInfo {
                texture: Cow::Borrowed(&texture),
                base_mip_level: 0,
                mip_levels: 1,
                base_array_layer: 0,
                array_layers: 1,
                src_access: gpu::AccessFlags::empty(),
                dst_access: gpu::AccessFlags::COPY_WRITE,
                src_layout: gpu::TextureLayout::ShaderReadOnlyOptimal,
                dst_layout: gpu::TextureLayout::CopyDstOptimal,
            }],
        )
        .unwrap();

    command_buffer
        .copy_buffer_to_texture(
            staging_buffer.slice_ref(..),
            texture_slice.clone(),
            gpu::TextureLayout::CopyDstOptimal,
        )
        .unwrap();

    command_buffer
        .pipeline_barrier(
            gpu::PipelineStageFlags::COPY,
            gpu::PipelineStageFlags::BOTTOM_OF_PIPE,
            &[],
            &[gpu::TextureAccessInfo {
                texture: Cow::Borrowed(&texture),
                base_mip_level: 0,
                mip_levels: 1,
                base_array_layer: 0,
                array_layers: 1,
                src_access: gpu::AccessFlags::COPY_WRITE,
                dst_access: gpu::AccessFlags::empty(),
                src_layout: gpu::TextureLayout::CopyDstOptimal,
                dst_layout: gpu::TextureLayout::ShaderReadOnlyOptimal,
            }],
        )
        .unwrap();

    command_buffer.end().unwrap();

    command_buffer.submit().unwrap();

    command_buffer.wait(!0).unwrap();

    let descriptor_set = device
        .create_descriptor_set(&gpu::DescriptorSetDesc {
            name: None,
            layout: &descriptor_layout,
            entries: &[
                // gpu::DescriptorSetEntry::combined_texture_sampler_ref(
                //     &texture_view,
                //     gpu::TextureLayout::ShaderReadOnlyOptimal,
                //     &sampler,
                // )
                gpu::DescriptorSetEntry::texture_ref(
                    &texture_view,
                    gpu::TextureLayout::ShaderReadOnlyOptimal,
                ),
                gpu::DescriptorSetEntry::sampler_ref(&sampler),
            ],
        })
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => resized = true,
            Event::RedrawRequested(_) => {
                if resized {
                    swapchain.recreate(&device).unwrap();
                    resized = false;

                    let extent = swapchain.extent();
                    viewport.width = extent.width;
                    viewport.height = extent.height;

                    pipeline = device
                        .create_graphics_pipeline(&gpu::GraphicsPipelineDesc {
                            name: None,
                            layout: &layout,
                            pass: &render_pass,
                            vertex: (&vertex_shader, None),
                            geometry: None,
                            tessellation: None,
                            fragment: Some((&fragment_shader, None)),
                            rasterizer,
                            vertex_states: &[vertex_state],
                            blend_states: &[blend_state],
                            depth_stencil: None,
                            viewports: &[viewport],
                            cache: None,
                        })
                        .unwrap();
                }

                let view = match swapchain.acquire(!0) {
                    Ok((view, _)) => view,
                    Err(e) => {
                        if e.can_continue() {
                            return;
                        } else {
                            panic!("{}", e)
                        }
                    }
                };

                command_buffer.begin(true).unwrap();

                command_buffer
                    .begin_graphics_pass(
                        &[gpu::Attachment::Swapchain(
                            &view,
                            gpu::ClearValue::ColorFloat([0.0, 0.0, 0.0, 1.0]),
                        )],
                        &[],
                        None,
                        &pipeline,
                    )
                    .unwrap();

                command_buffer
                    .bind_vertex_buffer(vertex_buffer.slice_ref(..), 0)
                    .unwrap();

                command_buffer
                    .bind_index_buffer(index_buffer.slice_ref(..), gpu::IndexType::U32)
                    .unwrap();

                command_buffer
                    .bind_descriptors(
                        0,
                        &[&descriptor_set],
                        gpu::PipelineBindPoint::Graphics,
                        &layout,
                    )
                    .unwrap();

                command_buffer
                    .draw_indexed(0, indices.len() as _, 0, 1, 0)
                    .unwrap();
                command_buffer.end_graphics_pass().unwrap();

                command_buffer.end().unwrap();

                command_buffer.submit().unwrap();

                match swapchain.present(view) {
                    Ok(_) => (),
                    Err(e) => {
                        if e.can_continue() {
                            return;
                        } else {
                            panic!("{}", e);
                        }
                    }
                }
            }
            _ => (),
        }
    })
}
