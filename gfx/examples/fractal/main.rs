
use winit_input_helper::WinitInputHelper;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    event::VirtualKeyCode,
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniform {
    center: [f32; 2],
    width: f32,
    height: f32,
    start_val: [f32; 2],
    julia: i32,
    iterations: u32,
    offset: f32,
}

unsafe impl bytemuck::Pod for Uniform { }
unsafe impl bytemuck::Zeroable for Uniform { }


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: [f32; 2],
    tex_coord: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex { }
unsafe impl bytemuck::Zeroable for Vertex { }

impl gfx::Vertex for Vertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec2)),
            "in_uv" => Some((
                std::mem::size_of::<[f32; 2]>() as u32,
                gpu::VertexFormat::Vec2
            )),
            _ => None,
        }
    }
}

struct Fractal {
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,

    swapchain: gpu::Swapchain,

    command: gpu::CommandBuffer,

    target: gfx::GTexture2D,

    fractal_graphics: gfx::reflect::ReflectedGraphics,
    fractal_bundle: gfx::reflect::Bundle,

    display_graphics: gfx::reflect::ReflectedGraphics,
    display_bundle: gfx::reflect::Bundle,

    mesh: gfx::IndexedMesh<Vertex>,
    uniform: gfx::Uniform<Uniform>,

    prev_time: std::time::Instant,
}

