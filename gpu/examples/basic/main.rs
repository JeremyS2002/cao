
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use env_logger::{Builder, Target};

const APP_NAME: &'static str = "basic";

struct BasicApp {
    window: Window,
    _instance: gpu::Instance,
    // _surface: gpu::Surface,
    // device: gpu::Device,
    // swapchain: gpu::Swapchain,
    recreate_swapchain: bool,
}

impl BasicApp {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = Window::default_attributes().with_title("BasicApp");

        let window = event_loop.create_window(window_attributes).unwrap();

        for name in gpu::Instance::validation_layers().unwrap() {
            println!("instance validation layer : {}", name);
        }
        println!("");
        for name in gpu::Instance::extensions().unwrap() {
            println!("instance extension : {}", name);
        } 
        println!("");

        let instance_desc = gpu::InstanceDesc::default();
        let instance = gpu::Instance::new(&instance_desc).unwrap();

        for device_info in instance.phys_devices().unwrap() {
            println!("{:?}", device_info.name);
            println!("{:?}", device_info.device_type);
            println!("api version:    {:?}", device_info.api_version);
            println!("driver version: {:?}", device_info.driver_version);
            // for ext in &device_info.extensions {
            //     println!("device extensions : {}", ext);
            // }
            println!();
        }

        println!("create surface");

        let surface = instance.create_surface(&window).unwrap();

        // println!("create device");

        // let device = instance
        //     .create_device(&gpu::DeviceDesc {
        //         compatible_surfaces: &[&surface],
        //         ..Default::default()
        //     })
        //     .unwrap();

        // println!("create swapchain");

        // let mut swapchain_desc = gpu::SwapchainDesc::from_surface(&surface, &device).unwrap();
        // swapchain_desc.name = Some("swapchain".to_string());
        // println!("{:?}", swapchain_desc);
        // let swapchain = device
        //     .create_swapchain(
        //         &surface,
        //         &swapchain_desc,
        //     )
        //     .unwrap();

        // println!("create app");

        BasicApp {
            window,
            _instance: instance,
            // _surface: surface,
            // device,
            // swapchain,
            recreate_swapchain: false,
        }
    }

    pub fn redraw(&mut self) {
        // if self.recreate_swapchain {
        //     self.swapchain.recreate(&self.device).unwrap();
        //     self.recreate_swapchain = false;
        // }

        // let (view, suboptimal) = match self.swapchain.acquire(!0) {
        //     Ok((view, suboptimal)) => (view, suboptimal),
        //     Err(e) => if e.can_continue() {
        //         #[cfg(feature = "logging")]
        //         log::trace!("acquired swapchain - is out of date");
        //         self.recreate_swapchain = true;
        //         return;
        //     } else {
        //         panic!("{}", e);
        //     }
        // };

        // if suboptimal {
        //     #[cfg(feature = "logging")]
        //     log::trace!("acquired swapchain - is suboptimal");
        //     self.recreate_swapchain = true;
        //     // return;
        // }

        // match self.swapchain.present(view) {
        //     Ok(suboptimal) => if suboptimal { 
        //         #[cfg(feature = "logging")]
        //         log::trace!("presented swapchain - is suboptimal");
        //         self.recreate_swapchain = true 
        //     },
        //     Err(e) => if e.can_continue() {
        //         #[cfg(feature = "logging")]
        //         log::trace!("presented swapchain - is out of date");
        //         self.recreate_swapchain = true;
        //         return;
        //     } else {
        //         panic!("{}", e);
        //     }
        // }
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
            WindowEvent::Resized(_size) => { 
                state.recreate_swapchain = true;
                #[cfg(feature = "logging")]
                log::trace!("window resized {:?}", _size);
                state.window.request_redraw();
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
    // let file = std::fs::File::create(format!("{}.log", APP_NAME)).unwrap();
    // let buf = std::io::BufWriter::new(file);
    // let mut builder = Builder::new();
    // builder.parse_env("RUST_LOG").target(Target::Pipe(Box::new(buf))).init();
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
