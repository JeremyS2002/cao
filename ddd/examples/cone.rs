#![feature(vec_into_raw_parts)]

use ddd::cone;
use ddd::glam;
use ddd::prelude::*;
use gfx::image;

use std::fs::File;
use std::io::BufReader;

use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use winit_input_helper::WinitInputHelper;

use std::convert::TryFrom;

const WIDTH: u32 = 850;
const HEIGHT: u32 = 850;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct Vertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    uv: glam::Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl gfx::Vertex for Vertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
            "in_normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            "in_uv" => Some((
                std::mem::size_of::<glam::Vec3>() as u32 * 2,
                gpu::VertexFormat::Vec2,
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct TVertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    tangent: glam::Vec3,
    uv: glam::Vec2,
}

impl TVertex {
    pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position: pos.into(),
            normal: normal.into(),
            tangent: glam::vec3(0.0, 0.0, 0.0),
            uv: uv.into(),
        }
    }
}

unsafe impl bytemuck::Pod for TVertex {}
unsafe impl bytemuck::Zeroable for TVertex {}

impl gfx::Vertex for TVertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
            "in_normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            "in_tangent" => Some((
                std::mem::size_of::<glam::Vec3>() as u32 * 2,
                gpu::VertexFormat::Vec3,
            )),
            "in_uv" => Some((
                std::mem::size_of::<glam::Vec3>() as u32 * 3,
                gpu::VertexFormat::Vec2,
            )),
            _ => None,
        }
    }
}

impl mesh::TangentVertex for TVertex {
    fn position(&self) -> glam::Vec3 {
        self.position
    }

    fn uv(&self) -> glam::Vec2 {
        self.uv
    }

    fn set_tangent(&mut self, tangent: glam::Vec3) {
        self.tangent = tangent;
    }
}

pub fn load_obj(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    path: &str,
) -> Result<gfx::IndexedMesh<Vertex>, anyhow::Error> {
    let (models, _) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: false,
            ignore_lines: true,
            ignore_points: true,
            single_index: true,
        },
    )?;

    let mesh = &models[0].mesh;
    let vertices = mesh
        .positions
        .chunks(3)
        .zip(mesh.normals.chunks(3))
        .zip(mesh.texcoords.chunks(2))
        .map(|((position, normal), uv)| Vertex {
            position: <[f32; 3]>::try_from(position).unwrap().into(),
            normal: <[f32; 3]>::try_from(normal).unwrap().into(),
            uv: <[f32; 2]>::try_from(uv).unwrap().into(),
        })
        .collect::<Vec<_>>();

    let indices = &*mesh.indices;

    Ok(gfx::IndexedMesh::new(
        encoder, device, &vertices, indices, None,
    )?)
}

#[allow(dead_code)]
pub struct Cone {
    _instance: gpu::Instance,
    _surface: gpu::Surface,
    device: gpu::Device,
    swapchain: gpu::Swapchain,

    controller: ddd::utils::GameController,
    camera: ddd::utils::Camera,
    buffer: cone::GeometryBuffer,
    env_renderer: cone::EnvironmentRenderer,
    point_renderer: cone::PointLightRenderer,
    smaa_renderer: cone::SMAARenderer,
    display_renderer: cone::DisplayRenderer,

    monkey: gfx::IndexedMesh<Vertex>,
    plane: gfx::IndexedMesh<TVertex>,

