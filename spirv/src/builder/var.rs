use slab::Slab;

use crate::data::DataType;




#[derive(Default, Clone, Debug)]
pub(crate) struct Variables {
    pub(crate) boolean: Slab<Option<String>>,
    pub(crate) int: Slab<Option<String>>,
    pub(crate) uint: Slab<Option<String>>,
    pub(crate) float: Slab<Option<String>>,
    pub(crate) double: Slab<Option<String>>,
    pub(crate) bvec2: Slab<Option<String>>,
    pub(crate) bvec3: Slab<Option<String>>,
    pub(crate) bvec4: Slab<Option<String>>,
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
    pub fn get_new_id(&mut self, ty: crate::data::DataType) -> usize {
        match ty {
            crate::data::DataType::Bool => self.boolean.insert(None),
            crate::data::DataType::Int => self.int.insert(None),
            crate::data::DataType::UInt => self.uint.insert(None),
            crate::data::DataType::Float => self.float.insert(None),
            crate::data::DataType::Double => self.double.insert(None),
            crate::data::DataType::BVec2 => self.bvec2.insert(None),
            crate::data::DataType::BVec3 => self.bvec3.insert(None),
            crate::data::DataType::BVec4 => self.bvec4.insert(None),
            crate::data::DataType::IVec2 => self.ivec2.insert(None),
            crate::data::DataType::IVec3 => self.ivec3.insert(None),
            crate::data::DataType::IVec4 => self.ivec4.insert(None),
            crate::data::DataType::UVec2 => self.uvec2.insert(None),
            crate::data::DataType::UVec3 => self.uvec3.insert(None),
            crate::data::DataType::UVec4 => self.uvec4.insert(None),
            crate::data::DataType::Vec2 => self.vec2.insert(None),
            crate::data::DataType::Vec3 => self.vec3.insert(None),
            crate::data::DataType::Vec4 => self.vec4.insert(None),
            crate::data::DataType::DVec2 => self.dvec2.insert(None),
            crate::data::DataType::DVec3 => self.dvec3.insert(None),
            crate::data::DataType::DVec4 => self.dvec4.insert(None),
            crate::data::DataType::Mat2 => self.mat2.insert(None),
            crate::data::DataType::Mat3 => self.mat3.insert(None),
            crate::data::DataType::Mat4 => self.mat4.insert(None),
            crate::data::DataType::DMat2 => self.dmat2.insert(None),
            crate::data::DataType::DMat3 => self.dmat3.insert(None),
            crate::data::DataType::DMat4 => self.dmat4.insert(None),
        }
    }

    pub fn name_var(&mut self, ty: DataType, id: usize, name: String) {
        match ty {
            DataType::Bool => *self.boolean.get_mut(id).unwrap() = Some(name),
            DataType::Int => *self.int.get_mut(id).unwrap() = Some(name),
            DataType::UInt => *self.uint.get_mut(id).unwrap() = Some(name),
            DataType::Float => *self.float.get_mut(id).unwrap() = Some(name),
            DataType::Double => *self.double.get_mut(id).unwrap() = Some(name),
            DataType::BVec2 => *self.bvec2.get_mut(id).unwrap() = Some(name),
            DataType::BVec3 => *self.bvec3.get_mut(id).unwrap() = Some(name),
            DataType::BVec4 => *self.bvec4.get_mut(id).unwrap() = Some(name),
            DataType::IVec2 => *self.ivec2.get_mut(id).unwrap() = Some(name),
            DataType::IVec3 => *self.ivec3.get_mut(id).unwrap() = Some(name),
            DataType::IVec4 => *self.ivec4.get_mut(id).unwrap() = Some(name),
            DataType::UVec2 => *self.uvec2.get_mut(id).unwrap() = Some(name),
            DataType::UVec3 => *self.uvec3.get_mut(id).unwrap() = Some(name),
            DataType::UVec4 => *self.uvec4.get_mut(id).unwrap() = Some(name),
            DataType::Vec2 => *self.vec2.get_mut(id).unwrap() = Some(name),
            DataType::Vec3 => *self.vec3.get_mut(id).unwrap() = Some(name),
            DataType::Vec4 => *self.vec4.get_mut(id).unwrap() = Some(name),
            DataType::DVec2 => *self.dvec2.get_mut(id).unwrap() = Some(name),
            DataType::DVec3 => *self.dvec3.get_mut(id).unwrap() = Some(name),
            DataType::DVec4 => *self.dvec4.get_mut(id).unwrap() = Some(name),
            DataType::Mat2 => *self.mat2.get_mut(id).unwrap() = Some(name),
            DataType::Mat3 => *self.mat3.get_mut(id).unwrap() = Some(name),
            DataType::Mat4 => *self.mat4.get_mut(id).unwrap() = Some(name),
            DataType::DMat2 => *self.dmat2.get_mut(id).unwrap() = Some(name),
            DataType::DMat3 => *self.dmat3.get_mut(id).unwrap() = Some(name),
            DataType::DMat4 => *self.dmat4.get_mut(id).unwrap() = Some(name),
        }
    }
}