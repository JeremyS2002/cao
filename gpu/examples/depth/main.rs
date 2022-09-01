use std::borrow::Cow;

use winit::{
    event::{Event, WindowEvent},
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    col: [f32; 3],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub struct Uniform {
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
}

unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}

fn main() {
    let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Depth")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        }).build(&event_loop).unwrap();

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

    let red = [1.0, 0.0, 0.0];
    let green = [0.0, 1.0, 0.0];
    let blue = [0.0, 0.0, 1.0];
    let yellow = [1.0, 1.0, 0.0];
    let purple = [1.0, 0.0, 1.0];
    let turqoise = [0.0, 1.0, 1.0];

    let vertices = vec![
        // back face
        Vertex { pos: [-1.0, -1.0, -1.0], col: red },
        Vertex { pos: [1.0, -1.0, -1.0], col: red },
        Vertex { pos: [1.0, 1.0, -1.0], col: red },
        Vertex { pos: [-1.0, 1.0, -1.0], col: red },
        Vertex { pos: [-1.0, -1.0, -1.0], col: red },
        Vertex { pos: [1.0, 1.0, -1.0], col: red },
        // front face
        Vertex { pos: [-1.0, -1.0, 1.0], col: green },
        Vertex { pos: [1.0, -1.0, 1.0], col: green },
        Vertex { pos: [1.0, 1.0, 1.0], col: green },
        Vertex { pos: [-1.0, 1.0, 1.0], col: green },
        Vertex { pos: [-1.0, -1.0, 1.0], col: green },
        Vertex { pos: [1.0, 1.0, 1.0], col: green },
        // top face
        Vertex { pos: [-1.0, 1.0, -1.0], col: blue },
        Vertex { pos: [1.0, 1.0, -1.0], col: blue },
        Vertex { pos: [1.0, 1.0, 1.0], col: blue },
        Vertex { pos: [-1.0, 1.0, 1.0], col: blue },
        Vertex { pos: [-1.0, 1.0, -1.0], col: blue },
        Vertex { pos: [1.0, 1.0, 1.0], col: blue },
        // bottom face
        Vertex { pos: [-1.0, -1.0, -1.0], col: yellow },
        Vertex { pos: [1.0, -1.0, -1.0], col: yellow },
        Vertex { pos: [1.0, -1.0, 1.0], col: yellow },
        Vertex { pos: [-1.0, -1.0, 1.0], col: yellow },
        Vertex { pos: [-1.0, -1.0, -1.0], col: yellow },
        Vertex { pos: [1.0, -1.0, 1.0], col: yellow },
        // left face
        Vertex { pos: [-1.0, -1.0, -1.0], col: purple },
        Vertex { pos: [-1.0, -1.0, 1.0], col: purple },
        Vertex { pos: [-1.0, 1.0, 1.0], col: purple },
        Vertex { pos: [-1.0, 1.0, -1.0], col: purple },
        Vertex { pos: [-1.0, -1.0, -1.0], col: purple },
        Vertex { pos: [-1.0, 1.0, 1.0], col: purple },
        // right face
        Vertex { pos: [1.0, -1.0, -1.0], col: turqoise },
        Vertex { pos: [1.0, -1.0, 1.0], col: turqoise },
        Vertex { pos: [1.0, 1.0, 1.0], col: turqoise },
        Vertex { pos: [1.0, 1.0, -1.0], col: turqoise },
        Vertex { pos: [1.0, -1.0, -1.0], col: turqoise },
        Vertex { pos: [1.0, 1.0, 1.0], col: turqoise },
    ];
    let vertex_buffer = device.create_buffer(&gpu::BufferDesc {
        name: Some("vertex_buffer".to_string()),
        size: (std::mem::size_of::<Vertex>() * vertices.len()) as u64,
        usage: gpu::BufferUsage::VERTEX,
        memory: gpu::MemoryType::Host,
    }).unwrap();

    vertex_buffer.slice_ref(..).write(bytemuck::cast_slice(&vertices)).unwrap();

    let start = std::time::Instant::now();

    let mut uniform = Uniform {
        model: glam::Mat4::IDENTITY,
        view: glam::Mat4::from_translation(glam::vec3(0.0, 0.0, -5.0)),
        projection: glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, WIDTH as f32 / HEIGHT as f32, 0.01, 100.0),
    };

    let uniform_buffer = device.create_buffer(&gpu::BufferDesc {
        name: Some("uniform_buffer".to_string()),
        size: std::mem::size_of::<Uniform>() as u64,
        usage: gpu::BufferUsage::UNIFORM
            | gpu::BufferUsage::COPY_DST,
        memory: gpu::MemoryType::Host,
    }).unwrap();

    uniform_buffer.slice_ref(..).write(bytemuck::bytes_of(&uniform)).unwrap();

    let depth_map = device.create_texture(&gpu::TextureDesc {
        name: Some("depth".to_string()),
        format: gpu::Format::Depth32Float,
        usage: gpu::TextureUsage::DEPTH_OUTPUT,
        dimension: gpu::TextureDimension::D2(WIDTH, HEIGHT, gpu::Samples::S1),
        mip_levels: std::num::NonZeroU32::new(1).unwrap(),
        memory: gpu::MemoryType::Device,
        layout: gpu::TextureLayout::DepthAttachmentOptimal,
    }).unwrap();

    let mut depth_view = depth_map.create_default_view().unwrap();

    let render_pass = device.create_render_pass(&gpu::RenderPassDesc {
        name: Some("render_pass".to_string()),
        colors: &[
            gpu::ColorAttachmentDesc {
                format: swapchain.format(),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
                initial_layout: gpu::TextureLayout::Undefined,
                final_layout: gpu::TextureLayout::SwapchainPresent,
            }
        ],
        resolves: &[],
        depth: Some(gpu::DepthAttachmentDesc {
            format: depth_map.format(),
            load: gpu::LoadOp::Clear,
            store: gpu::StoreOp::DontCare,
            initial_layout: gpu::TextureLayout::DepthStencilAttachmentOptimal,
            final_layout: gpu::TextureLayout::DepthStencilAttachmentOptimal,
        }),
        samples: gpu::Samples::S1,
    }).unwrap();

    let desc_layout = device.create_descriptor_layout(&gpu::DescriptorLayoutDesc {
        name: Some("desc_layout".to_string()),
        entries: &[
            gpu::DescriptorLayoutEntry {
                ty: gpu::DescriptorLayoutEntryType::UniformBuffer,
                stage: gpu::ShaderStages::VERTEX,
                count: std::num::NonZeroU32::new(1).unwrap(),
            }
        ],
    }).unwrap();

    let desc_set = device.create_descriptor_set(&gpu::DescriptorSetDesc {
        name: Some("desc_set".to_string()),
        layout: &desc_layout,
        entries: &[
            gpu::DescriptorSetEntry::Buffer(uniform_buffer.slice_ref(..)),
        ],
    }).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&gpu::PipelineLayoutDesc {
        name: Some("pipeline_layout".to_string()),
        descriptor_sets: &[&desc_layout],
        push_constants: &[],
    }).unwrap();

    let vertex_spv = gpu::include_spirv!("vert.spv");
    let vertex_shader = device.create_shader_module(&gpu::ShaderModuleDesc {
        name: Some("vertex".to_string()),
        entries: &[(gpu::ShaderStages::VERTEX, "main")],
        spirv: &vertex_spv,
    }).unwrap();

    let fragment_spv = gpu::include_spirv!("frag.spv");
    let fragment_shader = device.create_shader_module(&gpu::ShaderModuleDesc {
        name: Some("fragment".to_string()),
        entries: &[(gpu::ShaderStages::FRAGMENT, "main")],
        spirv: &fragment_spv,
    }).unwrap();

    let vertex_state = gpu::VertexState {
        stride: std::mem::size_of::<Vertex>() as _,
        input_rate: gpu::VertexInputRate::Vertex,
        attributes: &[
            gpu::VertexAttribute {
                location: 0,
                format: gpu::VertexFormat::Vec3,
                offset: 0,
            },
            gpu::VertexAttribute {
                location: 1,
                format: gpu::VertexFormat::Vec3,
                offset: std::mem::size_of::<glam::Vec3>() as _,
            }
        ],
    };

    let depth_stencil = Some(gpu::DepthStencilState {
        depth: Some(gpu::DepthState {
            test_enable: true,
            write_enable: true,
            compare_op: gpu::CompareOp::LessEqual,
        }),
        stencil_front: None,
        stencil_back: None,
    });

    let mut viewport = gpu::Viewport {
        x: 0,
        y: 0,
        width: WIDTH,
        height: HEIGHT,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let blend_state = gpu::BlendState::default();

    let rasterizer = gpu::Rasterizer::default();

    let mut pipeline = device.create_graphics_pipeline(&gpu::GraphicsPipelineDesc {
        name: Some("pipeline".to_string()),
        layout: &pipeline_layout,
        pass: &render_pass,
        vertex: &vertex_shader,
        tessellation: None,
        geometry: None,
        fragment: Some(&fragment_shader),
        rasterizer,
        vertex_states: &[vertex_state],
        blend_states: &[blend_state],
        depth_stencil,
        viewports: &[viewport],
    }).unwrap();

    let mut command = device.create_command_buffer(None).unwrap();

    let mut width = WIDTH;
    let mut height = HEIGHT;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                resized = true;
                width = size.width;
                height = size.height;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if resized {
                    resized = false;

                    uniform.projection = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, width as f32 / height as f32, 0.01, 100.0);

                    let depth_map = device.create_texture(&gpu::TextureDesc {
                        name: Some("depth".to_string()),
                        format: gpu::Format::Depth32Float,
                        usage: gpu::TextureUsage::DEPTH_OUTPUT,
                        dimension: gpu::TextureDimension::D2(width, height, gpu::Samples::S1),
                        mip_levels: std::num::NonZeroU32::new(1).unwrap(),
                        memory: gpu::MemoryType::Device,
                        layout: gpu::TextureLayout::DepthAttachmentOptimal,
                    }).unwrap();

                    depth_view = depth_map.create_default_view().unwrap();

                    viewport.width = width;
                    viewport.height = height;

                    pipeline = device.create_graphics_pipeline(&gpu::GraphicsPipelineDesc {
                        name: Some("pipeline".to_string()),
                        layout: &pipeline_layout,
                        pass: &render_pass,
                        vertex: &vertex_shader,
                        tessellation: None,
                        geometry: None,
                        fragment: Some(&fragment_shader),
                        rasterizer,
                        vertex_states: &[vertex_state],
                        blend_states: &[blend_state],
                        depth_stencil,
                        viewports: &[viewport],
                    }).unwrap();

                    swapchain.recreate(&device).unwrap();
                }

                let elapsed = start.elapsed().as_secs_f32();

                let rot1 = glam::Mat4::from_rotation_x(0.5);
                let rot2 = glam::Mat4::from_rotation_y(elapsed);
                uniform.model = rot2 * rot1;

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

                command.begin(true).unwrap();

                command.update_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniform)).unwrap();

                command.begin_graphics_pass(
                    &[gpu::Attachment::Swapchain(
                        &view, 
                        gpu::ClearValue::ColorFloat([0.0, 0.0, 0.0, 1.0]),
                    )], 
                    &[], 
                    Some(gpu::Attachment::View(
                        Cow::Borrowed(&depth_view),
                        gpu::ClearValue::Depth(1.0),
                    )), 
                    &pipeline,
                ).unwrap();

                command.bind_descriptor(
                    0, 
                    &desc_set, 
                    gpu::PipelineBindPoint::Graphics, 
                    &pipeline_layout,
                ).unwrap();

                command.bind_vertex_buffer(vertex_buffer.slice_ref(..), 0).unwrap();

                command.draw(0, vertices.len() as _, 0, 1).unwrap();

                command.end_graphics_pass().unwrap();

                command.end().unwrap();

                command.submit().unwrap();

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
