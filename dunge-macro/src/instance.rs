use {
    crate::member,
    proc_macro2::{Span, TokenStream},
    syn::{Data, DataStruct, DeriveInput, Fields, GenericParam, Ident, Lifetime, spanned::Spanned},
};

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    use std::iter;

    let Data::Struct(DataStruct { fields, .. }) = input.data else {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the instance type must be a struct");
        };
    };

    let named = match &fields {
        Fields::Named(_) => true,
        Fields::Unnamed(_) => false,
        Fields::Unit => {
            return quote::quote_spanned! { input.ident.span() =>
                ::std::compile_error!("the instance type cannot be a unit struct");
            };
        }
    };

    let mut lts = Vec::with_capacity(input.generics.params.len());
    for param in input.generics.params {
        let GenericParam::Lifetime(param) = param else {
            return quote::quote_spanned! { param.span() =>
                ::std::compile_error!("the instance struct cannot have non-lifetime generic parameters");
            };
        };

        if !param.attrs.is_empty() {
            return quote::quote_spanned! { param.span() =>
                ::std::compile_error!("the lifetime cannot have any attributes");
            };
        }

        if !param.bounds.is_empty() {
            return quote::quote_spanned! { param.span() =>
                ::std::compile_error!("the lifetime cannot have any bounds");
            };
        }

        lts.push(param.lifetime);
    }

    if fields.is_empty() {
        return quote::quote_spanned! { fields.span() =>
            ::std::compile_error!("the instance struct must have some fields");
        };
    }

    let static_lt = Lifetime {
        apostrophe: Span::call_site(),
        ident: Ident::new("static", Span::call_site()),
    };

    let static_lts = lts.iter().map(|_| &static_lt);
    let anon_lt = Lifetime {
        apostrophe: Span::call_site(),
        ident: Ident::new("_", Span::call_site()),
    };

    let anon_lts = lts
        .iter()
        .map(|lt| if lt.ident == "static" { lt } else { &anon_lt });

    let name = input.ident;
    let projection_name = quote::format_ident!("{name}Proj");
    let instance_types = fields.iter().map(|field| {
        let ty = &field.ty;
        quote::quote! { <#ty as dunge::instance::MemberProjection>::TYPE }
    });

    let instance_set_members = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        quote::quote! { dunge::instance::SetMember::set_member(&self.#ident, setter) }
    });

    let instance_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        if named {
            quote::quote! { #ident: <#ty as dunge::instance::MemberProjection>::Field }
        } else {
            quote::quote! { <#ty as dunge::instance::MemberProjection>::Field }
        }
    });

    let instance_member_projections = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as dunge::instance::MemberProjection>::member_projection(id + #index) }
    });

    let projection = if named {
        quote::quote! {
            pub struct #projection_name<#(#lts),*> {
                #(#instance_fields),*,
            }
        }
    } else {
        quote::quote! {
            pub struct #projection_name<#(#lts),*>(
                #(#instance_fields),*,
            );
        }
    };

    quote::quote! {
        impl<#(#lts),*> dunge::Instance for #name<#(#lts),*> {
            type Projection = #projection_name<#(#static_lts),*>;
            const DEF: dunge::sl::Define<dunge::types::ValueType> = dunge::sl::Define::new(&[
                #(#instance_types),*,
            ]);
        }

        impl dunge::instance::Set for #name<#(#anon_lts),*> {
            fn set(&self, setter: &mut dunge::instance::Setter<'_, '_>) {
                #(#instance_set_members);*;
            }
        }

        #projection

        impl<#(#lts),*> dunge::instance::Projection for #projection_name<#(#lts),*> {
            fn projection(id: ::core::primitive::u32) -> Self {
                Self {
                    #(#instance_member_projections),*,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_instance() {
        let input = quote::quote! {
            struct Transform<'slice> {
                pos: Row<[f32; 2]>,
                col: RowSlice<'slice, [f32; 3]>,
            }
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'slice> dunge::Instance for Transform<'slice> {
                type Projection = TransformProj<'static>;
                const DEF: dunge::sl::Define<dunge::types::ValueType> = dunge::sl::Define::new(&[
                    <Row<[f32; 2]> as dunge::instance::MemberProjection>::TYPE,
                    <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::TYPE,
                ]);
            }

            impl dunge::instance::Set for Transform<'_> {
                fn set(&self, setter: &mut dunge::instance::Setter<'_, '_>) {
                    dunge::instance::SetMember::set_member(&self.pos, setter);
                    dunge::instance::SetMember::set_member(&self.col, setter);
                }
            }

            pub struct TransformProj<'slice> {
                pos: <Row<[f32; 2]> as dunge::instance::MemberProjection>::Field,
                col: <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::Field,
            }

            impl<'slice> dunge::instance::Projection for TransformProj<'slice> {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        pos: <Row<[f32; 2]> as dunge::instance::MemberProjection>::member_projection(id + 0u32),
                        col: <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::member_projection(id + 1u32),
                    }
                }
            }
        };

        helpers::eq_lines(&actual.to_string(), &expected.to_string());
    }

    #[test]
    fn derive_tuple_instance() {
        let input = quote::quote! {
            struct Transform<'slice>(Row<[f32; 2]>, RowSlice<'slice, [f32; 3]>);
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'slice> dunge::Instance for Transform<'slice> {
                type Projection = TransformProj<'static>;
                const DEF: dunge::sl::Define<dunge::types::ValueType> = dunge::sl::Define::new(&[
                    <Row<[f32; 2]> as dunge::instance::MemberProjection>::TYPE,
                    <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::TYPE,
                ]);
            }

            impl dunge::instance::Set for Transform<'_> {
                fn set(&self, setter: &mut dunge::instance::Setter<'_, '_>) {
                    dunge::instance::SetMember::set_member(&self.0, setter);
                    dunge::instance::SetMember::set_member(&self.1, setter);
                }
            }

            pub struct TransformProj<'slice>(
                <Row<[f32; 2]> as dunge::instance::MemberProjection>::Field,
                <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::Field,
            );

            impl<'slice> dunge::instance::Projection for TransformProj<'slice> {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        0: <Row<[f32; 2]> as dunge::instance::MemberProjection>::member_projection(id + 0u32),
                        1: <RowSlice<'slice, [f32; 3]> as dunge::instance::MemberProjection>::member_projection(id + 1u32),
                    }
                }
            }
        };

        helpers::eq_lines(&actual.to_string(), &expected.to_string());
    }
}
