#![feature(vec_into_raw_parts)]

use ddd::clay;
use ddd::cone;
use ddd::glam;
use ddd::prelude::*;
use gfx::image;

use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

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
pub struct Cone {
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,

    query1: gpu::TimeQuery,
    query2: gpu::TimeQuery,

    controller: ddd::utils::DebugController,
    camera: ddd::utils::Camera,
    buffer: cone::GeometryBuffer,
    env_renderer: cone::EnvironmentRenderer,
    point_renderer: cone::PointLightRenderer,
    ao_renderer: cone::AORenderer,
    smaa_renderer: ddd::utils::SMAARenderer,
    display_renderer: ddd::utils::CopyRenderer,
    solid_renderer: clay::SolidRenderer,
    bloom_renderer: cone::BloomRenderer,
    tonemap_renderer: cone::GlobalToneMapRenderer,

    antialiased: gfx::GTexture2D,

    mesh: gfx::Mesh<cone::Vertex>,
    mesh_small: gfx::Mesh<cone::Vertex>,
    plane: gfx::Mesh<cone::Vertex>,
    cube: gfx::Mesh<clay::Vertex>,

    leather_instance: ddd::utils::Instances,
    metal_instance: ddd::utils::Instances,
    wax_instance: ddd::utils::Instances,
    chrome_instance: ddd::utils::Instances,
    light_instance: ddd::utils::Instances,
    wood_instance: ddd::utils::Instances,

    metal_material: cone::Material,
    leather_material: cone::Material,
    chrome_material: cone::Material,
    wax_material: cone::Material,
    wood_material: cone::Material,

    light: cone::PointLight,
    shadow: cone::PointDepthMap,
    subsurface: cone::PointSubsurfaceMap,
    shadow_renderer: cone::PointDepthMapRenderer,

    skybox: cone::SkyBox,
    env: cone::EnvironmentMap,

    update_mouse: bool,
    prev_time: std::time::Instant,
    start_time: std::time::Instant,

    width: u32,
    height: u32,

    offscreen_command: gpu::CommandBuffer,
    onscreen_command: gpu::CommandBuffer,
}

