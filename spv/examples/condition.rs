
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
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

    let vertex_spv = {
        let b = spv::Builder::new();

        let in_pos = b.in_vec2(0, "in_pos");

        let vk_pos = b.vk_position();

        b.entry(spv::Stage::Vertex, "main", || {
            let pos = in_pos.load();
            vk_pos.store(b.vec4(pos.x(), pos.y(), 0.0, 1.0));
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

        let push_data = b.push_constants::<spv::UInt>(Some("push_data"));

        let out_col = b.out_vec4(0, "out_color");

        b.entry(spv::Stage::Fragment, "main", || {
            let data = push_data.load();

            spv::spv_if(data.eq(0), || {
                out_col.store(b.vec4(0.0, 1.0, 0.0, 1.0));
            }).spv_else(|| {
                out_col.store(b.vec4(1.0, 0.0, 0.0, 1.0));
            });
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

    let layout = device
        .create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: None,
            descriptor_sets: &[],
            push_constants: &[gpu::PushConstantRange {
                stage: gpu::ShaderStages::FRAGMENT,
                offset: 0,
                size: std::mem::size_of::<u32>() as _,
            }],
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

    let mut resized = false;

    let mut set: u32 = 0;

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
            } => {
                resized = true;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                set = !set;
            }
            Event::RedrawRequested(_) => {
                if resized {
                    resized = false;
                    swapchain.recreate(&device).unwrap();

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
                            resized = true;
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
                    .push_constants(
                        0,
                        bytemuck::bytes_of(&set),
                        gpu::ShaderStages::FRAGMENT,
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
                            resized = true;
                            return;
                        } else {
                            panic!("{}", e);
                        }
                    }
                }
            }
            _ => (),
        }
    });
}
