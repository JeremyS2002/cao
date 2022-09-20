
// TODO rewrite all of this shit
// this is the first proc macro i've written and it shows

use proc_macro2::{Ident, Span};

use proc_macro::TokenStream;
use syn::DeriveInput;

use quote::ToTokens;

struct ParsedType {
    size: u32,
    dynamic_ty: proc_macro2::TokenStream,
    static_ty: proc_macro2::TokenStream,
}

impl ParsedType {
    fn new(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Array(a) => {
                let elem = Self::new(&*a.elem);
                
                let elem_size = elem.size;
                let elem_dynamic = elem.dynamic_ty;
                let elem_static = elem.static_ty;

                let len = if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), ..}) = &a.len {
                    i.base10_parse::<u32>().unwrap()
                } else {
                    panic!();
                };

                let size = len * elem_size;
                let dynamic_ty = quote::quote!(::spv::Type::Array(::spv::ArrayType { element_ty: #elem_dynamic, length: Some(#len) }));
                let static_ty = quote::quote!(::spv::ArrayType<'a, #elem_static, #len>);

                Self {
                    size,
                    dynamic_ty,
                    static_ty,
                }
            },
            syn::Type::Verbatim(ty) => {
                let dynamic_ty = Self::rust_to_dynamic_spv(ty);
                let static_ty = Self::rust_to_static_spv(ty);
                let size = Self::rust_to_size(ty);
                Self {
                    dynamic_ty,
                    static_ty,
                    size,
                }
            },
            syn::Type::Path(syn::TypePath { qself: None, path }) => {
                let s = path.to_token_stream();
                let dynamic_ty = Self::rust_to_dynamic_spv(&s);
                let static_ty = Self::rust_to_static_spv(&s);
                let size = Self::rust_to_size(&s);
                Self {
                    dynamic_ty,
                    static_ty,
                    size,
                }
            },
            _ => panic!(""),
        }
    }

    fn rust_to_dynamic_spv(ty: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match &*ty.to_string() {
            "()" => quote::quote!(::spv::Type::Void),
            "bool" => quote::quote!(::spv::Type::Scalar(::spv::ScalarType::Bool)),
            "i32" => quote::quote!(::spv::Type::Scalar(::spv::ScalarType::Signed(32))),
            "u32" => quote::quote!(::spv::Type::Scalar(::spv::ScalarType::Unsigned(32))),
            "f32" => quote::quote!(::spv::Type::Scalar(::spv::ScalarType::Float(32))),
            "f64" => quote::quote!(::spv::Type::Scalar(::spv::ScalatType::Float(64))),
            "glam :: IVec2" | ":: glam :: IVec2" | "GlamIVec2" | "IVec2" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Signed(32), n_scalar: 2 })),
            "glam :: IVec3" | ":: glam :: IVec3" | "GlamIVec3" | "IVec3" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Signed(32), n_scalar: 3 })),
            "glam :: IVec4" | ":: glam :: IVec4" | "GlamIVec4" | "IVec4" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Signed(32), n_scalar: 4 })),
            "glam :: UVec2" | ":: glam :: UVec2" | "GlamUVec2" | "UVec2" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Unsigned(32), n_scalar: 2 })),
            "glam :: UVec3" | ":: glam :: UVec3" | "GlamUVec3" | "UVec3" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Unsigned(32), n_scalar: 3 })),
            "glam :: UVec4" | ":: glam :: UVec4" | "GlamUVec4" | "UVec4" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Unsigned(32), n_scalar: 4 })),
            "glam :: Vec2" | ":: glam :: Vec2" | "GlamVec2" | "Vec2" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 2 })),
            "glam :: Vec3" | ":: glam :: Vec3" | "GlamVec3" | "Vec3" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 3 })),
            "glam :: Vec4" | ":: glam :: Vec4" | "GlamVec4" | "Vec4" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 4 })),
            "glam :: DVec2" | ":: glam :: DVec2" | "GlamDVec2" | "DVec2" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 2 })),
            "glam :: DVec3" | ":: glam :: DVec3" | "GlamDVec3" | "DVec3" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 3 })),
            "glam :: DVec4" | ":: glam :: DVec4" | "GlamDVec4" | "DVec4" => quote::quote!(::spv::Type::Vector(::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 4 })),
            "glam :: Mat2" | ":: glam :: Mat2" | "GlamMat2" | "Mat2" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 2 }, n_vec: 2 })),
            "glam :: Mat3" | ":: glam :: Mat3" | "GlamMat3" | "Mat3" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 3 }, n_vec: 3 })),
            "glam :: Mat4" | ":: glam :: Mat4" | "GlamMat4" | "Mat4" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(32), n_scalar: 4 }, n_vec: 4 })),
            "glam :: DMat2" | ":: glam :: DMat2" | "GlamDMat2" | "DMat2" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 2 }, n_vec: 2 })),
            "glam :: DMat3" | ":: glam :: DMat3" | "GlamDMat3" | "DMat3" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 3 }, n_vec: 3 })),
            "glam :: DMat4" | ":: glam :: DMat4" | "GlamDMat4" | "DMat4" => quote::quote!(::spv::Type::Matrix(::spv::MatrixType { vec_ty: ::spv::VectorType { scalar_ty: ::spv::ScalarType::Float(64), n_scalar: 4 }, n_vec: 4 })),
            s => panic!("Unsupported field type: {}", s),
        }
    }

    fn rust_to_static_spv(ty: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match &*ty.to_string() {
            "()" => quote::quote!(::spv::Void),
            "bool" => quote::quote!(::spv::Bool),
            "i32" => quote::quote!(::spv::Int),
            "u32" => quote::quote!(::spv::UInt),
            "f32" => quote::quote!(::spv::Float),
            "f64" => quote::quote!(::spv::Type::Double),
            "glam :: IVec2" | ":: glam :: IVec2" | "GlamIVec2" | "IVec2" => quote::quote!(::spv::IVec2),
            "glam :: IVec3" | ":: glam :: IVec3" | "GlamIVec3" | "IVec3" => quote::quote!(::spv::IVec3),
            "glam :: IVec4" | ":: glam :: IVec4" | "GlamIVec4" | "IVec4" => quote::quote!(::spv::IVec4),
            "glam :: UVec2" | ":: glam :: UVec2" | "GlamUVec2" | "UVec2" => quote::quote!(::spv::UVec2),
            "glam :: UVec3" | ":: glam :: UVec3" | "GlamUVec3" | "UVec3" => quote::quote!(::spv::UVec3),
            "glam :: UVec4" | ":: glam :: UVec4" | "GlamUVec4" | "UVec4" => quote::quote!(::spv::UVec4),
            "glam :: Vec2" | ":: glam :: Vec2" | "GlamVec2" | "Vec2" => quote::quote!(::spv::Vec2),
            "glam :: Vec3" | ":: glam :: Vec3" | "GlamVec3" | "Vec3" => quote::quote!(::spv::Vec3),
            "glam :: Vec4" | ":: glam :: Vec4" | "GlamVec4" | "Vec4" => quote::quote!(::spv::Vec4),
            "glam :: DVec2" | ":: glam :: DVec2" | "GlamDVec2" | "DVec2" => quote::quote!(::spv::DVec2),
            "glam :: DVec3" | ":: glam :: DVec3" | "GlamDVec3" | "DVec3" => quote::quote!(::spv::DVec3),
            "glam :: DVec4" | ":: glam :: DVec4" | "GlamDVec4" | "DVec4" => quote::quote!(::spv::DVec4),
            "glam :: Mat2" | ":: glam :: Mat2" | "GlamMat2" | "Mat2" => quote::quote!(::spv::Mat2),
            "glam :: Mat3" | ":: glam :: Mat3" | "GlamMat3" | "Mat3" => quote::quote!(::spv::Mat3),
            "glam :: Mat4" | ":: glam :: Mat4" | "GlamMat4" | "Mat4" => quote::quote!(::spv::Mat4),
            "glam :: DMat2" | ":: glam :: DMat2" | "GlamDMat2" | "DMat2" => quote::quote!(::spv::DMat2),
            "glam :: DMat3" | ":: glam :: DMat3" | "GlamDMat3" | "DMat3" => quote::quote!(::spv::DMat3),
            "glam :: DMat4" | ":: glam :: DMat4" | "GlamDMat4" | "DMat4" => quote::quote!(::spv::DMat4),
            s => panic!("Unsupported field type: {}", s),
        }
    }

    fn rust_to_size(ty: &proc_macro2::TokenStream) -> u32 {
        match &*ty.to_string() {
            "()" => 0,
            "bool" => 1,
            "i32" => 4,
            "u32" => 4,
            "f32" => 4,
            "f64" => 8,
            "glam :: IVec2" | ":: glam :: IVec2" | "GlamIVec2" | "IVec2" => 2 * 4,
            "glam :: IVec3" | ":: glam :: IVec3" | "GlamIVec3" | "IVec3" => 3 * 4,
            "glam :: IVec4" | ":: glam :: IVec4" | "GlamIVec4" | "IVec4" => 4 * 4,
            "glam :: UVec2" | ":: glam :: UVec2" | "GlamUVec2" | "UVec2" => 2 * 4,
            "glam :: UVec3" | ":: glam :: UVec3" | "GlamUVec3" | "UVec3" => 3 * 4,
            "glam :: UVec4" | ":: glam :: UVec4" | "GlamUVec4" | "UVec4" => 4 * 4,
            "glam :: Vec2" | ":: glam :: Vec2" | "GlamVec2" | "Vec2" => 2 * 4,
            "glam :: Vec3" | ":: glam :: Vec3" | "GlamVec3" | "Vec3" => 3 * 4,
            "glam :: Vec4" | ":: glam :: Vec4" | "GlamVec4" | "Vec4" => 4 * 4,
            "glam :: DVec2" | ":: glam :: DVec2" | "GlamDVec2" | "DVec2" => 2 * 8,
            "glam :: DVec3" | ":: glam :: DVec3" | "GlamDVec3" | "DVec3" => 3 * 8,
            "glam :: DVec4" | ":: glam :: DVec4" | "GlamDVec4" | "DVec4" => 4 * 8,
            "glam :: Mat2" | ":: glam :: Mat2" | "GlamMat2" | "Mat2" => 2 * 2 * 4,
            "glam :: Mat3" | ":: glam :: Mat3" | "GlamMat3" | "Mat3" => 3 * 3 * 4,
            "glam :: Mat4" | ":: glam :: Mat4" | "GlamMat4" | "Mat4" => 4 * 4 * 4,
            "glam :: DMat2" | ":: glam :: DMat2" | "GlamDMat2" | "DMat2" => 2 * 2 * 8,
            "glam :: DMat3" | ":: glam :: DMat3" | "GlamDMat3" | "DMat3" => 3 * 3 * 8,
            "glam :: DMat4" | ":: glam :: DMat4" | "GlamDMat4" | "DMat4" => 4 * 4 * 8,
            s => panic!("Unsupported field type: {}", s),
        }
    }
}

