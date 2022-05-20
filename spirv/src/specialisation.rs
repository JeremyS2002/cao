
use either::*;
use crate::data::*;

pub trait ShaderTY { 
    const TY: rspirv::spirv::ExecutionModel;
}

pub struct Vertex { }

impl ShaderTY for Vertex {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::Vertex;
}

pub struct Fragment { }

impl ShaderTY for Fragment {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::Fragment;
}

pub struct Geometry { }

impl ShaderTY for Geometry {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::Geometry;
}

pub struct TessControl { }

impl ShaderTY for TessControl {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::TessellationControl;
}

pub struct TessEval { }

impl ShaderTY for TessEval {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::TessellationEvaluation;
}

pub struct Compute { }

impl ShaderTY for Compute {
    const TY: rspirv::spirv::ExecutionModel = rspirv::spirv::ExecutionModel::GLCompute;
}

pub type VertexBuilder = super::Builder<Vertex>;
pub type FragmentBuilder = super::Builder<Fragment>;
pub type GeometryBuilder = super::Builder<Geometry>;
pub type TessControlBuilder = super::Builder<TessControl>;
pub type TessEvalBuilder = super::Builder<TessEval>;
pub type ComputeBuilder = super::Builder<Compute>;

macro_rules! impl_specialisation {
    ($($name:ident: 
        [ 
            $($spec_in:ident, $ty_in:ident, $built_in_a:ident,)* 
        ], [ 
            $($spec_out:ident, $ty_out:ident, $built_in_b:ident,)* 
        ],
    )*) => {
        $(
            impl $name {
                $(
                    pub fn $spec_in(&self) -> crate::interface::In<$ty_in> {
                        let index = self.raw.outputs.borrow().len();
                        self.raw.outputs.borrow_mut().push((
                            crate::data::PrimitiveType::$ty_in, 
                            Right(rspirv::spirv::BuiltIn::$built_in_a), 
                            Some(stringify!($spec_in)), 
                        ));
                        crate::interface::In {
                            index,
                            _marker: std::marker::PhantomData
                        }
                    }
                )*

                $(
                    pub fn $spec_out(&self) -> crate::interface::Out<$ty_out> {
                        let index = self.raw.outputs.borrow().len();
                        self.raw.outputs.borrow_mut().push((
                            crate::data::PrimitiveType::$ty_out, 
                            Right(rspirv::spirv::BuiltIn::$built_in_b), 
                            Some(stringify!($spec_out)), 
                        ));
                        crate::interface::Out {
                            index,
                            _marker: std::marker::PhantomData
                        }
                    }
                )*
            }
        )*
    };
}

impl_specialisation!(
    VertexBuilder : [
        vertex_id, Int, VertexId,
        instance_id, Int, InstanceId,
        draw_index, Int, DrawIndex,
        base_vertex, Int, BaseVertex,

    ], [
        position, Vec4, Position,
        point_size, Float, PointSize,
    ],
    TessControlBuilder : [
        patch_vertices, Int, PatchVertices,
        primitive_id, Int, PrimitiveId,
        invocation_id, Int, InvocationId,
    ], [

    ],
    TessEvalBuilder : [
        tess_coord, Vec3, TessCoord,
        primitive_id, Int, PrimitiveId,
        invocation_id, Int, InvocationId,
    ], [

    ],
    GeometryBuilder : [
        primitive_id, Int, PrimitiveId,
        invocation_id, Int, InvocationId,
    ], [

    ],
    FragmentBuilder : [
        frag_coord, Vec4, FragCoord,
        front_facing, Bool, FrontFacing,
        point_coord, Vec2, PointCoord,
        layer, Int, Layer,
    ], [
        frag_depth, Float, FragDepth,
    ],
    ComputeBuilder : [
        num_work_gropus, UVec3, NumWorkgroups,
        work_group_id, UVec3, WorkgroupId,
        local_invocation_id, UVec3, LocalInvocationId,
        global_invocation_id, UVec3, GlobalInvocationId,
        local_invocation_index, UInt, LocalInvocationIndex,
    ], [

    ],
);
