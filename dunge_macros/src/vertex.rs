use {
    proc_macro2::TokenStream,
    syn::{meta::ParseNestedMeta, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput},
};

pub(crate) fn impl_vertex(input: DeriveInput) -> TokenStream {
    use std::{borrow::Cow, iter};

    let Data::Struct(DataStruct { fields, .. }) = input.data else {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the vertex type must be a struct");
        };
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

    let make_ident = |index: u32, ident| match ident {
        Some(ident) => Cow::Borrowed(ident),
        None => Cow::Owned(quote::format_ident!("f{index}")),
    };

    let name = input.ident;
    let projection_name = quote::format_ident!("{name}Projection");
    let vector_types = fields.iter().map(|field| {
        let ty = &field.ty;
        quote::quote! { <#ty as ::dunge::vertex::InputProjection>::TYPE }
    });

    let projection_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = make_ident(index, field.ident.as_ref());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::vertex::InputProjection>::Field }
    });

    let projection_inputs = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = make_ident(index, field.ident.as_ref());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::vertex::InputProjection>::input_projection(id, #index) }
    });

    quote::quote! {
        unsafe impl ::dunge::vertex::Vertex for #name {
            type Projection = #projection_name;
            const DECL: ::dunge::vertex::DeclareInput = ::dunge::vertex::DeclareInput::new(&[
                #(#vector_types),*,
            ]);
        }

        struct #projection_name {
            #(#projection_fields),*,
        }

        impl ::dunge::vertex::Projection for #projection_name {
            fn projection(id: u32) -> Self {
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
        let actual = impl_vertex(input);
        let expectd = quote::quote! {
            unsafe impl ::dunge::vertex::Vertex for Vert {
                type Projection = VertProjection;
                const DECL: ::dunge::vertex::DeclareInput = ::dunge::vertex::DeclareInput::new(&[
                    <[f32; 2] as ::dunge::vertex::InputProjection>::TYPE,
                    <[f32; 3] as ::dunge::vertex::InputProjection>::TYPE,
                ]);
            }

            struct VertProjection {
                pos: <[f32; 2] as ::dunge::vertex::InputProjection>::Field,
                col: <[f32; 3] as ::dunge::vertex::InputProjection>::Field,
            }

            impl ::dunge::vertex::Projection for VertProjection {
                fn projection(id: u32) -> Self {
                    Self {
                        pos: <[f32; 2] as ::dunge::vertex::InputProjection>::input_projection(id, 0u32),
                        col: <[f32; 3] as ::dunge::vertex::InputProjection>::input_projection(id, 1u32),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expectd.to_string());
    }
}
