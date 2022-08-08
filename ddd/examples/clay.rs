
use std::borrow::Cow;

use ddd::clay;
use ddd::glam;
use ddd::prelude::*;

use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 850;
const HEIGHT: u32 = 850;

#[allow(dead_code)]
pub struct Clay {
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,

    target: gfx::GTexture2D,
    depth: gfx::GTexture2D,

    controller: ddd::utils::DebugController,
    camera: ddd::utils::Camera,

    mesh: gfx::IndexedMesh<clay::Vertex>,
    mesh_instance: ddd::utils::Instance,

    update_mouse: bool,
    prev_time: std::time::Instant,
    start_time: std::time::Instant,

    smaa_renderer: ddd::utils::SMAARenderer,
    display_renderer: ddd::utils::DisplayRenderer,
    smooth_renderer: clay::SmoothRenderer,

    width: u32,
    height: u32,

    command: gpu::CommandBuffer,
}

impl Clay {
    pub fn new(window: &winit::window::Window) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            features: gpu::DeviceFeatures::BASE,
            ..Default::default()
        })?;

        let mut sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        sc_desc.format = gpu::Format::Bgra8Unorm;
        let swapchain = device.create_swapchain(&surface, &mut sc_desc)?;

        let target = gfx::GTexture2D::from_formats(
            &device,
            swapchain.extent().width,
            swapchain.extent().height,
            gpu::Samples::S1,
            gpu::TextureUsage::COLOR_OUTPUT
                | gpu::TextureUsage::SAMPLED,
            1,
            gfx::alt_formats(gpu::Format::Rgba8Unorm),
            None,
        )?.unwrap();

        let depth = gfx::GTexture2D::from_formats(
            &device,
            swapchain.extent().width,
            swapchain.extent().height,
            gpu::Samples::S1,
            gpu::TextureUsage::DEPTH_OUTPUT
                | gpu::TextureUsage::SAMPLED,
            1,
            gfx::alt_formats(gpu::Format::Depth32Float),
            None,
        )?.unwrap();

        let mut command_buffer = device.create_command_buffer(None)?;

        let mut encoder = gfx::CommandEncoder::new();

        let mesh = mesh::load_meshes_from_obj(
            &mut encoder, 
            &device, 
            true, 
            "../resources/models/dragon.obj",
            None,
        )?.remove(0);

        let controller = ddd::utils::DebugController::from_flipped_perspective(
            glam::vec3(0.0, 0.0, 2.0),
            0.0,
            -std::f32::consts::FRAC_PI_2,
            2.0,
            0.05,
            std::f32::consts::FRAC_PI_4,
            WIDTH as f32 / HEIGHT as f32,
            0.01,
            100.0,
            true,
        );

        let camera = controller.create_cam(&mut encoder, &device, None)?;

        let smaa_renderer = ddd::utils::SMAARenderer::new(
            &mut encoder,
            &device,
            &target.view,
            ddd::utils::SMAAState::MEDIUM,
            ddd::utils::DisplayFlags::all(),
            None,
        )?;

        let display_renderer = ddd::utils::DisplayRenderer::new(
            &device, 
            &target.view, 
            ddd::utils::DisplayFlags::all(),
            None,
        )?;

        let smooth_renderer = clay::SmoothRenderer::new(&device, None)?;

        let scale = glam::Mat4::from_scale(glam::vec3(2.0, 2.0, 2.0));

        let mesh_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            scale.into(),
            None,
        )?;

        encoder.submit(&mut command_buffer, true)?;

        Ok(Self {
            _instance: instance,
            _surface: surface,
            device,
            swapchain,

            target,
            depth,

            controller,
            camera,
        
            mesh,
            mesh_instance,

            update_mouse: true,
            prev_time: std::time::Instant::now(),
            start_time: std::time::Instant::now(),

            smaa_renderer,
            display_renderer,
            smooth_renderer,

            width: WIDTH,
            height: HEIGHT,

            command: command_buffer,
        })
    }

    pub fn redraw(
        &mut self,
        helper: &WinitInputHelper,
        window: &Window,
        control_flow: &mut ControlFlow,
    ) -> Result<(), anyhow::Error> {
        let mut dt = self.prev_time.elapsed().as_secs_f32();
        self.prev_time = std::time::Instant::now();

        if let Some(size) = helper.window_resized() {
            self.width = size.width;
            self.height = size.height;
            self.swapchain.recreate(&self.device)?;
        }

        if helper.held_shift() {
            dt *= 3.0;
        }

        if helper.key_held(VirtualKeyCode::A) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Right, dt);
        }
        if helper.key_held(VirtualKeyCode::D) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Left, dt);
        }
        if helper.key_held(VirtualKeyCode::W) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Forward, dt);
        }
        if helper.key_held(VirtualKeyCode::S) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Back, dt);
        }
        if helper.key_held(VirtualKeyCode::Up) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Up, dt);
        }
        if helper.key_held(VirtualKeyCode::Down) {
            self.controller
                .move_cam(ddd::utils::CameraMoveDirection::Down, dt);
        }

        if helper.key_held(VirtualKeyCode::Escape) {
            *control_flow = ControlFlow::Exit;
        }

        if self.update_mouse {
            self.controller.look_cam(helper.mouse_diff(), dt);
        } else {
            self.update_mouse = true;
        }

        if let Some((x, y)) = helper.mouse() {
            let width = self.width as f32;
            let height = self.height as f32;
            if x > (3.0 * width / 4.0)
                || x < (width / 4.0)
                || y > (3.0 * height / 4.0)
                || y < (height / 4.0)
            {
                window.set_cursor_position(winit::dpi::PhysicalPosition {
                    x: width / 2.0,
                    y: height / 2.0,
                })?;
                self.update_mouse = false;
            }
        }

        let (frame, _) = self.swapchain.acquire(!0)?;

        let mut encoder = gfx::CommandEncoder::new();

        self.controller
            .update_cam_owned(&mut encoder, &mut self.camera);

        self.smooth_renderer.pass(
            &mut encoder, 
            &self.device, 
            gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&self.target.view),
                    gpu::ClearValue::ColorFloat([0.2, 0.2, 0.2, 1.0]),
                ), 
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            },
            gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&self.depth.view),
                    gpu::ClearValue::ColorFloat([0.2, 0.2, 0.2, 1.0]),
                ), 
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            }, 
            Some((
                &self.mesh as _,
                &self.mesh_instance,
                [0.7, 0.7, 0.7, 1.0],
            )), 
            &self.camera,
        )?;

        // self.smaa_renderer.aces(
        //     &mut encoder,
        //     &self.device,
        //     gfx::Attachment {
        //         raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
        //         load: gpu::LoadOp::Clear,
        //         store: gpu::StoreOp::Store,
        //     },
        // )?;

        // self.display_renderer.aces(
        //     &mut encoder,
        //     &self.device,
        //     gfx::Attachment {
        //         raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
        //         load: gpu::LoadOp::Clear,
        //         store: gpu::StoreOp::Store,
        //     },
        // )?;

        encoder.submit(&mut self.command, true)?;

        self.swapchain.present(frame)?;

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("3d")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut cone = Clay::new(&window).unwrap();

    let mut input_helper = WinitInputHelper::new();
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if input_helper.update(&event) {
            match cone.redraw(&input_helper, &window, control_flow) {
                Ok(_) => (),
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<gpu::Error>() {
                        if e.can_continue() {
                            return;
                        }
                    }
                    panic!("{}", e);
                }
            }

            if input_helper.quit() {
                *control_flow = ControlFlow::Exit;
            }
        }
    })
}
