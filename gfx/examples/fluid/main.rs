
use std::borrow::Cow;

use rand::Rng;
use winit_input_helper::WinitInputHelper;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    event::VirtualKeyCode,
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

const SIM_RESOLUTION: u32 = 1024;
const INK_RESOLUTION: u32 = 1024;
const INK_DISSIPATION: f32 = 0.5;
const VELOCITY_DISSIPATION: f32 = 0.01;
const PRESSURE: f32 = 0.9;
const PRESSURE_ITERATIONS: u32 = 20;
const CURL: f32 = 50.0;
const SPLAT_RADIUS: f32 = 0.0025;
const SPLAT_FORCE: f32 = 0.05;
const COLOR_TIME: f32 = 1.0;

macro_rules! swap_vel {
    ($name:ident) => {
        std::mem::swap(&mut $name.vel_a, &mut $name.vel_b);
        std::mem::swap(&mut $name.vel_splat_bundle_a, &mut $name.vel_splat_bundle_b);
        std::mem::swap(
            &mut $name.advect_vel_bundle_a,
            &mut $name.advect_vel_bundle_b,
        );
        std::mem::swap(
            &mut $name.advect_ink_bundle_a.descriptor_sets[1],
            &mut $name.advect_ink_bundle_b.descriptor_sets[1],
        );
        std::mem::swap(&mut $name.curl_bundle_a, &mut $name.curl_bundle_b);
        std::mem::swap(
            &mut $name.divergence_bundle_a,
            &mut $name.divergence_bundle_b,
        );
        std::mem::swap(
            &mut $name.grad_sub_bundle_a.descriptor_sets[2],
            &mut $name.grad_sub_bundle_b.descriptor_sets[2],
        );
        std::mem::swap(&mut $name.vorticity_bundle_a, &mut $name.vorticity_bundle_b);
    };
}

macro_rules! swap_ink {
    ($name:ident) => {
        std::mem::swap(&mut $name.ink_a, &mut $name.ink_b);
        std::mem::swap(&mut $name.ink_splat_bundle_a, &mut $name.ink_splat_bundle_b);
        std::mem::swap(&mut $name.display_bundle_a, &mut $name.display_bundle_b);
        std::mem::swap(
            &mut $name.advect_ink_bundle_a.descriptor_sets[2],
            &mut $name.advect_ink_bundle_b.descriptor_sets[2],
        );
    };
}

macro_rules! swap_pressure {
    ($name:ident) => {
        std::mem::swap(&mut $name.pressure_a, &mut $name.pressure_b);
        std::mem::swap(&mut $name.clear_bundle_a, &mut $name.clear_bundle_b);
        std::mem::swap(
            &mut $name.grad_sub_bundle_a.descriptor_sets[1],
            &mut $name.grad_sub_bundle_b.descriptor_sets[1],
        );
        std::mem::swap(&mut $name.pressure_bundle_a, &mut $name.pressure_bundle_b);
    };
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match (i as u32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => unreachable!(),
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Vertex { }
unsafe impl bytemuck::Pod for Vertex { }

impl gfx::Vertex for Vertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec2)),
            "in_uv" => Some((
                std::mem::size_of::<[f32; 2]>() as u32,
                gpu::VertexFormat::Vec2,
            )),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexParams {
    pub texel_size: [f32; 2],
}

unsafe impl bytemuck::Zeroable for VertexParams { }
unsafe impl bytemuck::Pod for VertexParams { }

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SplatParams {
    pub aspect_ratio: f32,
    pub radius: f32,
    pub point: [f32; 2],
    pub color: [f32; 3],
}

unsafe impl bytemuck::Zeroable for SplatParams {}
unsafe impl bytemuck::Pod for SplatParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AdvectionParams {
    pub texel_size: [f32; 2],
    pub dissapation: f32,
    pub dt: f32,
}

unsafe impl bytemuck::Zeroable for AdvectionParams {}
unsafe impl bytemuck::Pod for AdvectionParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VorticityParams {
    pub curl: f32,
    pub dt: f32,
}

unsafe impl bytemuck::Zeroable for VorticityParams {}
unsafe impl bytemuck::Pod for VorticityParams {}

#[allow(dead_code)]
struct Fluid {
    instance: gpu::Instance,
    surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,
    mesh: gfx::IndexedMesh<Vertex>,
    command: gpu::CommandBuffer,

    splat_update_needed: bool,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    paused: bool,
    rng: rand::rngs::ThreadRng,
    color_change: bool,

