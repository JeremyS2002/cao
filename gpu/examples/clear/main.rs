use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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

    let mut command_buffer = device.create_command_buffer(None).unwrap();

    let mut resized = false;

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
            Event::RedrawRequested(_) => {
                if resized {
                    swapchain.recreate(&device).unwrap();
                    resized = false;
                }

                let view = match swapchain.acquire(!0) {
                    Ok((view, _)) => view,
                    Err(e) => if e.can_continue() {
                        resized = true;
                        return
                    } else {
                        panic!("{}", e)
                    }
                };

                command_buffer.begin(true).unwrap();

                command_buffer
                    .empty_pass(
                        &[gpu::Attachment::Swapchain(
                            &view,
                            gpu::ClearValue::ColorFloat([1.0, 0.0, 0.0, 1.0]),
                        )],
                        &[],
                        None,
                        &render_pass,
                    )
                    .unwrap();

                command_buffer.end().unwrap();

                command_buffer.submit().unwrap();

                match swapchain.present(view) {
                    Ok(_) => (),
                    Err(e) => if e.can_continue() {
                        resized = true;
                        return
                    } else {
                        panic!("{}", e);
                    }
                }
            }
            _ => (),
        }
    });
}
