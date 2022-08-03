#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InstanceData {
    pub model: glam::Mat4,
}

impl From<glam::Mat4> for InstanceData {
    fn from(model: glam::Mat4) -> Self {
        Self { model }
    }
}

unsafe impl bytemuck::Pod for InstanceData {}
unsafe impl bytemuck::Zeroable for InstanceData {}

unsafe impl spv::AsSpvStruct for InstanceData {
    const DESC: spv::StructDesc = spv::StructDesc {
        name: "InstanceData",
        names: &["model"],
        fields: &[spv::DataType::Primitive(spv::PrimitiveType::Mat4)],
    };

    fn fields<'a>(&'a self) -> Vec<&'a dyn spv::AsData> {
        vec![&self.model]
    }
}

pub type Instance = gfx::Uniform<InstanceData>;
pub type Instances = gfx::Storage<InstanceData>;
