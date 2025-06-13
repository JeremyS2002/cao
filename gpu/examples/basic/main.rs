
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct BasicApp {
    window: Window,
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,
    resized: bool,
}

impl BasicApp {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = Window::default_attributes().with_title("BasicApp");

        let window = event_loop.create_window(window_attributes).unwrap();

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

        let surface = instance.create_surface(&window).unwrap();
        let device = instance
            .create_device(&gpu::DeviceDesc {
                compatible_surfaces: &[&surface],
                ..Default::default()
            })
            .unwrap();

        let swapchain = device
            .create_swapchain(
                &surface,
                &gpu::SwapchainDesc::from_surface(&surface, &device).unwrap(),
            )
            .unwrap();

        BasicApp {
            window,
            _instance: instance,
            _surface: surface,
            device,
            swapchain,
            resized: false,
        }
    }

    pub fn redraw(&mut self) {
        if self.resized {
            self.swapchain.recreate(&self.device).unwrap();
            self.resized = false;
        }

        let view = match self.swapchain.acquire(!0) {
            Ok((view, _)) => view,
            Err(e) => if e.can_continue() {
                self.resized = true;
                return;
            } else {
                panic!("{}", e);
            }
        };

        match self.swapchain.present(view) {
            Ok(_) => (),
            Err(e) => if e.can_continue() {
                self.resized = true;
                return;
            } else {
                panic!("{}", e);
            }
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<BasicApp>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state = Some(BasicApp::new(event_loop));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = if let Some(s) = self.state.as_mut() {
            s
        } else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::Resized(_) => { 
                state.resized = true;
            },
            WindowEvent::RedrawRequested => {
                state.redraw();
                state.window.request_redraw();
            }
            _ => (),
        }
    }

}

fn main() {
    colog::init();

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