impl Cone {
    pub fn new(window: &winit::window::Window, debug: bool) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            features: gpu::DeviceFeatures::BASE | gpu::DeviceFeatures::GEOMETRY_SHADER,
            ..Default::default()
        })?;

        let mut sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        sc_desc.format = gpu::Format::Bgra8Unorm;
        let swapchain = device.create_swapchain(&surface, &mut sc_desc)?;

        let mut command_buffer = device.create_command_buffer(None)?;
        let offscreen_command = device.create_command_buffer(None)?;

        let mut encoder = gfx::CommandEncoder::new();

        println!("loading objects...");

        let mesh_small = mesh::load_meshes_from_obj(
            &mut encoder,
            &device,
            false,
            "../resources/models/dragon_small.obj",
            if debug {
                Some("mesh_small")
            } else {
                None
            },
        )?
        .remove(0);

        let mesh = mesh::load_meshes_from_obj(
            &mut encoder,
            &device,
            true,
            "../resources/models/dragon.obj",
            if debug {
                Some("mesh")
            } else {
                None
            },
        )?
        .remove(0);

        let plane = mesh::xz_plane(&mut encoder, &device, if debug { Some("plane") } else { None })?;

        let cube = mesh::cube(&mut encoder, &device, if debug { Some("cube") } else { None })?;

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

        let camera = controller.create_cam(&mut encoder, &device, if debug { Some("camera") } else { None })?;

        let buffer = cone::GeometryBuffer::new(
            &device,
            &cone::GeometryBufferDesc {
                width: WIDTH,
                height: HEIGHT,
                precision: cone::GeometryBufferPrecision::Medium,
                samples: gpu::Samples::S1,
                maps: cone::GeometryBufferDesc::ALL_MAPS,
                map_features: |s| match s {
                    "ao" => (None, Some(0.5)),
                    _ => (None, None),
                },
                depth_usage: gpu::TextureUsage::empty(),
                name: if debug {
                    Some("geometry_buffer".to_string())
                } else {
                    None
                },
            },
        )?;

        let smaa_renderer =
            ddd::utils::SMAARenderer::new(&mut encoder, &device, ddd::utils::SMAAState::LOW, None, if debug { Some("smaa") } else { None })?;

        let env_renderer = cone::EnvironmentRenderer::new(
            &mut encoder,
            &device,
            cone::EnvironmentRendererFlags::all(),
            None,
            if debug {
                Some("env")
            } else {
                None
            },
        )?;

        let solid_renderer = clay::SolidRenderer::new(&device, None, None)?;

        let ao_renderer = cone::AORenderer::new(
            &mut encoder,
            &device,
            // need to be tweaked based on scene geometry
            cone::AOParams {
                kernel_size: 8,
                radius: 1.0,
                bias: 0.005,
                power: 5.0,
                ..Default::default()
            },
            false,
            None,
            if debug {
                Some("ao")
            } else {
                None
            }
        )?;

        let bloom_renderer = cone::BloomRenderer::new(&mut encoder, &device, 0.5, 1.5, None, if debug { Some("bloom") } else { None })?;

        let tonemap_renderer = cone::GlobalToneMapRenderer::new(
            &mut encoder,
            &device,
            cone::GlobalToneMapParams::default(),
            None,
            if debug {
                Some("tonemap")
            } else {
                None
            }
        )?;
        let antialiased = gfx::GTexture2D::from_formats(
            &device,
            buffer.width(),
            buffer.height(),
            gpu::Samples::S1,
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_OUTPUT,
            1,
            gfx::alt_formats(gpu::Format::Rgba32Float),
            if debug {
                Some("antialiased")
            } else {
                None
            }
        )?
        .unwrap();

        let point_renderer =
            cone::PointLightRenderer::new(&device, cone::PointLightRendererFlags::all(), None, if debug { Some("point_renderer") } else { None })?;

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            wrap_x: gpu::WrapMode::ClampToEdge,
            wrap_y: gpu::WrapMode::ClampToEdge,
            wrap_z: gpu::WrapMode::ClampToEdge,
            min_filter: gpu::FilterMode::Linear,
            mag_filter: gpu::FilterMode::Linear,
            mipmap_filter: gpu::FilterMode::Linear,
            ..Default::default()
        })?;

        let scale = glam::Mat4::from_scale(glam::vec3(2.0, 2.0, 2.0));

        let leather_instance =
            [(glam::Mat4::from_translation(glam::vec3(-4.5, -1.0, 0.0)) * scale).into()];
        let leather_instance =
            ddd::utils::Instances::new(&mut encoder, &device, &leather_instance, if debug { Some("leather instance") } else { None })?;

        let metal_instance =
            [(glam::Mat4::from_translation(glam::vec3(-1.5, -1.0, 0.0)) * scale).into()];
        let metal_instance =
            ddd::utils::Instances::new(&mut encoder, &device, &metal_instance, if debug { Some("metal instance") } else { None })?;

        let wax_instance =
            [(glam::Mat4::from_translation(glam::vec3(1.5, -1.0, 0.0)) * scale).into()];
        let wax_instance = ddd::utils::Instances::new(&mut encoder, &device, &wax_instance, if debug { Some("wax instance") } else { None })?;

        let chrome_instance =
            [(glam::Mat4::from_translation(glam::vec3(4.5, -1.0, 0.0)) * scale).into()];
        let chrome_instance =
            ddd::utils::Instances::new(&mut encoder, &device, &chrome_instance, if debug { Some("chrome instance") } else { None })?;

        let wood_instance = [glam::Mat4::from_scale_rotation_translation(
            glam::vec3(7.0, 1.0, 7.0),
            glam::Quat::IDENTITY,
            glam::vec3(0.0, -1.0, 0.0),
        )
        .into()];
        let wood_instance =
            ddd::utils::Instances::new(&mut encoder, &device, &wood_instance, if debug { Some("wood instance") } else { None })?;

        println!("loading textures...");

        let leather_albedo_image = image::open("../resources/images/leather/color.jpg")
            .unwrap()
            .to_rgba8();
        let leather_albedo = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &leather_albedo_image,
            gpu::TextureUsage::SAMPLED,
            3,
            if debug { Some("leather albedo") } else { None },
        )?;

        let leather_roughness_image = image::open("../resources/images/leather/roughness.jpg")
            .unwrap()
            .to_luma8();
        let leather_roughness = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &leather_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            if debug {
                Some("leather roughness")
            } else {
                None
            },
        )?;

        // let leather_normal_image = image::open("../resources/images/leather/normal.jpg")
        //     .unwrap()
        //     .to_rgba8();
        // let leather_normal = gfx::Texture2D::from_image_buffer(
        //     &mut encoder,
        //     &device,
        //     &leather_normal_image,
        //     gpu::TextureUsage::SAMPLED,
        //     1,
        //     None,
        // )?;

        let leather_material = cone::Material::textured(
            &device,
            &leather_albedo,
            &leather_roughness,
            None,
            None, //Some(&leather_normal),
            &sampler,
            false,
            None,
        )
        .unwrap();

        let metal_albedo_image = image::open("../resources/images/metal/color.jpg")
            .unwrap()
            .to_rgba8();
        let metal_albedo = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &metal_albedo_image,
            gpu::TextureUsage::SAMPLED,
            3,
            if debug {
                Some("metal albedo")
            } else {
                None
            },
        )?;

        let metal_roughness_image = image::open("../resources/images/metal/roughness.jpg")
            .unwrap()
            .to_luma8();
        let metal_roughness = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &metal_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            if debug {
                Some("metal roughness")
            } else {
                None
            },
        )?;

        let metal_metallic_image = image::open("../resources/images/metal/metallic.jpg")
            .unwrap()
            .to_luma8();
        let metal_metallic = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &metal_metallic_image,
            gpu::TextureUsage::SAMPLED,
            1,
            if debug {
                Some("metal metallic")
            } else {
                None
            },
        )?;

        let metal_material = cone::Material::textured(
            &device,
            &metal_albedo,
            &metal_roughness,
            Some(&metal_metallic),
            None,
            &sampler,
            false,
            None,
        )
        .unwrap();

        let wax_material = cone::Material::constant(
            &device,
            &cone::MaterialData {
                albedo: glam::vec4(0.6, 0.2, 0.1, 1.0),
                roughness: 0.8,
                metallic: 0.0,
                subsurface: glam::vec4(0.95, 0.66, 0.35, 0.5),
            },
            None,
        )?;

        let chrome_material = cone::Material::constant(
            &device,
            &cone::MaterialData {
                albedo: glam::vec4(0.9, 0.9, 1.0, 1.0),
                roughness: 0.01,
                metallic: 1.0,
                subsurface: glam::vec4(0.0, 0.0, 0.0, 0.0),
            },
            None,
        )?;

        let wood_albedo_image = image::open("../resources/images/wood/color.jpg")
            .unwrap()
            .to_rgba8();
        let wood_albedo = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &wood_albedo_image,
            gpu::TextureUsage::SAMPLED,
            3,
            if debug {
                Some("wood albedo")
            } else {
                None
            },
        )?;

        let wood_roughness_image = image::open("../resources/images/wood/roughness.jpg")
            .unwrap()
            .to_luma8();
        let wood_roughness = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &wood_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            if debug {
                Some("wood roughness")
            } else {
                None
            },
        )?;

        let wood_normal_image = image::open("../resources/images/wood/normal.jpg")
            .unwrap()
            .to_rgba8();
        let wood_normal = gfx::Texture2D::from_image_buffer(
            &mut encoder,
            &device,
            &wood_normal_image,
            gpu::TextureUsage::SAMPLED,
            1,
            if debug {
                Some("wood normal")
            } else {
                None
            },
        )?;

        let wood_material = cone::Material::textured(
            &device,
            &wood_albedo,
            &wood_roughness,
            None,
            Some(&wood_normal),
            &sampler,
            false,
            None,
        )
        .unwrap();

        let read = BufReader::new(File::open("../resources/images/hdri/env.hdr")?);
        let decoder = image::codecs::hdr::HdrDecoder::new(read)?;
        let meta = decoder.metadata();

        // TODO: Not this
        let buf = unsafe {
            let v = decoder.read_image_hdr()?;
            let (ptr, len, cap) = v.into_raw_parts();
            Vec::from_raw_parts(ptr as *mut f32, len * 3, cap * 3)
        };
        let hdri = image::ImageBuffer::<image::Rgb<f32>, _>::from_vec(meta.width, meta.height, buf)
            .unwrap();

        let skybox = cone::new_skybox(&mut encoder, &device, hdri, 512)?;

        let env = cone::new_env_map(&mut encoder, &device, &skybox, 32, 128, 512, 512, 2048)?;

        let light_pos = glam::vec3(0.0, 2.0, 0.0);

        let light_data = cone::PointLightData::new(0.05, light_pos, [2.5; 3].into(), 0.05);
        let light = cone::PointLight::new(&mut encoder, &device, light_data, if debug { Some("light") } else { None })?;

        let light_instance = [(glam::Mat4::from_translation(light_pos)
            * glam::Mat4::from_scale(glam::vec3(0.1, 0.1, 0.1)))
        .into()];
        let light_instance =
            ddd::utils::Instances::new(&mut encoder, &device, &light_instance, if debug { Some("light instance") } else { None })?;

        let shadow = cone::PointDepthMap::new(
            &mut encoder,
            &device,
            cone::PointDepthData::from_light(&light.data, 0.1, 20.0, 0.05, 0.005),
            512,
            if debug {
                Some("shadow map")
            } else {
                None
            },
        )?;

        let subsurface = cone::PointSubsurfaceMap::new(
            &mut encoder,
            &device,
            cone::PointDepthData::from_light(&light.data, 0.1, 20.0, 0.05, 0.005),
            512,
            64,
            if debug {
                Some("subsurface map")
            } else {
                None
            },
        )?;

        let shadow_renderer = cone::PointDepthMapRenderer::new(
            &device,
            gpu::CullFace::Front,
            gpu::FrontFace::Clockwise,
            None,
            if debug {
                Some("shadow renderer")
            } else {
                None
            },
        )?;

        println!("pre-computing lookup tables...");

        encoder.submit(&mut command_buffer, true)?;

        let display_renderer = ddd::utils::CopyRenderer::new(&device, None, None)?;

        let query1 = device.create_time_query(16, None)?;
        let query2 = device.create_time_query(2, None)?;

        let mut s = Self {
            _instance: instance,
            _surface: surface,
            device,
            swapchain,

            controller,
            camera,
            buffer,
            env_renderer,
            smaa_renderer,
            point_renderer,
            ao_renderer,
            display_renderer,
            solid_renderer,
            bloom_renderer,
            tonemap_renderer,
            antialiased,

            query1,
            query2,

            mesh,
            mesh_small,
            plane,
            cube,

            metal_material,
            leather_material,
            chrome_material,
            wood_material,
            wax_material,
            skybox,
            env,

            leather_instance,
            metal_instance,
            wax_instance,
            chrome_instance,
            light_instance,
            wood_instance,

            light,
            shadow,
            subsurface,
            shadow_renderer,

            update_mouse: true,
            prev_time: std::time::Instant::now(),
            start_time: std::time::Instant::now(),

            width: WIDTH,
            height: HEIGHT,

            onscreen_command: command_buffer,
            offscreen_command,
        };

        s.render_offscreen()?;

        Ok(s)
    }

    fn render_offscreen(&mut self) -> Result<(), anyhow::Error> {
        let mut encoder = gfx::CommandEncoder::new();

        encoder.reset_time_query_ref(&self.query1, 0, 16);
        encoder.write_timestamp_ref(&self.query1, 0, gpu::PipelineStage::TopOfPipe);

        self.shadow_renderer.single_pass(
            &mut encoder,
            &self.device,
            &self.shadow,
            [
                (&self.mesh_small as _, &self.leather_instance),
                (&self.mesh_small as _, &self.metal_instance),
                (&self.mesh_small as _, &self.wax_instance),
                (&self.mesh_small as _, &self.chrome_instance),
            ]
            .into_iter(),
            true,
        )?;

        self.shadow_renderer.single_pass(
            &mut encoder,
            &self.device,
            &self.subsurface,
            std::iter::once((&self.mesh_small as _, &self.wax_instance)),
            true,
        )?;

        encoder.write_timestamp_ref(&self.query1, 1, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 2, gpu::PipelineStage::TopOfPipe);

        self.metal_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.mesh as _, &self.metal_instance)),
            true,
        )?;

        self.wax_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.mesh as _, &self.wax_instance)),
            false,
        )?;

        self.leather_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.mesh as _, &self.leather_instance)),
            false,
        )?;

        self.chrome_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.mesh as _, &self.chrome_instance)),
            false,
        )?;

        self.wood_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.plane as _, &self.wood_instance)),
            false,
        )?;

        encoder.write_timestamp_ref(&self.query1, 3, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 4, gpu::PipelineStage::TopOfPipe);

        self.ao_renderer
            .pass(&mut encoder, &self.device, &self.buffer, &self.camera, 3.0)?;

        // encoder.clear_texture(
        //     self.buffer.get("ao").unwrap().whole_slice_ref(),
        //     gpu::ClearValue::ColorFloat([1.0; 4]),
        // );

        encoder.write_timestamp_ref(&self.query1, 5, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 6, gpu::PipelineStage::TopOfPipe);

        self.env_renderer.environment_pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            &self.env,
            1.0,
            true,
        )?;

        encoder.write_timestamp_ref(&self.query1, 7, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 8, gpu::PipelineStage::TopOfPipe);

        // self.point_renderer.base_pass(
        //     &mut encoder,
        //     &self.device,
        //     &self.buffer,
        //     &self.camera,
        //     Some(&self.light),
        //     1.0,
        //     false,
        // )?;

        // self.point_renderer.shadow_pass(
        //     &mut encoder,
        //     &self.device,
        //     &self.buffer,
        //     &self.camera,
        //     Some((&self.light, &self.shadow)),
        //     1.0,
        //     15,
        //     false
        // )?;

        self.point_renderer.subsurface_pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.light, &self.shadow, &self.subsurface)),
            1.0,
            15,
            15,
            false,
        )?;

        encoder.write_timestamp_ref(&self.query1, 9, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 10, gpu::PipelineStage::TopOfPipe);

        self.solid_renderer.pass(
            &mut encoder,
            &self.device,
            gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&self.buffer.get("output").unwrap().view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            },
            gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&self.buffer.depth.view),
                    gpu::ClearValue::Depth(1.0),
                ),
                load: gpu::LoadOp::Load,
                store: gpu::StoreOp::Store,
            },
            [(&self.cube as _, &self.light_instance, [2.0, 2.0, 2.0, 1.0])],
            &self.camera,
        )?;

        self.env_renderer.skybox_pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            &self.skybox,
            1.0,
            false,
        )?;

        encoder.write_timestamp_ref(&self.query1, 11, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 12, gpu::PipelineStage::TopOfPipe);

        self.bloom_renderer
            .pass(&mut encoder, &self.device, &self.buffer, 4)?;

        encoder.write_timestamp_ref(&self.query1, 13, gpu::PipelineStage::BottomOfPipe);
        encoder.write_timestamp_ref(&self.query1, 14, gpu::PipelineStage::TopOfPipe);

        self.smaa_renderer.pass(
            &mut encoder,
            &self.device,
            &self.buffer.get("output").unwrap().view,
            None,
            gfx::Attachment {
                raw: gpu::Attachment::View(
                    Cow::Borrowed(&self.antialiased.view),
                    gpu::ClearValue::ColorFloat([0.0; 4]),
                ),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            },
        )?;

        encoder.write_timestamp_ref(&self.query1, 15, gpu::PipelineStage::BottomOfPipe);

        encoder.record(&mut self.offscreen_command, false)?;

        Ok(())
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

        encoder.reset_time_query_ref(&self.query2, 0, 2);

        self.light.data.position.z = (self.start_time.elapsed().as_secs_f32() / 2.0).sin() * 6.0;
        self.shadow.data = cone::PointDepthData::from_light(
            &self.light.data,
            0.01,
            self.shadow.data.z_far,
            self.shadow.data.strength,
            self.shadow.data.bias,
        );
        self.subsurface.data = cone::PointDepthData::from_light(
            &self.light.data,
            0.01,
            self.subsurface.data.z_far,
            self.subsurface.data.strength,
            self.subsurface.data.bias,
        );
        let light_instances = [(glam::Mat4::from_translation(self.light.data.position)
            * glam::Mat4::from_scale(glam::vec3(0.1, 0.1, 0.1)))
        .into()];

        self.light.update_gpu_ref(&mut encoder);
        self.shadow.update_gpu_ref(&mut encoder);
        self.subsurface.update_gpu_ref(&mut encoder);
        self.light_instance
            .update_gpu(&mut encoder, &light_instances);

        self.controller
            .update_cam_owned(&mut encoder, &mut self.camera);

        encoder.write_timestamp_ref(&self.query2, 0, gpu::PipelineStage::TopOfPipe);

        self.tonemap_renderer.pass(
            &mut encoder,
            &self.device,
            &self.antialiased.view,
            gfx::Attachment {
                raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
                load: gpu::LoadOp::DontCare,
                store: gpu::StoreOp::Store,
            },
        )?;

        encoder.write_timestamp_ref(&self.query2, 1, gpu::PipelineStage::BottomOfPipe);

        // for debugging
        // try taking a look at the geometry buffers other frames
        // self.display_renderer.pass(
        //     &mut encoder,
        //     &self.device,
        //     &self.buffer.get("view_pos").unwrap().view,
        //     gfx::Attachment {
        //         raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
        //         load: gpu::LoadOp::Clear,
        //         store: gpu::StoreOp::Store,
        //     },
        // )?;

        encoder.submit(&mut self.onscreen_command, true)?;

        self.offscreen_command.submit()?;

        self.swapchain.present(frame)?;

        let durations = self.query1.get_paired_times(0, 16)?;

        println!("fps     : {}", 1.0 / dt);
        println!("");
        let names = &[
            "shadows : ",
            "geometry: ",
            "ao      : ",
            "env     : ",
            "light   : ",
            "sky+fwd : ",
            "bloom   : ",
            "smaa    : ",
        ];
        for (duration, name) in durations.iter().zip(names) {
            println!("{}{:?}", name, duration);
        }

        let tonemap_duration = self.query2.get_paired_times(0, 2)?[0];
        println!("tonemap : {:?}", tonemap_duration);

        println!("");

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("cone")
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut cone = Cone::new(&window, true).unwrap();

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