    // stores a velocity field for each pixel
    vel_a: gfx::GTexture2D,
    vel_b: gfx::GTexture2D,
    // stores a pressure value for each pixel
    pressure_a: gfx::GTexture2D,
    pressure_b: gfx::GTexture2D,
    // stores color values for each pixel to be moved by the velocity
    ink_a: gfx::GTexture2D,
    ink_b: gfx::GTexture2D,
    // stores properties of the vector field for updating over time
    curl: gfx::GTexture2D,
    divergence: gfx::GTexture2D,

    width: u32,
    height: u32,

    vertex_params: gfx::Uniform<VertexParams>,
    ink_splat_params: gfx::Uniform<SplatParams>,
    vel_splat_params: gfx::Uniform<SplatParams>,
    advect_vel_params: gfx::Uniform<AdvectionParams>,
    advect_ink_params: gfx::Uniform<AdvectionParams>,
    vorticity_params: gfx::Uniform<VorticityParams>,
    clear_params: gfx::Uniform<f32>,

    sampler: gpu::Sampler,

    splat_stage: gfx::reflect::ReflectedGraphics,
    ink_splat_bundle_a: gfx::reflect::Bundle,
    ink_splat_bundle_b: gfx::reflect::Bundle,
    vel_splat_bundle_a: gfx::reflect::Bundle,
    vel_splat_bundle_b: gfx::reflect::Bundle,

    advection_stage: gfx::reflect::ReflectedGraphics,
    advect_vel_bundle_a: gfx::reflect::Bundle,
    advect_vel_bundle_b: gfx::reflect::Bundle,
    advect_ink_bundle_a: gfx::reflect::Bundle,
    advect_ink_bundle_b: gfx::reflect::Bundle,

    divergence_stage: gfx::reflect::ReflectedGraphics,
    divergence_bundle_a: gfx::reflect::Bundle,
    divergence_bundle_b: gfx::reflect::Bundle,

    curl_stage: gfx::reflect::ReflectedGraphics,
    curl_bundle_a: gfx::reflect::Bundle,
    curl_bundle_b: gfx::reflect::Bundle,

    vorticity_stage: gfx::reflect::ReflectedGraphics,
    vorticity_bundle_a: gfx::reflect::Bundle,
    vorticity_bundle_b: gfx::reflect::Bundle,

    clear_stage: gfx::reflect::ReflectedGraphics,
    clear_bundle_a: gfx::reflect::Bundle,
    clear_bundle_b: gfx::reflect::Bundle,

    pressure_stage: gfx::reflect::ReflectedGraphics,
    pressure_bundle_a: gfx::reflect::Bundle,
    pressure_bundle_b: gfx::reflect::Bundle,

    grad_sub_stage: gfx::reflect::ReflectedGraphics,
    grad_sub_bundle_a: gfx::reflect::Bundle,
    grad_sub_bundle_b: gfx::reflect::Bundle,

    display_stage: gfx::reflect::ReflectedGraphics,
    display_bundle_a: gfx::reflect::Bundle,
    display_bundle_b: gfx::reflect::Bundle,
}

impl Fluid {
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

        // (vector/scalar) fields
        // =======================================================================
        // =======================================================================

