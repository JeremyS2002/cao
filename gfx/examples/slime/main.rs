use gfx::prelude::*;

use winit_input_helper::WinitInputHelper;

use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use rand::prelude::*;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const NUM_AGENTS: u32 = 250000;
const UPDATE_DISPATCH: u32 = NUM_AGENTS / 64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Data {
    speed: f32,
    dt: f32,
    fade_speed: f32,
    diffuse_speed: f32,
    sensor_size: i32,
    sensor_dist: i32,
    sensor_spacing: f32,
    turn_speed: f32,
    seed: u32,
}

#[allow(dead_code)]
impl Data {
    const PRESET_1: Self = Self {
        speed: 50.0,
        dt: 0.0,
        fade_speed: 0.5,
        diffuse_speed: 5.0,
        turn_speed: 20.0,
        sensor_size: 3,
        sensor_dist: 10,
        sensor_spacing: std::f32::consts::FRAC_PI_4,
        seed: 0,
    };

    const PRESET_2: Self = Self {
        speed: 50.0,
        dt: 0.0,
        fade_speed: 2.3991458,
        diffuse_speed: 22.53,
        sensor_size: 3,
        sensor_dist: 10,
        sensor_spacing: 0.7853982,
        turn_speed: 185.57643,
        seed: 0,
    };

    const PRESET_3: Self = Self {
        speed: 255.72432,
        dt: 0.02,
        fade_speed: 8.703089,
        diffuse_speed: 3.4185317,
        sensor_size: 3,
        sensor_dist: 10,
        sensor_spacing: 0.7853982,
        turn_speed: 39.94233,
        seed: 0,
    };
}

unsafe impl bytemuck::Pod for Data {}
unsafe impl bytemuck::Zeroable for Data {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Agent {
    pos: [f32; 2],
    angle: f32,
}

unsafe impl bytemuck::Pod for Agent {}
unsafe impl bytemuck::Zeroable for Agent {}

struct Slime {
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,

    swapchain: gpu::Swapchain,

    command: gpu::CommandBuffer,

    update: gfx::ReflectedCompute,
    update_bundle: gfx::Bundle,

    fade: gfx::ReflectedCompute,
    fade_bundle: gfx::Bundle,

    trail_map: gfx::GTexture2D,
    sampler: gpu::Sampler,

    uniform: gfx::Uniform<Data>,
    agents: gfx::Storage<Agent>,

    graphics: gfx::ReflectedGraphics,
    graphics_bundle: gfx::Bundle,

    prev_time: std::time::Instant,
    paused: bool,

    width: u32,
    height: u32,

