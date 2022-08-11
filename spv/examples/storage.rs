use spv::prelude::*;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
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
        Vertex { pos: [0.0, -0.5] },
        Vertex { pos: [-0.5, 0.5] },
        Vertex { pos: [0.5, 0.5] },
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


    let data = [1.0, 0.0, 1.0];
    let storage_buffer = device
        .create_buffer(&gpu::BufferDesc {
            name: None,
            size: (std::mem::size_of::<f32>() * data.len()) as _,
            usage: gpu::BufferUsage::STORAGE | gpu::BufferUsage::COPY_DST,
            memory: gpu::MemoryType::Host,
        })
        .unwrap();

    storage_buffer
        .slice_ref(..)
        .write(bytemuck::bytes_of(&data))
        .unwrap();

    let vertex_spv = {
        let builder = spv::VertexBuilder::new();

        let in_pos = builder.in_vec2(0, false, Some("in_pos"));

        let position = builder.position();

        builder.main(|b| {
            let pos = b.load_in(in_pos);
            let x = pos.x(b);
            let y = pos.y(b);
            let pos = b.vec4(&x, &y, &0.0, &1.0);
            b.store_out(position, pos);
        });

        builder.compile()
    };

    let vertex_shader = device
        .create_shader_module(&gpu::ShaderModuleDesc {
            name: None,
            entries: &[(gpu::ShaderStages::VERTEX, "main")],
            spirv: &vertex_spv,
        })
        .unwrap();

    let fragment_spv = {
        let builder = spv::FragmentBuilder::new();

        //let u = builder.uniform_struct::<Uniform>(0, 0, Some("u_data"));

        let s = builder.storage::<spv::Float>(
            spv::StorageAccessDesc { read: true, write: false, atomic: false }, 
            0, 
            0, 
            Some("s_data"),
        );

        let out_col = builder.out_vec3(0, false, Some("out_color"));

        builder.main(|b| {
            let red = b.load_storage_element(s, &0);
            let green = b.load_storage_element(s, &1);
            let blue = b.load_storage_element(s, &2);
            let col = b.vec3(&red, &green, &blue);
            // store composite into output
            b.store_out(out_col, col);
        });

        builder.compile()
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

    let descriptor_set_layout = device
        .create_descriptor_layout(&gpu::DescriptorLayoutDesc {
            name: None,
            entries: &[gpu::DescriptorLayoutEntry {
                ty: gpu::DescriptorLayoutEntryType::StorageBuffer { read_only: true },
                stage: gpu::ShaderStages::FRAGMENT,
                count: std::num::NonZeroU32::new(1).unwrap(),
            }],
        })
        .unwrap();

    let descriptor_set = device
        .create_descriptor_set(&gpu::DescriptorSetDesc {
            name: None,
            layout: &descriptor_set_layout,
            entries: &[gpu::DescriptorSetEntry::Buffer(
                storage_buffer.slice_ref(..),
            )],
        })
        .unwrap();

    let layout = device
        .create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: None,
            descriptor_sets: &[&descriptor_set_layout],
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
            vertex: &vertex_shader,
            geometry: None,
            tessellation: None,
            fragment: Some(&fragment_shader),
            rasterizer,
            vertex_states: &[vertex_state],
            blend_states: &[blend_state],
            depth_stencil: None,
            viewport,
        })
        .unwrap();

    let mut command_buffer = device.create_command_buffer(None).unwrap();

    let start_time = std::time::Instant::now();

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
            Event::MainEventsCleared => window.request_redraw(),
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
                            vertex: &vertex_shader,
                            geometry: None,
                            tessellation: None,
                            fragment: Some(&fragment_shader),
                            rasterizer,
                            vertex_states: &[vertex_state],
                            blend_states: &[blend_state],
                            depth_stencil: None,
                            viewport,
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

                let elapsed = start_time.elapsed().as_secs_f32() / 5.0;
                let r = elapsed.cos().abs();
                let g = elapsed.sin().abs();
                let b = (elapsed.cos() * elapsed.sin()).abs();
                let col = [r, g, b];

                command_buffer.begin(true).unwrap();

                command_buffer
                    .update_buffer(&storage_buffer, 0, bytemuck::cast_slice(&col))
                    .unwrap();

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
                    .bind_descriptor(
                        0,
                        &descriptor_set,
                        gpu::PipelineBindPoint::Graphics,
                        &layout,
                    )
                    .unwrap();

                command_buffer
                    .bind_vertex_buffer(vertex_buffer.slice_ref(..), 0)
                    .unwrap();

                command_buffer.draw(0, vertices.len() as _, 0, 1).unwrap();

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
