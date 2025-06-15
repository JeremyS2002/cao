use syn::spanned::Spanned;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

fn convert_type_to_owned(ty: &syn::Type) -> syn::Type {
    // TODO maybe?? change this so it just strips out any life times to make them infered and 
    // then use <T as DescType>::InfoType as the field type definition?
    // let ty_type = match ty {
    //     syn::Type::Array(_) => "type_array",
    //     syn::Type::BareFn(_) => "type_bare_fn",
    //     syn::Type::Group(_) => "type_group",
    //     syn::Type::ImplTrait(_) => "type_impl_trait",
    //     syn::Type::Infer(_) => "type_infer",
    //     syn::Type::Macro(_) => "type_macro",
    //     syn::Type::Never(_) => "type_never",
    //     syn::Type::Paren(_) => "type_paren",
    //     syn::Type::Path(_) => "type_path",
    //     syn::Type::Ptr(_) => "type_ptr",
    //     syn::Type::Reference(_) => "type_reference",
    //     syn::Type::Slice(_) => "type_slice",
    //     syn::Type::TraitObject(_) => "type_trait_object",
    //     syn::Type::Tuple(_) => "type_tuple",
    //     syn::Type::Verbatim(_) => "token_stream",
    //     _ => todo!(),
    // };
    // println!("expand type {}", ty_type);
    match ty {
        syn::Type::Array(type_array) => syn::Type::Array(syn::TypeArray {
            bracket_token: type_array.bracket_token,
            elem: Box::new(convert_type_to_owned(&type_array.elem)),
            semi_token: type_array.semi_token,
            len: type_array.len.clone(),
        }),
        syn::Type::BareFn(_type_bare_fn) => todo!("type_bare_fn"),
        syn::Type::Group(_type_group) => todo!("type_group"),
        syn::Type::ImplTrait(_type_impl_trait) => todo!("type_impl_trait"),
        syn::Type::Infer(_type_infer) => todo!("type_infer"),
        syn::Type::Macro(_type_macro) => todo!("type_macro"),
        syn::Type::Never(_type_never) => todo!("type_never"),
        syn::Type::Paren(_type_paren) => todo!("type_paren"),
        syn::Type::Path(type_path) => {
            let mut new_path = syn::Path {
                leading_colon: type_path.path.leading_colon.clone(),
                segments: syn::punctuated::Punctuated::new(),
            };
            for segment in &type_path.path.segments {
                // println!("path segment {}", segment.ident.to_string());
                let new_segment = match &segment.arguments {
                    syn::PathArguments::None => syn::PathSegment {
                        ident: segment.ident.clone(),
                        arguments: syn::PathArguments::None,
                    },
                    syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                        let mut new_generic_arguments = syn::AngleBracketedGenericArguments {
                            colon2_token: angle_bracketed_generic_arguments.colon2_token,
                            lt_token: angle_bracketed_generic_arguments.lt_token,
                            args: syn::punctuated::Punctuated::new(),
                            gt_token: angle_bracketed_generic_arguments.gt_token,
                        };
                        for generic_arg in &angle_bracketed_generic_arguments.args {
                            let new_generic_arg = match generic_arg {
                                syn::GenericArgument::Lifetime(lifetime) => syn::GenericArgument::Lifetime(lifetime.clone()),
                                syn::GenericArgument::Type(generic_type) => syn::GenericArgument::Type(convert_type_to_owned(generic_type)),
                                syn::GenericArgument::Const(expr) => syn::GenericArgument::Const(expr.clone()),
                                syn::GenericArgument::Binding(binding) => syn::GenericArgument::Binding(binding.clone()),
                                syn::GenericArgument::Constraint(constraint) => syn::GenericArgument::Constraint(constraint.clone()),
                            };
                            new_generic_arguments.args.push(new_generic_arg);
                        }
                        syn::PathSegment {
                            ident: segment.ident.clone(),
                            arguments: syn::PathArguments::AngleBracketed(new_generic_arguments),
                        }
                    },
                    syn::PathArguments::Parenthesized(parenthesized_generic_arguments) => {
                        let new_generic_arguments = syn::ParenthesizedGenericArguments {
                            paren_token: parenthesized_generic_arguments.paren_token,
                            inputs: parenthesized_generic_arguments.inputs.iter().map(|this_ty| convert_type_to_owned(this_ty)).collect(),
                            output: match &parenthesized_generic_arguments.output {
                                syn::ReturnType::Default => syn::ReturnType::Default,
                                syn::ReturnType::Type(rarrow, generic_type) => syn::ReturnType::Type(rarrow.clone(), Box::new(convert_type_to_owned(&*generic_type))),
                            }
                        };

                        syn::PathSegment {
                            ident: segment.ident.clone(),
                            arguments: syn::PathArguments::Parenthesized(new_generic_arguments)
                        }
                    },
                };
                new_path.segments.push(new_segment);
            }
            syn::Type::Path(syn::TypePath {
                qself: type_path.qself.clone(),
                path: new_path
            })
        },
        syn::Type::Ptr(_type_ptr) => todo!("type_ptr"),
        syn::Type::Reference(type_reference) => {
            if let syn::Type::Slice(type_slice) = &*type_reference.elem {
                // &[T] => Vec<T>
                // if we have a reference to a slice T then we convert it to a Vec<T>

                let mut args = syn::punctuated::Punctuated::new();
                args.push(syn::GenericArgument::Type(convert_type_to_owned(&*type_slice.elem)));

                let mut segments = syn::punctuated::Punctuated::new();
                segments.push(syn::PathSegment {
                    ident: syn::Ident::new("Vec", type_slice.span()),
                    arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: syn::parse_str("<").unwrap(),
                        args,
                        gt_token: syn::parse_str(">").unwrap(),
                    })
                });

                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments,
                    }
                })
            } else if let syn::Type::Path(type_path) = &*type_reference.elem {
                // &str => String
                // if we have a reference to a str then we convert it to a String
                let mut cvt_string = false;
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident.to_string() == "str" {
                        cvt_string = true;
                    }
                }

                if cvt_string {
                    let mut segments = syn::punctuated::Punctuated::new();
                    segments.push(syn::PathSegment {
                        ident: syn::Ident::new("String", type_path.span()),
                        arguments: syn::PathArguments::None,
                    });

                    syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments,
                        }
                    })
                } else {
                    convert_type_to_owned(&*type_reference.elem)
                }
            } else {
                convert_type_to_owned(&type_reference.elem)
            }
        },
        syn::Type::Slice(_type_slice) => todo!("type_slice"),
        syn::Type::TraitObject(_type_trait_object) => todo!("type_trait_object"),
        syn::Type::Tuple(type_tuple) => syn::Type::Tuple(syn::TypeTuple { 
            paren_token: type_tuple.paren_token, 
            elems: type_tuple.elems.iter().map(|this_ty| convert_type_to_owned(this_ty)).collect()
        }),
        syn::Type::Verbatim(_token_stream) => todo!("token_stream"),
        _ => todo!(),
    }
}