    leather_instance: ddd::utils::Instance,
    metal_instance: ddd::utils::Instance,
    wax_instance: ddd::utils::Instance,
    chrome_instance: ddd::utils::Instance,

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
    pub fn new(window: &winit::window::Window) -> Result<Self, anyhow::Error> {
        let instance = gpu::Instance::new(&gpu::InstanceDesc::default())?;

        let surface = instance.create_surface(window)?;

        let device = instance.create_device(&gpu::DeviceDesc {
            compatible_surfaces: &[&surface],
            features: gpu::DeviceFeatures::BASE | gpu::DeviceFeatures::SAMPLER_ANISOTROPY,
            ..Default::default()
        })?;

        let mut sc_desc = gpu::SwapchainDesc::from_surface(&surface, &device)?;
        sc_desc.format = gpu::Format::Bgra8Unorm;
        let swapchain = device.create_swapchain(&surface, &mut sc_desc)?;

        let mut command_buffer = device.create_command_buffer(None)?;
        let offscreen_command = device.create_command_buffer(None)?;

        let mut encoder = gfx::CommandEncoder::new();

        println!("loading objects...");

        let monkey = load_obj(&mut encoder, &device, "../resources/models/suzanne.obj")?;
        let mut plane_vertices = [
            TVertex::new([-1.0, 0.0, -1.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
            TVertex::new([1.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0]),
            TVertex::new([1.0, 0.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
            TVertex::new([-1.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];
        let plane_indices = [0, 1, 2, 2, 3, 0];
        mesh::calc_tangent_indexed(&mut plane_vertices, &plane_indices);

        let plane =
            gfx::IndexedMesh::new(&mut encoder, &device, &plane_vertices, &plane_indices, None)?;

        let controller = ddd::utils::GameController::from_flipped_perspective(
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

        let buffer =
            cone::GeometryBuffer::new(&device, WIDTH, HEIGHT, gpu::Samples::S1, Some("buffers"))?;

        let smaa_renderer = cone::SMAARenderer::new(
            &mut encoder,
            &device,
            &buffer.get("output").unwrap().view,
            cone::SMAAState {
                edge: cone::SMAAEdgeMethod::Luma,
                quality: cone::SMAAQuality::Medium,
            },
            cone::DisplayFlags::all(),
            None,
        )?;

        let env_renderer = cone::EnvironmentRenderer::new(
            &mut encoder,
            &device,
            cone::EnvironmentRendererFlags::all(),
            None,
        )?;

        let point_renderer = cone::PointLightRenderer::new(
            &mut encoder,
            &device,
            cone::PointLightRendererFlags::FULL,
            None,
        )?;

        let sampler = device.create_sampler(&gpu::SamplerDesc {
            ..Default::default()
        })?;

        let leather_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            glam::Mat4::from_translation(glam::vec3(-4.5, 0.0, 0.0)).into(),
            None,
        )?;

        let metal_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            glam::Mat4::from_translation(glam::vec3(-1.5, 0.0, 0.0)).into(),
            None,
        )?;

        let wax_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            glam::Mat4::from_translation(glam::vec3(1.5, 0.0, 0.0)).into(),
            None,
        )?;

        let chrome_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            glam::Mat4::from_translation(glam::vec3(4.5, 0.0, 0.0)).into(),
            None,
        )?;

        let wood_instance = ddd::utils::Instance::new(
            &mut encoder,
            &device,
            glam::Mat4::from_scale_rotation_translation(
                glam::vec3(7.0, 1.0, 7.0),
                glam::Quat::IDENTITY,
                glam::vec3(0.0, -1.5, 0.0),
            )
            .into(),
            None,
        )?;

        println!("loading textures...");

        let leather_albedo_image = image::open("../resources/images/leather/color.jpg")
            .unwrap()
            .to_rgba8();
        let leather_albedo = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &leather_albedo_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let leather_roughness_image = image::open("../resources/images/leather/roughness.jpg")
            .unwrap()
            .to_luma8();
        let leather_roughness = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &leather_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        // let leather_normal_image = image::open("../resources/images/leather/normal.jpg")
        //     .unwrap()
        //     .to_rgba8();
        // let leather_normal = gfx::Texture2D::from_image(
        //     &mut encoder,
        //     &device,
        //     &leather_normal_image,
        //     gpu::TextureUsage::SAMPLED,
        //     1,
        //     None,
        // )?;

        let leather_material = cone::Material::textured(
            &device,
            &camera,
            &leather_instance,
            &leather_albedo,
            &leather_roughness,
            None,
            None,//Some(&leather_normal),
            &sampler,
        )
        .unwrap();

        let metal_albedo_image = image::open("../resources/images/metal/color.jpg")
            .unwrap()
            .to_rgba8();
        let metal_albedo = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &metal_albedo_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let metal_roughness_image = image::open("../resources/images/metal/roughness.jpg")
            .unwrap()
            .to_luma8();
        let metal_roughness = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &metal_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let metal_metallic_image = image::open("../resources/images/metal/metallic.jpg")
            .unwrap()
            .to_luma8();
        let metal_metallic = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &metal_metallic_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let metal_material = cone::Material::textured(
            &device,
            &camera,
            &metal_instance,
            &metal_albedo,
            &metal_roughness,
            Some(&metal_metallic),
            None,
            &sampler,
        )
        .unwrap();

        let wax_material = cone::Material::constant(
            &device,
            &camera,
            &wax_instance,
            &cone::MaterialData {
                albedo: glam::vec4(1.0, 0.0, 0.0, 0.99),
                roughness: 0.6,
                metallic: 0.0,
                subsurface: glam::vec4(0.95, 0.66, 0.35, 0.9),
            },
        )?;

        let chrome_material = cone::Material::constant(
            &device,
            &camera,
            &chrome_instance,
            &cone::MaterialData {
                albedo: glam::vec4(0.9, 0.9, 1.0, 1.0),
                roughness: 0.1,
                metallic: 0.9,
                ..Default::default()
            },
        )?;

        let wood_albedo_image = image::open("../resources/images/wood/color.jpg")
            .unwrap()
            .to_rgba8();
        let wood_albedo = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &wood_albedo_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let wood_roughness_image = image::open("../resources/images/wood/roughness.jpg")
            .unwrap()
            .to_luma8();
        let wood_roughness = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &wood_roughness_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let wood_normal_image = image::open("../resources/images/wood/normal.jpg")
            .unwrap()
            .to_rgba8();
        let wood_normal = gfx::Texture2D::from_image(
            &mut encoder,
            &device,
            &wood_normal_image,
            gpu::TextureUsage::SAMPLED,
            1,
            None,
        )?;

        let wood_material = cone::Material::textured(
            &device,
            &camera,
            &wood_instance,
            &wood_albedo,
            &wood_roughness,
            None,
            Some(&wood_normal),
            &sampler,
        )
        .unwrap();

        let read = BufReader::new(File::open("../resources/images/hdri/night.hdr")?);
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

        let skybox = cone::new_skybox(&mut encoder, &device, &hdri, 512)?;

        let env = cone::new_env_map(&mut encoder, &device, &skybox, 32, 4096)?;

        let light = cone::PointLight::new(
            &mut encoder,
            &device,
            cone::PointLightData::new(
                0.5,
                0.0,
                0.025,
                [0.0, 2.0, 0.0].into(),
                [5.0; 3].into(),
                0.05,
            ),
            None,
        )?;

        let shadow = cone::PointDepthMap::new(
            &mut encoder,
            &device,
            cone::PointDepthData::from_light(&light.data, 0.1, 20.0, 0.05, 0.005),
            1024,
            1024,
        )?;

        let subsurface = cone::PointSubsurfaceMap::new(
            &mut encoder,
            &device,
            cone::PointDepthData::from_light(&light.data, 0.1, 20.0, 0.05, 0.005),
            1024,
            1024,
            512,
        )?;

        let shadow_renderer =
            cone::PointDepthMapRenderer::new(&device, Some(gpu::FrontFace::Clockwise))?;

        encoder.submit(&mut command_buffer, true)?;

        let display_renderer = cone::DisplayRenderer::new(
            &device, 
            &buffer.get("output").unwrap().view, 
            cone::DisplayFlags::all(),
            None,
        )?;

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
            display_renderer,

            monkey,
            plane,
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

        println!("recording commands...");

        s.render_offscreen()?;

        println!("done!");

        Ok(s)
    }

    fn render_offscreen(&mut self) -> Result<(), anyhow::Error> {
        let mut encoder = gfx::CommandEncoder::new();

        self.shadow_renderer.single_pass(
            &mut encoder,
            &self.device,
            &self.shadow,
            [
                (&self.monkey as _, &self.leather_instance),
                (&self.monkey as _, &self.metal_instance),
                (&self.monkey as _, &self.wax_instance),
                (&self.monkey as _, &self.chrome_instance),
            ]
            .into_iter(),
            true,
        )?;

        self.shadow_renderer.single_pass(
            &mut encoder,
            &self.device,
            &self.subsurface,
            std::iter::once((&self.monkey as _, &self.wax_instance)),
            true,
        )?;

        self.metal_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            Some(&self.monkey as _),
            true,
        )?;

        self.leather_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            Some(&self.monkey as _),
            false,
        )?;

        self.chrome_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            Some(&self.monkey as _),
            false,
        )?;

        self.wax_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            Some(&self.monkey as _),
            false,
        )?;