    rng: rand::rngs::ThreadRng,
}

impl Slime {
    fn new(window: &Window, agents: &[Agent], data: Data) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            ..Default::default()
        })?;

        let sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        let swapchain = device.create_swapchain(&surface, &sc_desc)?;

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            mag_filter: gpu::FilterMode::Nearest,
            min_filter: gpu::FilterMode::Nearest,
            ..Default::default()
        })?;

        let mut command = device.create_command_buffer(None)?;

        let mut encoder = gfx::CommandEncoder::new();

        let uniform = gfx::Uniform::new(&mut encoder, &device, data, None)?;

        let rng = rand::thread_rng();

        let agents = gfx::Storage::new(&mut encoder, &device, agents, None)?;

        encoder.submit(&mut command, true)?;

        let trail_map = gfx::GTexture2D::new(
            &device,
            WIDTH,
            HEIGHT,
            gpu::Samples::S1,
            gpu::TextureUsage::STORAGE
                | gpu::TextureUsage::SAMPLED
                | gpu::TextureUsage::COPY_SRC
                | gpu::TextureUsage::COPY_DST,
            1,
            gpu::Format::Rgba16Float,
            None,
        )?;

        let update = gfx::ReflectedCompute::new(&device, &gpu::include_spirv!("update.spv"), None)?;

        let update_bundle = update
            .bundle()
            .unwrap()
            .set_resource("u_trail_map", &trail_map)?
            .set_resource("u_data", &uniform)?
            .set_resource("u_agents", &agents)?
            .build(&device)?;

        let fade = gfx::ReflectedCompute::new(&device, &gpu::include_spirv!("fade.spv"), None)?;

        let fade_bundle = fade
            .bundle()
            .unwrap()
            .set_resource("u_trail_map", &trail_map)?
            .set_resource("u_data", &uniform)?
            .build(&device)?;

        // let display_pass = device.create_render_pass(&gpu::RenderPassDesc {
        //     name: None,
        //     colors: &[
        //         gpu::ColorAttachmentDesc {
        //             format: swapchain.format(),
        //             load: gpu::LoadOp::DontCare,
        //             store: gpu::StoreOp::Store,
        //             initial_layout: gpu::TextureLayout::ColorAttachmentOptimal,
        //             final_layout: gpu::TextureLayout::SwapchainPresent,
        //         }
        //     ],
        //     resolves: &[],
        //     depth: None,
        //     samples: gpu::Samples::S1,
        // })?;

        let graphics = gfx::ReflectedGraphics::from_spv(
            &device,
            &gpu::include_spirv!("display_vert.spv"),
            None,
            Some(&gpu::include_spirv!("display_frag.spv")),
            gpu::Rasterizer::default(),
            &[gpu::BlendState::REPLACE],
            None,
            None,
        )?;

        let graphics_bundle = graphics
            .bundle()
            .unwrap()
            .set_resource("u_texture", &trail_map)?
            .set_resource("u_sampler", &sampler)?
            .build(&device)?;

        let prev_time = std::time::Instant::now();

        Ok(Self {
            _instance: instance,
            _surface: surface,
            device,

            swapchain,

            command,

            trail_map,
            sampler,

            uniform,
            agents,

            update,
            update_bundle,

            fade,
            fade_bundle,

            graphics,
            graphics_bundle,

            prev_time,
            paused: false,

            width: WIDTH,
            height: HEIGHT,

            rng,
        })
    }

    fn redraw(&mut self, helper: &WinitInputHelper) -> Result<(), anyhow::Error> {
        let dt = self.prev_time.elapsed().as_secs_f32();
        self.prev_time = std::time::Instant::now();

        self.uniform.data.dt = dt;
        self.uniform.data.seed = self.rng.gen();

        if helper.key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        if helper.key_held(VirtualKeyCode::S) {
            if helper.held_shift() {
                self.uniform.data.speed -= 10.0 * dt;
            } else {
                self.uniform.data.speed += 10.0 * dt;
            }
        }
        if helper.key_held(VirtualKeyCode::D) {
            if helper.held_shift() {
                self.uniform.data.diffuse_speed -= 10.0 * dt;
            } else {
                self.uniform.data.diffuse_speed += 10.0 * dt;
            }
        }
        if helper.key_held(VirtualKeyCode::F) {
            if helper.held_shift() {
                self.uniform.data.fade_speed -= 10.0 * dt;
            } else {
                self.uniform.data.fade_speed += 10.0 * dt;
            }
        }
        if helper.key_held(VirtualKeyCode::T) {
            if helper.held_shift() {
                self.uniform.data.turn_speed -= 10.0 * dt;
            } else {
                self.uniform.data.turn_speed += 10.0 * dt;
            }
        }
        if helper.key_pressed(VirtualKeyCode::P) {
            dbg!(self.uniform.data);
        }

        let mut encoder = gfx::CommandEncoder::new();

        if let Some(size) = helper.window_resized() {
            self.width = size.width;
            self.height = size.height;

            self.swapchain.recreate(&self.device)?;

            let mut trail_map = gfx::GTexture2D::new(
                &self.device,
                size.width,
                size.height,
                gpu::Samples::S1,
                gpu::TextureUsage::STORAGE
                    | gpu::TextureUsage::SAMPLED
                    | gpu::TextureUsage::COPY_SRC
                    | gpu::TextureUsage::COPY_DST,
                1,
                gpu::Format::Rgba16Float,
                None,
            )?;

            std::mem::swap(&mut trail_map, &mut self.trail_map);

            self.update_bundle = self
                .update
                .bundle()
                .unwrap()
                .set_resource("u_trail_map", &self.trail_map)?
                .set_resource("u_data", &self.uniform)?
                .set_resource("u_agents", &self.agents)?
                .build(&self.device)?;

            self.fade_bundle = self
                .fade
                .bundle()
                .unwrap()
                .set_resource("u_trail_map", &self.trail_map)?
                .set_resource("u_data", &self.uniform)?
                .build(&self.device)?;

            self.graphics_bundle = self
                .graphics
                .bundle()
                .unwrap()
                .set_resource("u_texture", &self.trail_map)?
                .set_resource("u_sampler", &self.sampler)?
                .build(&self.device)?;

            let old_extent: gpu::Extent3D = trail_map.dimension().into();
            let new_extent: gpu::Extent3D = self.trail_map.dimension().into();
            let extent = gpu::Extent3D {
                width: old_extent.width.min(new_extent.width),
                height: old_extent.height.min(new_extent.height),
                depth: 1,
            };

            encoder.copy_texture_to_texture(
                trail_map.texture.slice_owned(&gpu::TextureSliceDesc {
                    offset: gpu::Offset3D::ZERO,
                    extent,
                    base_array_layer: 0,
                    array_layers: 1,
                    base_mip_level: 0,
                    mip_levels: 1,
                }),
                self.trail_map.slice_ref(&gpu::TextureSliceDesc {
                    offset: gpu::Offset3D::ZERO,
                    extent,
                    base_array_layer: 0,
                    array_layers: 1,
                    base_mip_level: 0,
                    mip_levels: 1,
                }),
            );
        }

        let (view, _) = self.swapchain.acquire(!0)?;

        self.uniform.update_gpu_ref(&mut encoder);

        if !self.paused {
            let mut update_pass = encoder.compute_pass_reflected_ref(&self.update)?;
            update_pass.set_bundle_ref(&self.update_bundle);
            update_pass.dispatch(UPDATE_DISPATCH, 1, 1);
            update_pass.finish();

            let mut fade_pass = encoder.compute_pass_reflected_ref(&self.fade)?;
            fade_pass.set_bundle_ref(&self.fade_bundle);
            fade_pass.dispatch(self.width, self.height, 1);
            fade_pass.finish();
        }

        let mut pass = encoder.graphics_pass_reflected::<()>(
            &self.device,
            &[gfx::Attachment {
                raw: gpu::Attachment::Swapchain(&view, gpu::ClearValue::ColorFloat([0.0; 4])),
                load: gpu::LoadOp::DontCare,
                store: gpu::StoreOp::Store,
            }],
            &[],
            None,
            &self.graphics,
        )?;

        pass.set_bundle_ref(&self.graphics_bundle);
        pass.draw(0, 6, 0, 1);

        pass.finish();

        encoder.submit(&mut self.command, true)?;

        self.swapchain.present(view)?;

        Ok(())
    }
}