        // some of the fields can work with multiple formats
        let vel_a = gfx::GTexture2D::from_formats(
            &device, 
            SIM_RESOLUTION, 
            SIM_RESOLUTION, 
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT, 
            1, 
            [gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let vel_b = gfx::GTexture2D::from_formats(
            &device, 
            SIM_RESOLUTION, 
            SIM_RESOLUTION, 
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT, 
            1, 
            [gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let pressure_a = gfx::GTexture2D::from_formats(
            &device, 
            SIM_RESOLUTION, 
            SIM_RESOLUTION, 
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT, 
            1, 
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let pressure_b = gfx::GTexture2D::from_formats(
            &device, 
            SIM_RESOLUTION, 
            SIM_RESOLUTION, 
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT, 
            1, 
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let ink_a = gfx::GTexture2D::new(
            &device,
            SIM_RESOLUTION,
            SIM_RESOLUTION,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            gpu::Format::Rgba32Float,
            None,
        )?;
        let ink_b = gfx::GTexture2D::new(
            &device,
            SIM_RESOLUTION,
            SIM_RESOLUTION,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            gpu::Format::Rgba32Float,
            None,
        )?;
        let curl = gfx::GTexture2D::from_formats(
            &device,
            SIM_RESOLUTION,
            SIM_RESOLUTION,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float],
            None,
        )?.unwrap();
        let divergence = gfx::GTexture2D::from_formats(
            &device,
            SIM_RESOLUTION,
            SIM_RESOLUTION,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float],
            None,
        )?.unwrap();

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToBorder,
            wrap_y: gpu::WrapMode::ClampToBorder,
            wrap_z: gpu::WrapMode::ClampToBorder,
            border: gpu::BorderColor::OpaqueBlack,
            min_filter: gpu::FilterMode::Linear,
            mag_filter: gpu::FilterMode::Linear,
            ..Default::default()
        })?;

        // mesh
        // =======================================================================
        // =======================================================================

        let mesh = gfx::IndexedMesh::new(
            &mut encoder,
            &device,
            &[
                Vertex {
                    pos: [-1.0, -1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    pos: [1.0, -1.0],
                    uv: [1.0, 0.0],
                },
                Vertex {
                    pos: [1.0, 1.0],
                    uv: [1.0, 1.0],
                },
                Vertex {
                    pos: [-1.0, 1.0],
                    uv: [0.0, 1.0],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
            None,
        )?;

        // params
        // =======================================================================
        // =======================================================================

        let vertex_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            VertexParams {
                texel_size: [1.0 / SIM_RESOLUTION as f32, 1.0 / SIM_RESOLUTION as f32],
            },
            None,
        )?;

        let ink_splat_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            SplatParams {
                aspect_ratio: SIM_RESOLUTION as f32 / SIM_RESOLUTION as f32,
                radius: SPLAT_RADIUS,
                point: [0.0, 0.0],
                color: [0.0; 3],
            },
            None,
        )?;

        let vel_splat_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            SplatParams {
                aspect_ratio: SIM_RESOLUTION as f32 / SIM_RESOLUTION as f32,
                radius: SPLAT_RADIUS,
                point: [0.0, 0.0],
                color: [0.0; 3],
            },
            None,
        )?;

        let advect_vel_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            AdvectionParams {
                texel_size: [SIM_RESOLUTION as f32, SIM_RESOLUTION as f32],
                dissapation: VELOCITY_DISSIPATION,
                dt: 1.0 / 60.0,
            },
            None,
        )?;

        let advect_ink_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            AdvectionParams {
                texel_size: [INK_RESOLUTION as f32, INK_RESOLUTION as f32],
                dissapation: INK_DISSIPATION,
                dt: 1.0 / 60.0,
            },
            None,
        )?;

