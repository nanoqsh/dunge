use {
    crate::member,
    proc_macro2::TokenStream,
    syn::{spanned::Spanned, Data, DataStruct, DeriveInput, Fields},
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
            }
        }
    };

    if !input.generics.params.is_empty() {
        return quote::quote_spanned! { input.generics.params.span() =>
            ::std::compile_error!("the instance struct cannot have generic parameters");
        };
    }

    if fields.is_empty() {
        return quote::quote_spanned! { fields.span() =>
            ::std::compile_error!("the instance struct must have some fields");
        };
    }

    let name = input.ident;
    let projection_name = quote::format_ident!("{name}Projection");
    let instance_types = fields.iter().map(|field| {
        let ty = &field.ty;
        quote::quote! { <#ty as ::dunge::instance::MemberProjection>::TYPE }
    });

    let instance_set_members = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        quote::quote! { ::dunge::instance::SetMember::set_member(&self.#ident, setter) }
    });

    let instance_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        if named {
            quote::quote! { #ident: <#ty as ::dunge::instance::MemberProjection>::Field }
        } else {
            quote::quote! { <#ty as ::dunge::instance::MemberProjection>::Field }
        }
    });

    let instance_member_projections = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::instance::MemberProjection>::member_projection(id + #index) }
    });

    let projection = if named {
        quote::quote! {
            pub struct #projection_name {
                #(#instance_fields),*,
            }
        }
    } else {
        quote::quote! {
            pub struct #projection_name(
                #(#instance_fields),*,
            );
        }
    };

    quote::quote! {
        impl ::dunge::Instance for #name {
            type Projection = #projection_name;
            const DEF: ::dunge::sl::Define<::dunge::types::ValueType> = ::dunge::sl::Define::new(&[
                #(#instance_types),*,
            ]);
        }

        impl ::dunge::instance::Set for #name {
            fn set<'p>(&'p self, setter: &mut ::dunge::instance::Setter<'_, 'p>) {
                #(#instance_set_members);*;
            }
        }

        #projection

        impl ::dunge::instance::Projection for #projection_name {
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
            struct Transform {
                pos: Row<[f32; 2]>,
                col: Row<[f32; 3]>,
            }
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl ::dunge::Instance for Transform {
                type Projection = TransformProjection;
                const DEF: ::dunge::sl::Define<::dunge::types::ValueType> = ::dunge::sl::Define::new(&[
                    <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::TYPE,
                    <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::TYPE,
                ]);
            }

            impl ::dunge::instance::Set for Transform {
                fn set<'p>(&'p self, setter: &mut ::dunge::instance::Setter<'_, 'p>) {
                    ::dunge::instance::SetMember::set_member(&self.pos, setter);
                    ::dunge::instance::SetMember::set_member(&self.col, setter);
                }
            }

            pub struct TransformProjection {
                pos: <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::Field,
                col: <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::Field,
            }

            impl ::dunge::instance::Projection for TransformProjection {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        pos: <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::member_projection(id + 0u32),
                        col: <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::member_projection(id + 1u32),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn derive_tuple_instance() {
        let input = quote::quote! {
            struct Transform(Row<[f32; 2]>, Row<[f32; 3]>);
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl ::dunge::Instance for Transform {
                type Projection = TransformProjection;
                const DEF: ::dunge::sl::Define<::dunge::types::ValueType> = ::dunge::sl::Define::new(&[
                    <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::TYPE,
                    <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::TYPE,
                ]);
            }

            impl ::dunge::instance::Set for Transform {
                fn set<'p>(&'p self, setter: &mut ::dunge::instance::Setter<'_, 'p>) {
                    ::dunge::instance::SetMember::set_member(&self.0, setter);
                    ::dunge::instance::SetMember::set_member(&self.1, setter);
                }
            }

            pub struct TransformProjection(
                <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::Field,
                <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::Field,
            );

            impl ::dunge::instance::Projection for TransformProjection {
                fn projection(id: ::core::primitive::u32) -> Self {
                    Self {
                        0: <Row<[f32; 2]> as ::dunge::instance::MemberProjection>::member_projection(id + 0u32),
                        1: <Row<[f32; 3]> as ::dunge::instance::MemberProjection>::member_projection(id + 1u32),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
