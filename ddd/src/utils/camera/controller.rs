use super::CameraData;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub enum CameraMoveDirection {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

pub type CameraLookDirection = (f32, f32);

pub trait CameraController {
    /// should update the controllers internal data on the camera
    fn move_cam(&mut self, _dir: CameraMoveDirection, _dt: f32) {}

    /// should update the controllers internal data on the camera
    fn look_cam(&mut self, _dir: CameraLookDirection, _dt: f32) {}

    /// should generate view and projection matrices for the controller
    fn cam_data(&self) -> CameraData;

    /// create a new camera from the controller
    fn create_cam(
        &self,
        encoder: &mut gfx::CommandEncoder,
        device: &gpu::Device,
        name: Option<&str>,
    ) -> Result<super::Camera, gpu::Error> {
        let data = self.cam_data();
        super::Camera::new(encoder, device, data, name)
    }

    /// create new view/projection matrices then push to gpu
    fn update_cam_ref<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        camera: &'a mut gfx::Uniform<CameraData>,
    ) {
        let data = self.cam_data();
        camera.data = data;
        camera.update_gpu_ref(encoder)
    }

    fn update_cam_owned<'a>(
        &self,
        encoder: &mut gfx::CommandEncoder<'a>,
        camera: &mut gfx::Uniform<CameraData>,
    ) {
        let data = self.cam_data();
        camera.data = data;
        camera.update_gpu_owned(encoder)
    }
}

/// A basic fps camera controller type supporting either perspective or orthographic projections
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GameController {
    pub position: glam::Vec3,
    pub forward: glam::Vec3,
    pub world_up: glam::Vec3,
    pub side: glam::Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub flip_y: bool,
    pub projection: glam::Mat4,
}

impl Default for GameController {
    fn default() -> Self {
        Self::from_flipped_perspective(
            glam::vec3(0.0, 0.0, 0.0),
            0.0,
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.01,
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.01,
            100.0,
            false,
        )
    }
}

impl GameController {
    pub fn from_raw(
        position: glam::Vec3,
        forward: glam::Vec3,
        world_up: glam::Vec3,
        side: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        flip_y: bool,
        projection: glam::Mat4,
    ) -> Self {
        Self {
            position,
            forward,
            world_up,
            side,
            pitch,
            yaw,
            speed,
            sensitivity,
            projection,
            flip_y,
        }
    }

    pub fn new(
        position: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        flip_y: bool,
        projection: glam::Mat4,
    ) -> Self {
        let forward = glam::Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let world_up = glam::Vec3::new(0.0, 1.0, 0.0);
        let side = world_up.cross(forward);
        Self::from_raw(
            position,
            forward,
            world_up,
            side,
            pitch,
            yaw,
            speed,
            sensitivity,
            flip_y,
            projection,
        )
    }

    /// Create a new camera with a projection
    pub fn from_perspective(
        position: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        fovy: f32,
        aspect: f32,
        znear: f32,
        zfar: f32,
        flip_y: bool,
    ) -> Self {
        let projection = glam::Mat4::perspective_rh(fovy, aspect, znear, zfar);
        Self::new(position, pitch, yaw, speed, sensitivity, flip_y, projection)
    }

    /// Create a new camera with a flipped perspective projection that makes y look up
    /// This is useful as 3d models are often defined with opengl coordinates in mind
    pub fn from_flipped_perspective(
        position: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        fovy: f32,
        aspect: f32,
        znear: f32,
        zfar: f32,
        flip_y: bool,
    ) -> Self {
        let t = (fovy / 2.0).tan();
        let sy = 1.0 / t;
        let sx = sy / aspect;
        let nmf = znear - zfar;
        let projection = glam::Mat4::from_cols(
            glam::vec4(sx, 0.0, 0.0, 0.0),
            glam::vec4(0.0, -sy, 0.0, 0.0),
            glam::vec4(0.0, 0.0, zfar / nmf, -1.0),
            glam::vec4(0.0, 0.0, znear * zfar / nmf, 0.0),
        );
        Self::new(position, pitch, yaw, speed, sensitivity, flip_y, projection)
    }

    /// Create a new camera with an orthographic projection
    pub fn from_orthographic(
        position: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
        flip_y: bool,
    ) -> Self {
        let projection = glam::Mat4::orthographic_rh(left, right, bottom, top, znear, zfar);
        Self::new(position, pitch, yaw, speed, sensitivity, flip_y, projection)
    }

    /// Create a new camera with a flipped orthographic projection that makes y look up
    /// This is useful as 3d models are often defined with opengl coordinates in mind
    pub fn from_flipped_orthographic(
        position: glam::Vec3,
        pitch: f32,
        yaw: f32,
        speed: f32,
        sensitivity: f32,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
        flip_y: bool,
    ) -> Self {
        let rml = right - left;
        let rpl = right + left;
        let tmb = top - bottom;
        let tpb = top + bottom;
        let fmn = zfar - znear;
        let projection = glam::Mat4::from_cols(
            glam::vec4(2.0 / rml, 0.0, 0.0, 0.0),
            glam::vec4(0.0, -2.0 / tmb, 0.0, 0.0),
            glam::vec4(0.0, 0.0, -1.0 / fmn, 0.0),
            glam::vec4(-(rpl / rml), -(tpb / tmb), -(znear / fmn), 1.0),
        );
        Self::new(position, pitch, yaw, speed, sensitivity, flip_y, projection)
    }
}

impl CameraController for GameController {
    fn move_cam(&mut self, dir: CameraMoveDirection, dt: f32) {
        match dir {
            CameraMoveDirection::Forward => self.position += self.forward * self.speed * dt,
            CameraMoveDirection::Back => self.position -= self.forward * self.speed * dt,
            CameraMoveDirection::Left => self.position -= self.side * self.speed * dt,
            CameraMoveDirection::Right => self.position += self.side * self.speed * dt,
            CameraMoveDirection::Up => self.position += self.world_up * self.speed * dt,
            CameraMoveDirection::Down => self.position -= self.world_up * self.speed * dt,
        }
    }

    fn look_cam(&mut self, dir: CameraLookDirection, dt: f32) {
        self.yaw += dir.0 * self.sensitivity * dt;
        if self.flip_y {
            self.pitch -= dir.1 * self.sensitivity * dt;
        } else {
            self.pitch += dir.1 * self.sensitivity * dt;
        }
        self.pitch = self.pitch.min(1.53343).max(-1.53343);
        self.forward = glam::vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        self.side = self.world_up.cross(self.forward);
    }

    fn cam_data(&self) -> CameraData {
        let view =
            glam::Mat4::look_at_rh(self.position, self.position + self.forward, self.world_up);
        super::CameraData {
            view,
            projection: self.projection,
            position: self.position,
        }
    }
}
