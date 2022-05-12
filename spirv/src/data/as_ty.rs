
macro_rules! gen_as_ty {
    ($($name:ident,)*) => {
        $(
            pub trait $name {

            }
        )*
    };
}

gen_as_ty!(
    AsBool,
    AsInt,
    AsUInt,
    AsFloat,
    AsDouble,
    AsBVec2,
    AsBVec3,
    AsBVec4,
    AsIVec2,
    AsIVec3,
    AsIVec4,
    AsUVec2,
    AsUVec3,
    AsUVec4,
    AsVec2,
    AsVec3,
    AsVec4,
    AsDVec2,
    AsDVec3,
    AsDVec4,
    AsBMat2,
    AsBMat3,
    AsBMat4,
    AsIMat2,
    AsIMat3,
    AsIMat4,
    AsUMat2,
    AsUMat3,
    AsUMat4,
    AsMat2,
    AsMat3,
    AsMat4,
    AsDMat2,
    AsDMat3,
    AsDMat4,
);