#[proc_macro_derive(AsStructType)]
pub fn spv_struct(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let vis = &ast.vis;

    let spv_name_str = format!("Spv{}", name.to_string());
    let spv_name = Ident::new(&spv_name_str, Span::call_site());

    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) = &ast.data {
        &fields.named
    } else {
        panic!("Cannot derive SpvStruct on type not struct");
    };

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());
    let field_names2 = field_names.clone();
    let field_names3 = field_names.clone();
    let field_names4 = field_names.clone();
    let field_str_names = field_names.clone().map(|n| n.to_string());
    let field_types = fields.iter().map(|f| &f.ty);
    let mut field_static_spv_types = Vec::new();
    let mut field_dynamic_spv_types = Vec::new();
    let mut field_offsets = Vec::new();
    let mut offset = 0;
    for ty in field_types {
        let parsed = ParsedType::new(ty);
        field_static_spv_types.push(parsed.static_ty);
        field_dynamic_spv_types.push(parsed.dynamic_ty);
        field_offsets.push(offset);
        offset += parsed.size;
    }
    let field_static_spv_types2 = field_static_spv_types.clone();
    let field_dynamic_spv_types2 = field_dynamic_spv_types.clone();
    let field_indexes = 0u32..;

    let name_str = name.to_string();

    let gen = quote::quote! {
        impl #name {
            const MEMBERS: &'static [::spv::StructMember] = &[#(
                ::spv::StructMember {
                    name: Some(::spv::either::Left(#field_str_names)),
                    offset: #field_offsets,
                    ty: #field_dynamic_spv_types,
                },
            )*];
        }

        impl ::spv::RustStructType for #name {
            type Spv<'a> = #spv_name<'a>;

            fn fields<'a>(&'a self) -> Vec<&'a dyn ::spv::AsType> {
                let mut v = Vec::<&dyn ::spv::AsType>::new();

                #(
                    v.push((&self.#field_names) as _);
                )*

                v
            }
        }

        impl ::spv::AsStructTypeConst for #name {
            const STRUCT_TY: ::spv::StructType = ::spv::StructType {
                name: Some(::spv::either::Left(#name_str)),
                members: ::std::borrow::Cow::Borrowed(Self::MEMBERS),
            };
        }

        impl ::spv::AsStructType for #name {
            fn struct_ty(&self) -> ::spv::StructType {
                <Self as ::spv::AsStructTypeConst>::STRUCT_TY
            }

            fn struct_id(&self, s: &mut dyn ::spv::Scope) -> usize {
                use ::spv::RustStructType;
                let constituents = self.fields()
                    .into_iter()
                    .map(|c| {
                        (c.id(s), c.ty())
                    })
                    .collect::<Vec<_>>();
                let new_id = s.get_new_id();
                s.push_instruction(::spv::Instruction::Composite(::spv::OpComposite {
                    ty: ::spv::Type::Struct(<Self as ::spv::AsStructTypeConst>::STRUCT_TY),
                    id: new_id,
                    constituents,
                }));
                new_id
            }

            fn as_struct_ty_ref<'a>(&'a self) -> &'a dyn ::spv::AsStructType {
                self
            }
        }

        impl ::spv::AsTypeConst for #name {
            const TY: ::spv::Type = ::spv::Type::Struct(<Self as ::spv::AsStructTypeConst>::STRUCT_TY);
        }

        impl ::spv::AsType for #name {
            fn ty(&self) -> ::spv::Type {
                <Self as ::spv::AsTypeConst>::TY
            }
        
            fn id(&self, s: &mut dyn ::spv::Scope) -> usize {
                use ::spv::AsStructType;
                self.struct_id(s)
            }
        
            fn as_ty_ref<'b>(&'b self) -> &'b dyn ::spv::AsType {
                self
            }
        }

        #vis struct #spv_name<'a> {
            id: usize,
            b: &'a ::std::sync::Arc<::std::sync::Mutex<::spv::BuilderInner>>,
        }

        impl<'a, 'b> ::spv::SpvRustEq<#spv_name<'b>> for #spv_name<'a> {
            fn as_ty<'c>(&'c self) -> &'c dyn ::spv::AsType {
                self
            }
        }

        impl<'a> ::spv::SpvRustEq<#spv_name<'a>> for #name {
            fn as_ty<'b>(&'b self) -> &'b dyn ::spv::AsType {
                self
            }
        }

        impl<'a> #spv_name<'a> {
            pub fn new(
                __b: &'a ::spv::Builder,
                #(
                    #field_names2: &dyn ::spv::SpvRustEq<#field_static_spv_types>,
                )*
            ) -> Self {
                let mut inner = __b.__inner().lock().unwrap();
                if let Some(scope) = inner.__scope() {
                    use ::spv::SpvRustEq;
                    use ::spv::Scope;

                    let mut constituents = Vec::new();
                    #(
                        constituents.push(#field_names3.as_ty());
                    )*
                    let constituents = constituents.iter()
                        .map(|c| {
                            (c.id(scope), c.ty())
                        })
                        .collect::<Vec<_>>();
                    let new_id = scope.get_new_id();
                    scope.push_instruction(::spv::Instruction::Composite(::spv::OpComposite {
                        ty: ::spv::Type::Struct(<Self as ::spv::AsStructTypeConst>::STRUCT_TY),
                        id: new_id,
                        constituents,
                    }));
                    drop(scope);
                    drop(inner);
                    #spv_name {
                        id: new_id,
                        b: &__b.__inner(),
                    }
                } else {
                    panic!("Cannot create new struct when not in function")
                }
            }

            #(
                pub fn #field_names4(&self) -> #field_static_spv_types2<'a> {
                    let mut inner = self.b.lock().unwrap();
                    if let Some(scope) = inner.__scope() {
                        use ::spv::FromId;
                        let new_id = scope.get_new_id();
                        scope.push_instruction(::spv::Instruction::LoadStore(::spv::OpLoadStore {
                            ty: #field_dynamic_spv_types2,
                            src: ::spv::OpLoadStoreData::Struct {
                                id: self.id,
                                struct_ty: <Self as ::spv::AsStructTypeConst>::STRUCT_TY,
                                field: #field_indexes,
                            },
                            dst: ::spv::OpLoadStoreData::Variable { id: new_id },
                        }));

                        drop(scope);
                        drop(inner);

                        #field_static_spv_types2::from_id(new_id, self.b)
                    } else {
                        panic!("Cannot get field from struct when builder not in function");
                    }
                }
            )*
        }

        impl<'a> ::spv::FromId<'a> for #spv_name<'a> {
            fn from_id(id: usize, b: &'a ::std::sync::Arc<::std::sync::Mutex<::spv::BuilderInner>>) -> Self {
                Self {
                    id,
                    b
                }
            }
        }

        impl<'a> ::spv::AsStructTypeConst for #spv_name<'a> {
            const STRUCT_TY: ::spv::StructType = <#name as ::spv::AsStructTypeConst>::STRUCT_TY;
        }

        impl<'a> ::spv::IsStructTypeConst for #spv_name<'a> { }

        impl<'a> ::spv::AsStructType for #spv_name<'a> {
            fn struct_ty(&self) -> ::spv::StructType {
                <Self as ::spv::AsStructTypeConst>::STRUCT_TY
            }

            fn struct_id(&self, s: &mut dyn ::spv::Scope) -> usize {
                self.id
            }

            fn as_struct_ty_ref<'b>(&'b self) -> &'b dyn ::spv::AsStructType {
                self
            }
        }

        impl<'a> ::spv::IsStructType for #spv_name<'a> { }

        impl<'a> ::spv::AsTypeConst for #spv_name<'a> {
            const TY: ::spv::Type = ::spv::Type::Struct(<Self as ::spv::AsStructTypeConst>::STRUCT_TY);
        }

        impl<'a> ::spv::IsTypeConst for #spv_name<'a> { 
            type T<'b> = #spv_name<'b>;
        }

        impl<'a> ::spv::AsType for #spv_name<'a> {
            fn ty(&self) -> ::spv::Type {
                <Self as ::spv::AsTypeConst>::TY
            }
        
            fn id(&self, s: &mut dyn ::spv::Scope) -> usize {
                use ::spv::AsStructType;
                self.struct_id(s)
            }
        
            fn as_ty_ref<'b>(&'b self) -> &'b dyn ::spv::AsType {
                self
            }
        }

        impl<'a> ::spv::IsType for #spv_name<'a> { }
    };

    TokenStream::from(gen)
}
