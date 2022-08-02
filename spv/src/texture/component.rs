
use crate::PrimitiveType;

#[derive(Clone, Copy, Debug)]
pub enum Component {
    Float,
    Double,
    Int,
    UInt,
}

impl From<Component> for PrimitiveType {
    fn from(c: Component) -> Self {
        Self::from(&c)
    }
}

impl From<&'_ Component> for PrimitiveType {
    fn from(c: &'_ Component) -> Self {
        match c {
            Component::Float => Self::Float,
            Component::Double => Self::Double,
            Component::Int => Self::Int,
            Component::UInt => Self::UInt,
        }
    }
}

impl Component {
    pub(crate) fn base_type(&self, builder: &mut rspirv::dr::Builder) -> u32 {
        PrimitiveType::from(*self).base_type(builder)
    }
}

pub trait AsComponent {
    const COMPONENT: Component;

    type Read;
}

impl AsComponent for crate::Float {
    const COMPONENT: Component = Component::Float;

    type Read = crate::Vec4;
}

impl AsComponent for crate::Double {
    const COMPONENT: Component = Component::Double;

    type Read = crate::DVec4;
}

impl AsComponent for crate::Int {
    const COMPONENT: Component = Component::Int;

    type Read = crate::IVec4;
}

impl AsComponent for crate::UInt {
    const COMPONENT: Component = Component::UInt;

    type Read = crate::UVec2;
}