        self.wood_material.pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            Some(&self.plane as _),
            false,
        )?;

        self.env_renderer
            .ambient_pass(&mut encoder, &self.device, &self.buffer, 0.1, true)?;

        self.env_renderer.environment_pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            &self.env,
            1.0,
            false,
        )?;

        // self.point_renderer.base_full_pass(
        //     &mut encoder,
        //     &self.device,
        //     &self.buffer,
        //     &self.camera,
        //     Some(&self.light),
        //     1.0,
        //     false,
        // )?;

        // self.point_renderer.shadow_full_pass(
        //     &mut encoder,
        //     &self.device,
        //     &self.buffer,
        //     &self.camera,
        //     Some((&self.light, &self.shadow)),
        //     1.0,
        //     25,
        //     false
        // )?;

        self.point_renderer.subsurface_full_pass(
            &mut encoder,
            &self.device,
            &self.buffer,
            &self.camera,
            Some((&self.light, &self.shadow, &self.subsurface)),
            1.0,
            25,
            25,
            false,
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

        self.light.update_gpu_ref(&mut encoder);
        self.shadow.update_gpu_ref(&mut encoder);
        self.subsurface.update_gpu_ref(&mut encoder);

        self.controller
            .update_cam_owned(&mut encoder, &mut self.camera);

        self.smaa_renderer.aces(
            &mut encoder,
            &self.device,
            gfx::Attachment {
                raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
                load: gpu::LoadOp::Clear,
                store: gpu::StoreOp::Store,
            },
        )?;

        // self.display_renderer.clip(
        //     &mut encoder,
        //     &self.device,
        //     gfx::Attachment {
        //         raw: gpu::Attachment::Swapchain(&frame, gpu::ClearValue::ColorFloat([0.0; 4])),
        //         load: gpu::LoadOp::Clear,
        //         store: gpu::StoreOp::Store,
        //     },
        // )?;

        encoder.submit(&mut self.onscreen_command, true)?;

        self.offscreen_command.submit()?;

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

    let mut cone = Cone::new(&window).unwrap();

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