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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PushConstants {
    pub color: [f32; 4],
}

unsafe impl bytemuck::Pod for PushConstants {}
unsafe impl bytemuck::Zeroable for PushConstants {}

unsafe impl spv::AsSpvStruct for PushConstants {
    const DESC: spv::StructDesc = spv::StructDesc {
        name: "PushConstants",
        names: &["color"],
        fields: &[spv::DataType::Primitive(spv::PrimitiveType::Vec4)],
    };

    fn fields<'a>(&'a self) -> Vec<&'a dyn spv::AsData> {
        vec![&self.color]
    }
}

fn main() {
    env_logger::init();

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
        let builder = spv::VertexBuilder::new();

        let in_pos = builder.in_vec2(0, false, Some("in_pos"));

        let position = builder.position();

        builder.main(|b| {
            let pos = b.load_in(in_pos);
            let x = b.vector_shuffle(pos.x());
            let y = b.vector_shuffle(pos.y());
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

        builder.push_constant::<spv::Struct<PushConstants>>(None, Some("p_data"));

        let out_col = builder.out_vec4(0, false, Some("out_color"));

        builder.main(|b| {
            // load the field
            let col = b.load_push_constant_field::<spv::Vec4>("color");
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

    let layout = device
        .create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: None,
            descriptor_sets: &[],
            push_constants: &[gpu::PushConstantRange {
                stage: gpu::ShaderStages::FRAGMENT,
                offset: 0,
                size: std::mem::size_of::<PushConstants>() as _,
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

    let mut resized = false;

    let mut constants = PushConstants {
        color: [0.0, 0.0, 0.0, 1.0],
    };

    let start = std::time::Instant::now();

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

                let mut elapsed = start.elapsed().as_secs_f32();
                elapsed /= 5.0;

                constants.color = [
                    elapsed.cos().abs(),
                    elapsed.sin().abs(),
                    (elapsed.cos() * elapsed.sin()).abs(),
                    1.0,
                ];

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
                        bytemuck::bytes_of(&constants),
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
