use slab::Slab;

use crate::data::PrimitiveType;




#[derive(Default, Clone, Debug)]
pub(crate) struct Variables {
    pub(crate) boolean: Slab<Option<String>>,
    pub(crate) int: Slab<Option<String>>,
    pub(crate) uint: Slab<Option<String>>,
    pub(crate) float: Slab<Option<String>>,
    pub(crate) double: Slab<Option<String>>,
    pub(crate) ivec2: Slab<Option<String>>,
    pub(crate) ivec3: Slab<Option<String>>,
    pub(crate) ivec4: Slab<Option<String>>,
    pub(crate) uvec2: Slab<Option<String>>,
    pub(crate) uvec3: Slab<Option<String>>,
    pub(crate) uvec4: Slab<Option<String>>,
    pub(crate) vec2: Slab<Option<String>>,
    pub(crate) vec3: Slab<Option<String>>,
    pub(crate) vec4: Slab<Option<String>>,
    pub(crate) dvec2: Slab<Option<String>>,
    pub(crate) dvec3: Slab<Option<String>>,
    pub(crate) dvec4: Slab<Option<String>>,
    pub(crate) mat2: Slab<Option<String>>,
    pub(crate) mat3: Slab<Option<String>>,
    pub(crate) mat4: Slab<Option<String>>,
    pub(crate) dmat2: Slab<Option<String>>,
    pub(crate) dmat3: Slab<Option<String>>,
    pub(crate) dmat4: Slab<Option<String>>,
}

impl Variables {
    pub fn get_new_id(&mut self, ty: crate::data::PrimitiveType) -> usize {
        match ty {
            crate::data::PrimitiveType::Bool => self.boolean.insert(None),
            crate::data::PrimitiveType::Int => self.int.insert(None),
            crate::data::PrimitiveType::UInt => self.uint.insert(None),
            crate::data::PrimitiveType::Float => self.float.insert(None),
            crate::data::PrimitiveType::Double => self.double.insert(None),
            crate::data::PrimitiveType::IVec2 => self.ivec2.insert(None),
            crate::data::PrimitiveType::IVec3 => self.ivec3.insert(None),
            crate::data::PrimitiveType::IVec4 => self.ivec4.insert(None),
            crate::data::PrimitiveType::UVec2 => self.uvec2.insert(None),
            crate::data::PrimitiveType::UVec3 => self.uvec3.insert(None),
            crate::data::PrimitiveType::UVec4 => self.uvec4.insert(None),
            crate::data::PrimitiveType::Vec2 => self.vec2.insert(None),
            crate::data::PrimitiveType::Vec3 => self.vec3.insert(None),
            crate::data::PrimitiveType::Vec4 => self.vec4.insert(None),
            crate::data::PrimitiveType::DVec2 => self.dvec2.insert(None),
            crate::data::PrimitiveType::DVec3 => self.dvec3.insert(None),
            crate::data::PrimitiveType::DVec4 => self.dvec4.insert(None),
            crate::data::PrimitiveType::Mat2 => self.mat2.insert(None),
            crate::data::PrimitiveType::Mat3 => self.mat3.insert(None),
            crate::data::PrimitiveType::Mat4 => self.mat4.insert(None),
            crate::data::PrimitiveType::DMat2 => self.dmat2.insert(None),
            crate::data::PrimitiveType::DMat3 => self.dmat3.insert(None),
            crate::data::PrimitiveType::DMat4 => self.dmat4.insert(None),
        }
    }

    pub fn name_var(&mut self, ty: PrimitiveType, id: usize, name: String) {
        match ty {
            PrimitiveType::Bool => *self.boolean.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Int => *self.int.get_mut(id).unwrap() = Some(name),
            PrimitiveType::UInt => *self.uint.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Float => *self.float.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Double => *self.double.get_mut(id).unwrap() = Some(name),
            PrimitiveType::IVec2 => *self.ivec2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::IVec3 => *self.ivec3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::IVec4 => *self.ivec4.get_mut(id).unwrap() = Some(name),
            PrimitiveType::UVec2 => *self.uvec2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::UVec3 => *self.uvec3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::UVec4 => *self.uvec4.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Vec2 => *self.vec2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Vec3 => *self.vec3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Vec4 => *self.vec4.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DVec2 => *self.dvec2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DVec3 => *self.dvec3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DVec4 => *self.dvec4.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Mat2 => *self.mat2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Mat3 => *self.mat3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::Mat4 => *self.mat4.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DMat2 => *self.dmat2.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DMat3 => *self.dmat3.get_mut(id).unwrap() = Some(name),
            PrimitiveType::DMat4 => *self.dmat4.get_mut(id).unwrap() = Some(name),
        }
    }
}