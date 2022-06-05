
use std::borrow::Cow;

use winit_input_helper::WinitInputHelper;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    event::VirtualKeyCode,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const SIM_RESOLUTION: u32 = 256;
const INK_RESOLUTION: u32 = 1024;
const INK_DISSIPATION: f32 = 1.0;
const VELOCITY_DISSIPATION: f32 = 0.2;
const PRESSURE: f32 = 0.8;
const PRESSURE_ITERATIONS: u32 = 20;
const CURL: f32 = 30.0;
const SPLAT_RADIUS: f32 = 0.0025;
const SPLAT_FORCE: f32 = 6000.0;
const COLOR_TIME: f32 = 5.0;

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
    pub mul: f32,
}

unsafe impl bytemuck::Zeroable for SplatParams {}
unsafe impl bytemuck::Pod for SplatParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AdvectionParams {
    pub sim_texel_size: [f32; 2],
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
pub struct UniqueFields {
    splat_stage: gfx::ReflectedGraphics,
    advection_stage: gfx::ReflectedGraphics,
    divergence_stage: gfx::ReflectedGraphics,
    curl_stage: gfx::ReflectedGraphics,
    vorticity_stage: gfx::ReflectedGraphics,
    clear_stage: gfx::ReflectedGraphics,
    pressure_stage: gfx::ReflectedGraphics,
    grad_sub_stage: gfx::ReflectedGraphics,
    display_stage: gfx::ReflectedGraphics,

    ink_splat_bundle: gfx::Bundle,
    vel_splat_bundle: gfx::Bundle,

    curl: gfx::GTexture2D,
    divergence: gfx::GTexture2D,
}

#[allow(dead_code)]
pub struct DoubleFields {
    vel: gfx::GTexture2D,
    pressure: gfx::GTexture2D,
    ink: gfx::GTexture2D,

    advect_vel_bundle: gfx::Bundle,
    advect_ink_bundle: gfx::Bundle,
    divergence_bundle: gfx::Bundle,
    curl_bundle: gfx::Bundle,
    vorticity_bundle: gfx::Bundle,
    clear_bundle: gfx::Bundle,
    pressure_bundle: gfx::Bundle,
    grad_sub_bundle: gfx::Bundle,
    display_bundle: gfx::Bundle,
}

#[allow(dead_code)]
struct Fluid {
    instance: gpu::Instance,
    surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,
    mesh: gfx::IndexedMesh<Vertex>,
    offscreen_command_a: gpu::CommandBuffer,
    offscreen_command_b: gpu::CommandBuffer,
    onscreen_command: gpu::CommandBuffer,

    splat_update_needed: bool,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    paused: bool,
    rng: rand::rngs::ThreadRng,
    color_change: bool,

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

