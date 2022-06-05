use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    for name in gpu::Instance::validation_layers().unwrap() {
        println!("{}", name);
    }

    println!("");

    for name in gpu::Instance::extensions().unwrap() {
        println!("{}", name);
    }

    println!("");

    let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();

    for device in instance.devices().unwrap() {
        println!("{:?}", device.name);
        println!("{:?}", device.device_type);
        println!("api version:    {:?}", device.api_version);
        println!("driver version: {:?}", device.driver_version);
        println!();
    }

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
                    Err(e) => {
                        if e.can_continue() {
                            resized = true;
                            return;
                        } else {
                            panic!("{}", e)
                        }
                    }
                };

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
