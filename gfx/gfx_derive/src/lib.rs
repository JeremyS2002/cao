use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data};

const ERROR: &'static str = "Structs to derive vertex must have named fields of types, f32, [f32; 2], [f32; 3], [f32; 4], glam::Vec2, glam::Vec3, glam::Vec4";

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Format {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

struct IdentListParser {
    v: Vec<syn::Ident>,
}

impl syn::parse::Parse for IdentListParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut v = Vec::<syn::Ident>::new();

        loop {
            if input.is_empty() {
                break;
            }

            v.push(input.parse::<syn::Ident>()?);

            if input.is_empty() {
                break;
            }

            input.parse::<syn::Token!(,)>()?;
        }

        Ok(IdentListParser { v })
    }
}

fn syn_type_to_gpu_vertex_format(ty: &syn::Type) -> Option<Format> {
    match ty {
        syn::Type::Path(type_path) => {
            let type_path_string = type_path.clone().into_token_stream().to_string();
            // println!("{}", type_path_string);

            match &*type_path_string {
                ":: std :: f32" => Some(Format::Float),
                "std :: f32" => Some(Format::Float),
                "f32" => Some(Format::Float),
                "glam :: Vec2" => Some(Format::Vec2),
                "glam :: Vec3" => Some(Format::Vec3),
                "glam :: Vec4" => Some(Format::Vec4),
                "Vec2" => Some(Format::Vec2),
                "Vec3" => Some(Format::Vec3),
                "Vec4" => Some(Format::Vec4),
                "GlamVec2" => Some(Format::Vec2),
                "GlamVec3" => Some(Format::Vec3),
                "GlamVec4" => Some(Format::Vec4),
                _ => None,
            }
        }
        syn::Type::Array(type_array) => {
            let syn::Type::Path(type_path) = &*type_array.elem else {
                return None
            };
            let type_path_string = type_path.clone().into_token_stream().to_string();


            if type_path_string == "f32" || type_path_string == ":: std :: f32" || type_path_string == "std :: f32" {
                match &type_array.len {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Int(lit_int) => {
                            let lit_int_str = lit_int.base10_digits();
                            match lit_int_str {
                                "2" => Some(Format::Vec2),
                                "3" => Some(Format::Vec3),
                                "4" => Some(Format::Vec4),
                                _ => unimplemented!("{}", ERROR),
                            }
                        },
                        _ => unimplemented!("{}", ERROR),
                    },
                    _ => unimplemented!("{}", ERROR),
                }
            } else {
                unimplemented!("{}", ERROR)
            }
        },
        _ => unimplemented!("{}", ERROR),
    }
}

#[proc_macro_derive(Vertex, attributes(alias))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    // names that map to locations and types
    let mut names = Vec::new();
    // the current offset in bytes
    let mut offset = 0u32;
    // the offsets that each name maps to, in lockstep with names vec
    let mut offsets = Vec::new();
    // the format that each name maps to, in lockstep with the names and offsets vec
    let mut formats = Vec::new();

    // mush be a data struct
    let Data::Struct(struct_data) = input.data else {
        unimplemented!("{}", ERROR);
    };

    // for each field
    for field in &struct_data.fields {
        // must have a name
        let Some(ident) = &field.ident else {
            unimplemented!("{}", ERROR)
        };
        let Some(format) = syn_type_to_gpu_vertex_format(&field.ty) else {
            unimplemented!("{}", ERROR)
        };

        // utility for pushing to each vec
        let mut push = |n, o, f| {
            names.push(n);
            offsets.push(o);
            match f {
                Format::Float => formats.push(quote!(gpu::VertexFormat::Float)),
                Format::Vec2 => formats.push(quote!(gpu::VertexFormat::Vec2)),
                Format::Vec3 => formats.push(quote!(gpu::VertexFormat::Vec3)),
                Format::Vec4 => formats.push(quote!(gpu::VertexFormat::Vec4)),
            }
        };

        push(ident.to_string(), offset, format);
        
        for attr in &field.attrs {
            if attr.path.is_ident("alias") {
                let parsed: IdentListParser = attr.parse_args().unwrap();
                for alias in parsed.v {
                    push(alias.to_string(), offset, format);
                }
            }
        };

        // advance the offset based on the format
        match format {
            Format::Float => offset += 4,
            Format::Vec2 => offset += 4 * 2,
            Format::Vec3 => offset += 4 * 3,
            Format::Vec4 => offset += 4 * 4,
        }
    }

    let expanded = quote!(
        impl gfx::Vertex for #name {
            fn get(n: &str) -> Option<(u32, gpu::VertexFormat)> {
                match n {
                    #(
                        #names => Some((#offsets, #formats)),
                    )*
                    _ => None,
                }
            }
        }
    );

    TokenStream::from(expanded)
}