    a: DoubleFields,
    b: DoubleFields,
    u: UniqueFields,
}

impl Fluid {
    fn new(window: &Window) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;
        //let instance = unsafe { gpu::Instance::no_validation(&gpu::InstanceDesc::default())? };

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            ..Default::default()
        })?;

        let mut sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        sc_desc.format = gpu::Format::Bgra8Unorm;
        let swapchain = device.create_swapchain(&surface, &sc_desc)?;

        let mut onscreen_command = device.create_command_buffer(None)?;
        let offscreen_command_a = device.create_command_buffer(None)?;
        let offscreen_command_b = device.create_command_buffer(None)?;

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
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT
                | gpu::TextureUsage::COPY_SRC | gpu::TextureUsage::COPY_DST, 
            1, 
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let pressure_b = gfx::GTexture2D::from_formats(
            &device, 
            SIM_RESOLUTION, 
            SIM_RESOLUTION, 
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT
                | gpu::TextureUsage::COPY_SRC | gpu::TextureUsage::COPY_DST, 
            1, 
            [gpu::Format::R32Float, gpu::Format::Rg32Float, gpu::Format::Rgb32Float, gpu::Format::Rgba32Float], 
            None,
        )?.unwrap();
        let ink_a = gfx::GTexture2D::new(
            &device,
            INK_RESOLUTION,
            INK_RESOLUTION,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            gpu::Format::Rgba32Float,
            None,
        )?;
        let ink_b = gfx::GTexture2D::new(
            &device,
            INK_RESOLUTION,
            INK_RESOLUTION,
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
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
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
                mul: 0.0,
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
                mul: 0.0,
            },
            None,
        )?;

        let advect_vel_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            AdvectionParams {
                sim_texel_size: [1.0 / SIM_RESOLUTION as f32, 1.0 / SIM_RESOLUTION as f32],
                dissapation: VELOCITY_DISSIPATION,
                dt: 1.0 / 60.0,
            },
            None,
        )?;

        let advect_ink_params = gfx::Uniform::new(
            &mut encoder,
            &device,
            AdvectionParams {
                sim_texel_size: [1.0 / SIM_RESOLUTION as f32, 1.0 / SIM_RESOLUTION as f32],
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

        encoder.submit(&mut onscreen_command, true)?;

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

        let splat_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&splat_fragment),
            rasterizer,
            &[gpu::BlendState::ADD],
            depth_state,
            Some("splat_stage".to_string()),
        )?;

        let ink_splat_bundle = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &ink_splat_params)?
            .build(&device)?;

        let vel_splat_bundle = splat_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &vel_splat_params)?
            .build(&device)?;

        // advection
        // =======================================================================
        // =======================================================================

        let advection_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&advection_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("advect_stage".to_string()),
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
            .set_resource("u_params", &advect_vel_params)?
            .set_resource("u_velocity", &vel_a)?
            .set_resource("u_source", &ink_a)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        let advect_ink_bundle_b = advection_stage
            .bundle()
            .unwrap()
            .set_resource("u_vertex_params", &vertex_params)?
            .set_resource("u_params", &advect_vel_params)?
            .set_resource("u_velocity", &vel_b)?
            .set_resource("u_source", &ink_b)?
            .set_sampler_ref("u_sampler", &sampler)?
            .build(&device)?;

        // divergence
        // =======================================================================
        // =======================================================================

        let divergence_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&divergence_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("div_stage".to_string()),
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

        let curl_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&curl_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("curl_stage".to_string()),
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

        let vorticity_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&vorticity_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("vort_stage".to_string()),
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

        let clear_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&clear_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("clear_stage".to_string()),
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

        let pressure_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&pressure_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("pressure_stage".to_string()),
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

        let grad_sub_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&grad_sub_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("grad_sub_stage".to_string()),
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

        let display_stage = gfx::ReflectedGraphics::from_spv(
            &device,
            &basic_vertex,
            None,
            Some(&display_fragment),
            rasterizer,
            &[gpu::BlendState::REPLACE],
            depth_state,
            Some("display_stage".to_string()),
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

        let a = DoubleFields {
            vel: vel_a,
            pressure: pressure_a,
            ink: ink_a,
            advect_vel_bundle: advect_vel_bundle_a,
            advect_ink_bundle: advect_ink_bundle_a,
            divergence_bundle: divergence_bundle_a,
            curl_bundle: curl_bundle_a,
            vorticity_bundle: vorticity_bundle_a,
            clear_bundle: clear_bundle_a,
            pressure_bundle: pressure_bundle_a,
            grad_sub_bundle: grad_sub_bundle_a,
            display_bundle: display_bundle_a,
        };

        let b = DoubleFields {
            vel: vel_b,
            pressure: pressure_b,
            ink: ink_b,
            advect_vel_bundle: advect_vel_bundle_b,
            advect_ink_bundle: advect_ink_bundle_b,
            divergence_bundle: divergence_bundle_b,
            curl_bundle: curl_bundle_b,
            vorticity_bundle: vorticity_bundle_b,
            clear_bundle: clear_bundle_b,
            pressure_bundle: pressure_bundle_b,
            grad_sub_bundle: grad_sub_bundle_b,
            display_bundle: display_bundle_b,
        };

        let u = UniqueFields {
            splat_stage,
            advection_stage,
            divergence_stage,
            curl_stage,
            vorticity_stage,
            clear_stage,
            pressure_stage,
            grad_sub_stage,
            display_stage,

            ink_splat_bundle,
            vel_splat_bundle,

            curl,
            divergence,
        };

        let mut s = Self {
            instance,
            surface,
            device,
            swapchain,
            mesh,
            offscreen_command_a,
            offscreen_command_b,
            onscreen_command,

            splat_update_needed: false,
            start_time: std::time::Instant::now(),
            prev_time: std::time::Instant::now(),
            paused: false,
            rng: rand::thread_rng(),
            color_change: true,

            width: extent.width,
            height: extent.height,

            a,
            b,

            vertex_params,
            ink_splat_params,
            vel_splat_params,
            advect_vel_params,
            advect_ink_params,
            vorticity_params,
            clear_params,

            sampler,

            u,
        };

        s.record_offscreen()?;

        Ok(s)
    }

    pub fn update_pass<'a>(
        encoder: &mut gfx::CommandEncoder<'a>,
        device: &gpu::Device,
        mesh: &'a gfx::IndexedMesh<Vertex>,
        graphics: &gfx::ReflectedGraphics,
        bundle: &gfx::Bundle,
        output: &gpu::TextureView,
        load: gpu::LoadOp,
    ) -> Result<(), anyhow::Error> {
        let mut pass = encoder.graphics_pass_reflected(
            &device,
            &[
                gfx::Attachment {
                    raw: gpu::Attachment::View(
                        Cow::Owned(output.clone()), 
                        gpu::ClearValue::ColorFloat([0.0; 4])
                    ),
                    load,
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

    fn record_offscreen(&mut self) -> Result<(), anyhow::Error> {
        Self::render_offscreen_t(
            &self.device,
            &self.mesh,
            &mut self.u,
            &self.a,
            &self.b,
            &mut self.offscreen_command_a,
        )?;
        Self::render_offscreen_t(
            &self.device,
            &self.mesh,
            &mut self.u,
            &self.b,
            &self.a,
            &mut self.offscreen_command_b,
        )?;
        Ok(())
    }

    fn render_offscreen_t(
        device: &gpu::Device, 
        mesh: &gfx::IndexedMesh<Vertex>, 
        u: &mut UniqueFields, 
        a: &DoubleFields, 
        b: &DoubleFields,
        c: &mut gpu::CommandBuffer,
    ) -> Result<(), anyhow::Error> {
        // at the start 
        // a.vel must hold the data
        // a.ink must hold the data
        // 
        let mut encoder = gfx::CommandEncoder::new();

        // apply force into velocity field
        // render into a.vel requiring a.vel contains current data
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.splat_stage,
            &u.vel_splat_bundle,
            &a.vel.view,
            gpu::LoadOp::Load,
        )?;

        // apply color into ink
        // render into a.ink requiring a.ink contains current data
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.splat_stage,
            &u.ink_splat_bundle,
            &a.ink.view,
            gpu::LoadOp::Load, // ink_a needs to have the current data in already
        )?;

        // calculate curl 
        // reading from current velocity in a.vel
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.curl_stage,
            &a.curl_bundle,
            &u.curl.view,
            gpu::LoadOp::DontCare,
        )?;

        // vorticity
        // update velocity for vorticity effects
        // render into b.vel reading from a.vel
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.vorticity_stage,
            &a.vorticity_bundle,
            &b.vel.view,
            gpu::LoadOp::DontCare,
        )?;

        // divergence
        // calculate divergence read from b.vel
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.divergence_stage,
            &b.divergence_bundle,
            &u.divergence.view,
            gpu::LoadOp::DontCare,
        )?;

        // clear
        // render into a.pressure reading from b.pressure
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.clear_stage,
            &a.clear_bundle,
            &b.pressure.view,
            gpu::LoadOp::DontCare,
        )?;

        // pressure iterations
        for _ in 0..(PRESSURE_ITERATIONS/2) {
            // render into a.pressure reading from b.pressure
            Self::update_pass(
                &mut encoder,
                device,
                mesh,
                &u.pressure_stage,
                &b.pressure_bundle,
                &a.pressure.view,
                gpu::LoadOp::DontCare,
            )?;

            // render into b.pressure reading from a.pressure
            Self::update_pass(
                &mut encoder,
                device,
                mesh, 
                &u.pressure_stage,
                &a.pressure_bundle,
                &b.pressure.view,
                gpu::LoadOp::DontCare,
            )?;
        }

        // if odd need perform one more iteration and copy a.pressure into b.pressure 
        // as next cycle will assume that b has the current data
        if PRESSURE_ITERATIONS % 2 == 1 {
            Self::update_pass(
                &mut encoder,
                device,
                mesh,
                &u.pressure_stage,
                &b.pressure_bundle,
                &a.pressure.view,
                gpu::LoadOp::DontCare,
            )?;

            encoder.copy_texture_to_texture(
                a.pressure.whole_slice_ref(),
                b.pressure.whole_slice_ref(),
            );
        }

        // grad_sub
        // render into a.vel reading from b.vel and b.pressure
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.grad_sub_stage,
            &b.grad_sub_bundle,
            &a.vel.view,
            gpu::LoadOp::DontCare,
        )?;

        // advect velocity
        // render into b.vel reading from a.vel
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &u.advection_stage,
            &a.advect_vel_bundle,
            &b.vel.view,
            gpu::LoadOp::DontCare
        )?;

        // advect ink
        // rendering into b.ink reading from a.ink
        Self::update_pass(
            &mut encoder,
            device,
            mesh,
            &mut u.advection_stage,
            &a.advect_ink_bundle,
            &b.ink.view,
            gpu::LoadOp::DontCare,
        )?;

        encoder.record(c, false)?;

        Ok(())
    }

    pub fn redraw(&mut self, helper: &WinitInputHelper) -> Result<(), anyhow::Error> {
        let dt = self.prev_time.elapsed();
        let target = std::time::Duration::from_secs_f64(1.0 / 60.0);
        if dt < target {
            let wait = target - dt;
            std::thread::sleep(wait);
        }
        self.prev_time = std::time::Instant::now();

        if helper.key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
        }

        if helper.key_pressed(VirtualKeyCode::Tab) {
            self.color_change = !self.color_change;
        }

        if let Some(size) = helper.window_resized() {
            self.width = size.width;
            self.height = size.height;
            let ratio = self.width as f32 / self.height as f32;
            self.vel_splat_params.data.aspect_ratio = ratio;
            self.ink_splat_params.data.aspect_ratio = ratio;
            self.swapchain.recreate(&self.device)?;
        }

        let (frame, _) = self.swapchain.acquire(!0)?;

        let mut encoder = gfx::CommandEncoder::new();

        // self.advect_vel_params.data.dt = dt;
        // self.advect_ink_params.data.dt = dt;
        // self.vorticity_params.data.dt = dt;

        self.advect_vel_params.update_gpu_ref(&mut encoder);
        self.advect_ink_params.update_gpu_ref(&mut encoder);
        self.vorticity_params.update_gpu_ref(&mut encoder);

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

            let h = self.start_time.elapsed().as_secs_f32() % COLOR_TIME;
            let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
            self.ink_splat_params.data.color = [0.33 * r, 0.33 * g, 0.33 * b];
            self.ink_splat_params.data.point = [x, y];

            self.ink_splat_params.data.mul = 1.0;
            self.vel_splat_params.data.mul = 1.0;

            self.ink_splat_params.update_gpu_ref(&mut encoder);
            self.vel_splat_params.update_gpu_ref(&mut encoder);
        } else if helper.mouse_released(0) {
            self.ink_splat_params.data.mul = 0.0;
            self.vel_splat_params.data.mul = 0.0;

            self.ink_splat_params.update_gpu_ref(&mut encoder);
            self.vel_splat_params.update_gpu_ref(&mut encoder);
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
            &mut self.u.display_stage,
        )?;
        pass.set_bundle_owned(&self.a.display_bundle);
        pass.draw_mesh_ref(&self.mesh);
        pass.finish();

        encoder.submit(&mut self.onscreen_command, true)?;

        // integration
        if !self.paused {
            self.offscreen_command_a.submit()?;
            std::mem::swap(&mut self.offscreen_command_a, &mut self.offscreen_command_b);
            std::mem::swap(&mut self.a, &mut self.b);
        }

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