impl Fractal {
    fn new(window: &Window) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            ..Default::default()
        })?;

        let sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        let swapchain = device.create_swapchain(&surface, &sc_desc)?;

        let mut command = device.create_command_buffer(None)?;

        let mut encoder = gfx::CommandEncoder::new();

        let target = gfx::GTexture2D::new(
            &device,
            2048,
            2048,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            gpu::Format::Rgba8Unorm,
            None,
        )?;

        let sampler = device.create_sampler(&gpu::SamplerDesc::LINEAR)?;

        let mesh = gfx::IndexedMesh::new(
            &mut encoder,
            &device,
            &[
                Vertex {
                    pos: [-1.0, -1.0],
                    tex_coord: [0.0, 0.0],
                },
                Vertex {
                    pos: [1.0, -1.0],
                    tex_coord: [1.0, 0.0],
                },
                Vertex {
                    pos: [1.0, 1.0],
                    tex_coord: [1.0, 1.0],
                },
                Vertex {
                    pos: [-1.0, 1.0],
                    tex_coord: [0.0, 1.0],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
            None,
        )?;

        let uniform = gfx::Uniform::new(&mut encoder, &device, Uniform {
            center: [0.0, 0.0],
            width: 2.0,
            height: 2.0,
            start_val: [0.0, 0.0],
            julia: 0,
            iterations: 75,
            offset: 0.0,
        }, None)?;

        encoder.submit(&mut command, true)?;

        let fractal_pass = device.create_render_pass(&gpu::RenderPassDesc {
            name: None,
            colors: &[
                gpu::ColorAttachmentDesc {
                    format: target.format(),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                    initial_layout: gpu::TextureLayout::ColorAttachmentOptimal,
                    final_layout: gpu::TextureLayout::ShaderReadOnlyOptimal,
                }
            ],
            resolves: &[],
            depth: None,
            samples: gpu::Samples::S1,
        })?;

        let fractal_graphics = gfx::reflect::ReflectedGraphics::new(
            &device,
            &gpu::include_spirv!("vert.spv"),
            Some(&gpu::include_spirv!("fractal_frag.spv")),
            None,
            fractal_pass,
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        )?;

        let fractal_bundle = fractal_graphics.bundle().unwrap()
            .set_resource_by_location(0, 0, &uniform)?
            .build(&device)?;

        let display_pass = device.create_render_pass(&gpu::RenderPassDesc {
            name: None,
            colors: &[
                gpu::ColorAttachmentDesc {
                    format: swapchain.format(),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                    initial_layout: gpu::TextureLayout::ColorAttachmentOptimal,
                    final_layout: gpu::TextureLayout::SwapchainPresent,
                }
            ],
            resolves: &[],
            depth: None,
            samples: gpu::Samples::S1,
        })?;

        let display_graphics = gfx::reflect::ReflectedGraphics::new(
            &device,
            &gpu::include_spirv!("vert.spv"),
            Some(&gpu::include_spirv!("display_frag.spv")),
            None,
            display_pass,
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        )?;

        let display_bundle = display_graphics.bundle().unwrap()
            .set_resource("u_texture", &target)?
            .set_resource("u_sampler", &sampler)?
            .build(&device)?;

        let prev_time = std::time::Instant::now();

        Ok(Self {
            _instance: instance,
            _surface: surface,
            device,
            
            swapchain,

            command,

            target,

            fractal_graphics,
            fractal_bundle,

            display_graphics,
            display_bundle,

            mesh,
            uniform,

            prev_time,
        })
    }

    fn redraw(&mut self, helper: &WinitInputHelper) -> Result<(), anyhow::Error> {
        let dt = self.prev_time.elapsed().as_secs_f32();
        self.prev_time = std::time::Instant::now();

        let delta = helper.scroll_diff();
        self.uniform.data.width += 0.1 * delta * self.uniform.data.width;
        self.uniform.data.height += 0.1 * delta * self.uniform.data.height;

        let delta_pos = if helper.held_control() {
            0.005 * dt * self.uniform.data.height
        } else if helper.held_shift() {
            0.05 * dt * self.uniform.data.height
        } else {
            0.5 * dt * self.uniform.data.height
        };

        if helper.key_held(VirtualKeyCode::A) {
            self.uniform.data.center[0] -= delta_pos
        }
        if helper.key_held(VirtualKeyCode::D) {
            self.uniform.data.center[0] += delta_pos;
        }
        if helper.key_held(VirtualKeyCode::S) {
            self.uniform.data.center[1] -= delta_pos;
        }
        if helper.key_held(VirtualKeyCode::W) {
            self.uniform.data.center[1] += delta_pos;
        }

        if helper.key_held(VirtualKeyCode::Left) {
            self.uniform.data.start_val[0] -= 0.1 * delta_pos;
        }
        if helper.key_held(VirtualKeyCode::Right) {
            self.uniform.data.start_val[0] += 0.1 * delta_pos;
        }
        if helper.key_held(VirtualKeyCode::Down) {
            self.uniform.data.start_val[1] -= 0.1 * delta_pos;
        }
        if helper.key_held(VirtualKeyCode::Up) {
            self.uniform.data.start_val[1] += 0.1 * delta_pos;
        }

        if helper.key_released(VirtualKeyCode::Tab) {
            self.uniform.data.julia = 1 - self.uniform.data.julia;
        }

        if helper.key_held(VirtualKeyCode::T) {
            self.uniform.data.offset += 0.1 * dt;
        }

        if helper.key_held(VirtualKeyCode::Y) {
            self.uniform.data.offset -= 0.1 * dt;
        }

        if helper.key_released(VirtualKeyCode::Return) {
            let ratio = self.uniform.data.width / self.uniform.data.height;
            self.uniform.data = Uniform {
                center: [0.0, 0.0],
                width: ratio * 2.0,
                height: 2.0,
                start_val: [0.0, 0.0],
                julia: 0,
                iterations: 75,
                offset: 0.0,
            };
        }

        if let Some(size) = helper.window_resized() {
            let ration = size.width as f32 / size.height as f32;
            self.uniform.data.width = ration * self.uniform.data.height;
            self.swapchain.recreate(&self.device)?;
        }

        let mut encoder = gfx::CommandEncoder::new();

        let (view, _) = self.swapchain.acquire(!0)?;

        self.uniform.update_gpu_ref(&mut encoder);

        let mut pass = encoder.graphics_pass_reflected(
            &self.device,
            &[
                gpu::Attachment::View(
                    &self.target.view, 
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                )
            ],
            &[],
            None,
            &self.fractal_graphics,
        )?;

        pass.set_bundle_ref(&self.fractal_bundle);
        pass.draw_mesh_ref(&self.mesh);
        pass.finish();

        let mut pass = encoder.graphics_pass_reflected(
            &self.device,
            &[
                gpu::Attachment::Swapchain(
                    &view, 
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                )
            ],
            &[],
            None,
            &self.display_graphics,
        )?;

        pass.set_bundle_ref(&self.display_bundle);
        pass.draw_mesh_ref(&self.mesh);
        pass.finish();

        encoder.submit(&mut self.command, true)?;

        self.swapchain.present(view)?;

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("fractal")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut fractal = Fractal::new(&window).unwrap();

    let mut input_helper = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if input_helper.update(&event) {
            match fractal.redraw(&input_helper) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }

            if input_helper.quit() {
                *control_flow = ControlFlow::Exit;
            }
        }
    })
}