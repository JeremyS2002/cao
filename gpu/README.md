# gpu
Low (ish) level wrapper of vulkan. Manages creation and destruction of vulkan objects as well as hiding some of the more unpleasant parts of the vulkan api.

# Examples

Basic example of creating a window and not drawing anything to it

```rust
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let instance = gpu::Instance::new(&gpu::InstanceDesc::default()).unwrap();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let surface = instance.create_surface(&window).unwrap();
    let device = instance.create_device(&gpu::DeviceDesc {
        combatible_surfaces: &[&surface],
        ..Default::default()
    }).unwrap();
    
    let mut swapchain = device.create_swapchain(
        &surface,
        &gpu::SwapchainDesc::from_surface(&surface, &device).unwrap(),
    ).unwrap();
    
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
```

More examples can be found in the examples folder (run from inside `cao/gpu/` to avoid path errors in loading files)

|Name          |Command                             |Description                                                                               |
|--------------|------------------------------------|------------------------------------------------------------------------------------------|
|basic         |`cargo run --example basic`         |Create a vulkan context and device then print infomation about them                       |
|clear         |`cargo run --example clear`         |Create a swapchain and clear the texture in a solid color each frame                      |
|triangle      |`cargo run --example triangle`      |Draw a triangle, introduces shaders, graphics pipelines and buffers                       |
|push_constants|`cargo run --example push_constants`|Builds off triangle example, uses push constants to color the triange                     |
|depth         |`cargo run --example depth`         |Introduces depth testing to draw a rotating cube                                          |
|compute       |`cargo run --example compute`       |Introduces compute shaders to compute the collatz conjecture for multiple values at once  |