#[proc_macro_derive(DescType, attributes(skip_info))]
pub fn derive_gpu_desc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let desc_name = input.ident;
    let desc_generics = input.generics;

    let desc_name_str = desc_name.to_string();
    let info_name_str = if desc_name_str.ends_with("Desc") {
        let mut n = desc_name_str.clone();
        n.truncate(desc_name_str.len() - 4);
        n.push_str("Info");
        n
    } else {
        let mut n = desc_name_str.clone();
        n.push_str("Info");
        n
    };
    // println!("info_name_str {}", info_name_str);
    let info_name = syn::Ident::new(&info_name_str, desc_name.span());

    let Data::Struct(struct_data) = input.data else {
        unimplemented!("derive(DescType) can only be used on Data::Struct");
    };

    let mut field_names = Vec::new();
    let mut desc_field_types = Vec::new();
    let mut info_field_types = Vec::new();

    for field in &struct_data.fields {
        let Some(ident) = &field.ident else {
            unimplemented!("derive(DescType) can only be used on structs with named fields");
        };

        let mut skip = false;
        for attr in &field.attrs {
            if attr.path.is_ident("skip_info") {
                skip = true;
                break;
            }
        }

        if skip { continue; }

        field_names.push(ident);
        info_field_types.push(convert_type_to_owned(&field.ty));
        desc_field_types.push(field.ty.clone());
    }

    let expanded = quote!(
        #[derive(Clone, Debug)]
        pub struct #info_name {
            #(
                pub #field_names : #info_field_types,
            )*
        }

        impl #desc_generics DescType for #desc_name #desc_generics {
            type InfoType = #info_name;

            fn to_info(&self) -> #info_name {
                #info_name {
                    #(
                        #field_names: self.#field_names.to_info(),
                    )*
                }
            }
        }
    );

    // let expanded = quote!();

    TokenStream::from(expanded)
}