        let vorticity_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            VorticityParams {
                curl: CURL,
                dt: 1.0 / 60.0,
            },
            None,
        )?;

        let clear_params = gfx::Uniform::new(&mut encoder, &device, PRESSURE, None)?;

        encoder.submit(&mut command, true)?;

        // include spirv
        // =======================================================================
        // =======================================================================

        let basic_vertex = gpu::include_spirv!("basic_vertex.spv");
        let splat_fragment = gpu::include_spirv!("splat.spv");
        let advection_fragment = gpu::include_spirv!("advection.spv");
        let divergence_fragment = gpu::include_spirv!("divergence.spv");
        let curl_fragment = gpu::include_spirv!("curl.spv");
        let vorticity_fragment = gpu::include_spirv!("vorticity.spv");
        let pressure_fragment = gpu::include_spirv!("pressure.spv");
        let grad_sub_fragment = gpu::include_spirv!("gradient_sub.spv");
        let display_fragment = gpu::include_spirv!("display.spv");
        let clear_fragment = gpu::include_spirv!("clear.spv");

        let rasterizer = gpu::Rasterizer::default();
        let depth_state = None;

        let splat_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&splat_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let ink_splat_bundle_a = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &ink_splat_params)?
            .set_resource("u_target", &ink_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let ink_splat_bundle_b = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &ink_splat_params)?
            .set_resource("u_target", &ink_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let vel_splat_bundle_a = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &vel_splat_params)?
            .set_resource("u_target", &vel_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let vel_splat_bundle_b = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &vel_splat_params)?
            .set_resource("u_target", &vel_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // advection
        // =======================================================================
        // =======================================================================

        let advection_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&advection_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let advect_vel_bundle_a = advection_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &advect_vel_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_resource("u_source", &vel_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let advect_vel_bundle_b = advection_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &advect_vel_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_resource("u_source", &vel_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let advect_ink_bundle_a = advection_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &advect_ink_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_resource("u_source", &ink_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let advect_ink_bundle_b = advection_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &advect_ink_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_resource("u_source", &ink_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // divergence
        // =======================================================================
        // =======================================================================

        let divergence_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&divergence_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let divergence_bundle_a = divergence_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let divergence_bundle_b = divergence_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // curl
        // =======================================================================
        // =======================================================================

        let curl_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&curl_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let curl_bundle_a = curl_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let curl_bundle_b = curl_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // vorticity
        // =======================================================================
        // =======================================================================

        let vorticity_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&vorticity_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let vorticity_bundle_a = vorticity_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &vorticity_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_resource("u_curl", &curl)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let vorticity_bundle_b = vorticity_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &vorticity_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_resource("u_curl", &curl)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // clear
        // =======================================================================
        // =======================================================================

        let clear_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&clear_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let clear_bundle_a = clear_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &clear_params)?
            .set_resource("u_pressure", &pressure_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let clear_bundle_b = clear_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &clear_params)?
            .set_resource("u_pressure", &pressure_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // pressure
        // =======================================================================
        // =======================================================================

        let pressure_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&pressure_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let pressure_bundle_a = pressure_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_pressure", &pressure_a)?
            .set_resource("u_divergence", &divergence)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let pressure_bundle_b = pressure_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_pressure", &pressure_b)?
            .set_resource("u_divergence", &divergence)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // grad sub
        // =======================================================================
        // =======================================================================

        let grad_sub_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&grad_sub_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let grad_sub_bundle_a = grad_sub_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_resource("u_pressure", &pressure_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let grad_sub_bundle_b = grad_sub_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_resource("u_pressure", &pressure_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // display
        // =======================================================================
        // =======================================================================

        let display_stage = gfx::reflect::ReflectedGraphics::new(
            &device,
            &basic_vertex,
            Some(&display_fragment),
            None,
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            None,
        )?;

        let display_bundle_a = display_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_ink", &ink_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let display_bundle_b = display_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_ink", &ink_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // =======================================================================
        // =======================================================================

        let extent = swapchain.extent();

        Ok(Self {
            instance,
            surface,
            device,
            swapchain,
            mesh,
            command,

            splat_update_needed: false,
            start_time: std::time::Instant::now(),
            prev_time: std::time::Instant::now(),
            paused: false,
            rng: rand::thread_rng(),
            color_change: true,

            width: extent.width,
            height: extent.height,

            vel_a,
            vel_b,
            pressure_a,
            pressure_b,
            ink_a,
            ink_b,
            curl,
            divergence,

            vertex_params,
            ink_splat_params,
            vel_splat_params,
            advect_vel_params,
            advect_ink_params,
            vorticity_params,
            clear_params,

            sampler,

            splat_stage,
            vel_splat_bundle_a,
            vel_splat_bundle_b,
            ink_splat_bundle_a,
            ink_splat_bundle_b,

            advection_stage,
            advect_vel_bundle_a,
            advect_vel_bundle_b,
            advect_ink_bundle_a,
            advect_ink_bundle_b,

            divergence_stage,
            divergence_bundle_a,
            divergence_bundle_b,

            curl_stage,
            curl_bundle_a,
            curl_bundle_b,

            vorticity_stage,
            vorticity_bundle_a,
            vorticity_bundle_b,

            clear_stage,
            clear_bundle_a,
            clear_bundle_b,

            pressure_stage,
            pressure_bundle_a,
            pressure_bundle_b,

            grad_sub_stage,
            grad_sub_bundle_a,
            grad_sub_bundle_b,

            display_stage,
            display_bundle_a,
            display_bundle_b,
        })
    }

    pub fn update_pass<'a>(
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        mesh: &'a gfx::IndexedMesh<Vertex>,
        graphics: &gfx::reflect::ReflectedGraphics,
        bundle: &gfx::reflect::Bundle,
        output: &gpu::TextureView,
    ) -> Result<(), anyhow::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            &device,
            &[
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(output.clone()), 
                        gpu::ClearValue::ColorFloat([0.0; 4])
                    ),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                }
            ],
            &[],
            None,
            graphics,
        )?;
        pass.set_bundle_owned(&bundle);
        pass.draw_mesh_ref(mesh);
        Ok(())
    }

    pub fn redraw(&mut self, helper: &WinitInputHelper) -> Result<(), anyhow::Error> {
        let dt = self.prev_time.elapsed().as_secs_f32();
        self.prev_time = std::time::Instant::now();

        if helper.key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        if helper.key_pressed(VirtualKeyCode::Tab) {
            self.color_change = !self.color_change;
        }

        if let Some(_) = helper.window_resized() {
            self.swapchain.recreate(&self.device)?;
        }

        let (frame, _) = self.swapchain.acquire(!0)?;

        let mut encoder = gfx::CommandEncoder::new();

        self.advect_ink_params.data.dt = dt;
        self.advect_vel_params.data.dt = dt;
        self.vorticity_params.data.dt = dt;

        self.advect_ink_params.update_gpu_ref(&mut encoder);
        self.advect_vel_params.update_gpu_ref(&mut encoder);
        self.vorticity_params.update_gpu_ref(&mut encoder);

        if self.color_change {
            if self.start_time.elapsed().as_secs_f32() % COLOR_TIME <= 0.02 {
                let h = self.rng.gen_range(0.0..1.0);
                let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
                self.ink_splat_params.data.color = [0.5 * r, 0.5 * g, 0.5 * b];
            }
        } else {
            if helper.mouse_released(0) {
                let h = self.rng.gen_range(0.0..1.0);
                let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
                self.ink_splat_params.data.color = [0.5 * r, 0.5 * g, 0.5 * b];
            }
        }

        // apply force and color from user input
        if helper.mouse_held(0) {
            // get mouse data
            let (mut dx, mut dy) = helper.mouse_diff();
            let (mut x, mut y) = helper.mouse().unwrap();
            // scale to be texture coordinates
            let w = self.width as f32;
            let h = self.height as f32;
            x = x / w;
            y = y / h;
            dx = dx / w;
            dy = dy / h;

            self.vel_splat_params.data.color = [dx * SPLAT_FORCE, dy * SPLAT_FORCE, 0.0];
            self.vel_splat_params.data.point = [x, y];

            self.ink_splat_params.data.point = [x, y];

            self.ink_splat_params.update_gpu_ref(&mut encoder);
            self.vel_splat_params.update_gpu_ref(&mut encoder);

            // apply force to velocity
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &self.splat_stage,
                &self.vel_splat_bundle_b,
                &self.vel_a.view,
            )?;

            swap_vel!(self);

            // apply color into ink
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.splat_stage,
                &self.ink_splat_bundle_b,
                &self.ink_a.view,
            )?;

            swap_ink!(self);
        }

        // integration
        if !self.paused {
            // curl
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.curl_stage,
                &self.curl_bundle_a,
                &self.curl.view,
            )?;

            // vorticity
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.vorticity_stage,
                &self.vorticity_bundle_b,
                &self.vel_a.view,
            )?;

            swap_vel!(self);

            // divergence
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.divergence_stage,
                &self.divergence_bundle_b,
                &self.divergence.view,
            )?;

            // clear
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.clear_stage,
                &self.clear_bundle_b,
                &self.pressure_a.view,
            )?;

            swap_pressure!(self);

            // pressure
            for _ in 0..PRESSURE_ITERATIONS {
                Self::update_pass(
                    &mut encoder,
                    &self.device,
                    &self.mesh,
                    &self.pressure_stage,
                    &self.pressure_bundle_b,
                    &self.pressure_a.view,
                )?;

                swap_pressure!(self);
            }

            // grad_sub
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.grad_sub_stage,
                &mut self.grad_sub_bundle_b,
                &self.vel_a.view,
            )?;

            swap_vel!(self);

            // advect velocity
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &self.advection_stage,
                &mut self.advect_vel_bundle_b,
                &self.vel_a.view,
            )?;

            swap_vel!(self);

            // advect ink
            Self::update_pass(
                &mut encoder,
                &self.device,
                &self.mesh,
                &mut self.advection_stage,
                &mut self.advect_ink_bundle_b,
                &self.ink_a.view,
            )?;

            swap_ink!(self);
        }
        
        let mut pass = encoder.graphics_pass_reflected(
            &self.device,
            &[
                gfx::Attachment {
                    raw: gpu::Attachment::Swapchain(
                        &frame, 
                        gpu::ClearValue::ColorFloat([0.0; 4]),
                    ),
                    load: gpu::LoadOp::DontCare,
                    store: gpu::StoreOp::Store,
                }
            ],
            &[],
            None,
            &mut self.display_stage,
        )?;
        pass.set_bundle_owned(&self.display_bundle_a);
        pass.draw_mesh_ref(&self.mesh);
        pass.finish();

        encoder.submit(&mut self.command, true)?;

        self.swapchain.present(frame)?;

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("fractal")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut fractal = Fluid::new(&window).unwrap();

    let mut input_helper = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if input_helper.update(&event) {
            match fractal.redraw(&input_helper) {
                Ok(_) => (),
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<gpu::Error>() {
                        if e.can_continue() {
                            return
                        } 
                    }
                    panic!("{}", e);
                },
            }

            if input_helper.quit() {
                *control_flow = ControlFlow::Exit;
            }
        }
    })
}