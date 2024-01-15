use {
    crate::member,
    proc_macro2::TokenStream,
    syn::{
        meta::ParseNestedMeta, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields,
    },
};

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    use std::iter;

    let Data::Struct(DataStruct { fields, .. }) = input.data else {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the vertex type must be a struct");
        };
    };

    let named = match &fields {
        Fields::Named(_) => true,
        Fields::Unnamed(_) => false,
        Fields::Unit => {
            return quote::quote_spanned! { input.ident.span() =>
                ::std::compile_error!("the vertex type cannot be a unit struct");
            }
        }
    };

    if !input.generics.params.is_empty() {
        return quote::quote_spanned! { input.generics.params.span() =>
            ::std::compile_error!("the vertex struct cannot have generic parameters");
        };
    }

    if !input.attrs.iter().any(is_repr_c) {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the vertex struct must have the `#[repr(C)]` attribute");
        };
    }

    if fields.is_empty() {
        return quote::quote_spanned! { fields.span() =>
            ::std::compile_error!("the vertex struct must have some fields");
        };
    }

    let name = input.ident;
    let projection_name = quote::format_ident!("{name}Projection");
    let vector_types = fields.iter().map(|field| {
        let ty = &field.ty;
        quote::quote! { <#ty as ::dunge::vertex::InputProjection>::TYPE }
    });

    let projection_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        if named {
            quote::quote! { #ident: <#ty as ::dunge::vertex::InputProjection>::Field }
        } else {
            quote::quote! { <#ty as ::dunge::vertex::InputProjection>::Field }
        }
    });

    let projection_inputs = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::vertex::InputProjection>::input_projection(id, #index) }
    });

    let projection = if named {
        quote::quote! {
            struct #projection_name {
                #(#projection_fields),*,
            }
        }
    } else {
        quote::quote! {
            struct #projection_name(
                #(#projection_fields),*,
            );
        }
    };

    quote::quote! {
        unsafe impl ::dunge::Vertex for #name {
            type Projection = #projection_name;
            const DEF: ::dunge::sl::Define<::dunge::types::VectorType> = ::dunge::sl::Define::new(&[
                #(#vector_types),*,
            ]);
        }

        #projection

        impl ::dunge::vertex::Projection for #projection_name {
            fn projection(id: ::core::primitive::u32) -> Self {
                Self {
                    #(#projection_inputs),*,
                }
            }
        }
    }
}

fn is_repr_c(attr: &Attribute) -> bool {
    let parse_meta = |meta: ParseNestedMeta| {
        if meta.path.is_ident("C") {
            Ok(())
        } else {
            Err(meta.error("unrecognized repr"))
        }
    };

    attr.path().is_ident("repr") && attr.parse_nested_meta(parse_meta).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_vertex() {
        let input = quote::quote! {
            #[repr(C)]
            struct Vert {
                pos: [f32; 2],
                col: [f32; 3],
            }
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            unsafe impl ::dunge::Vertex for Vert {
                type Projection = VertProjection;
                const DEF: ::dunge::sl::Define<::dunge::types::VectorType> = ::dunge::sl::Define::new(&[
                    <[f32; 2] as ::dunge::vertex::InputProjection>::TYPE,
                    <[f32; 3] as ::dunge::vertex::InputProjection>::TYPE,
                ]);
            }

            struct VertProjection {
                pos: <[f32; 2] as ::dunge::vertex::InputProjection>::Field,
                col: <[f32; 3] as ::dunge::vertex::InputProjection>::Field,
            }

            impl ::dunge::vertex::Projection for VertProjection {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        pos: <[f32; 2] as ::dunge::vertex::InputProjection>::input_projection(id, 0u32),
                        col: <[f32; 3] as ::dunge::vertex::InputProjection>::input_projection(id, 1u32),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn derive_tuple_vertex() {
        let input = quote::quote! {
            #[repr(C)]
            struct Vert([f32; 2], [f32; 3]);
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            unsafe impl ::dunge::Vertex for Vert {
                type Projection = VertProjection;
                const DEF: ::dunge::sl::Define<::dunge::types::VectorType> = ::dunge::sl::Define::new(&[
                    <[f32; 2] as ::dunge::vertex::InputProjection>::TYPE,
                    <[f32; 3] as ::dunge::vertex::InputProjection>::TYPE,
                ]);
            }

            struct VertProjection(
                <[f32; 2] as ::dunge::vertex::InputProjection>::Field,
                <[f32; 3] as ::dunge::vertex::InputProjection>::Field,
            );

            impl ::dunge::vertex::Projection for VertProjection {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        0: <[f32; 2] as ::dunge::vertex::InputProjection>::input_projection(id, 0u32),
                        1: <[f32; 3] as ::dunge::vertex::InputProjection>::input_projection(id, 1u32),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