#[allow(dead_code)]
fn agents1() -> Vec<Agent> {
    let mut agents = Vec::new();
    for i in 0..NUM_AGENTS {
        agents.push(Agent {
            pos: [WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0],
            angle: (i as f32 / NUM_AGENTS as f32) * std::f32::consts::TAU,
        })
    }
    agents
}

#[allow(dead_code)]
fn agents2() -> Vec<Agent> {
    let mut agents = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_AGENTS {
        let theta = rng.gen::<f32>() * std::f32::consts::TAU;
        let r = rng.gen::<f32>() * WIDTH.min(HEIGHT) as f32 / 4.0;
        agents.push(Agent {
            pos: [
                WIDTH as f32 / 2.0 + r * theta.cos(),
                HEIGHT as f32 / 2.0 + r * theta.sin(),
            ],
            angle: std::f32::consts::TAU - theta,
        })
    }
    agents
}

#[allow(dead_code)]
fn agents3() -> Vec<Agent> {
    let mut agents = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_AGENTS {
        let x = rng.gen::<f32>() * WIDTH as f32;
        let y = rng.gen::<f32>() * HEIGHT as f32;
        let theta = rng.gen::<f32>() * std::f32::consts::TAU;
        agents.push(Agent {
            pos: [x, y],
            angle: theta,
        });
    }
    agents
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Slime")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let agents = agents2();

    let data = Data::PRESET_2;

    let mut slime = Slime::new(&window, &agents, data).unwrap();

    let mut input_helper = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if input_helper.update(&event) {
            match slime.redraw(&input_helper) {
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
