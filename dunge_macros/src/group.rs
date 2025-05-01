use {
    crate::member,
    proc_macro2::{Span, TokenStream},
    syn::{Data, DataStruct, DeriveInput, Fields, GenericParam, Ident, Lifetime, spanned::Spanned},
};

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    use std::iter;

    let Data::Struct(DataStruct { fields, .. }) = input.data else {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the group type must be a struct");
        };
    };

    let named = match &fields {
        Fields::Named(_) => true,
        Fields::Unnamed(_) => false,
        Fields::Unit => {
            return quote::quote_spanned! { input.ident.span() =>
                ::std::compile_error!("the group type cannot be a unit struct");
            };
        }
    };

    let mut lts = Vec::with_capacity(input.generics.params.len());
    for param in input.generics.params {
        let GenericParam::Lifetime(param) = param else {
            return quote::quote_spanned! { param.span() =>
                ::std::compile_error!("the group struct cannot have non-lifetime generic parameters");
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
            ::std::compile_error!("the group struct must have some fields");
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
    let projection_name = quote::format_ident!("{name}Projection");
    let group_types = fields.iter().map(|field| {
        let ty = &field.ty;
        quote::quote! { <#ty as dunge::group::MemberProjection>::MEMBER }
    });

    let n_members = fields.len();
    let group_visit_members = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        quote::quote! { dunge::set::VisitMember::visit_member(self.#ident, visitor) }
    });

    let group_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        if named {
            quote::quote! { #ident: <#ty as dunge::group::MemberProjection>::Field }
        } else {
            quote::quote! { <#ty as dunge::group::MemberProjection>::Field }
        }
    });

    let group_member_projections = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as dunge::group::MemberProjection>::member_projection(id, #index, out.clone()) }
    });

    let projection = if named {
        quote::quote! {
            pub struct #projection_name<#(#lts),*> {
                #(#group_fields),*,
            }
        }
    } else {
        quote::quote! {
            pub struct #projection_name<#(#lts),*>(
                #(#group_fields),*,
            );
        }
    };

    quote::quote! {
        impl<#(#lts),*> dunge::Group for #name<#(#lts),*> {
            type Projection = #projection_name<#(#static_lts),*>;
            const DEF: dunge::sl::Define<dunge::types::MemberData> = dunge::sl::Define::new(&[
                #(#group_types),*,
            ]);
        }

        impl dunge::set::Visit for #name<#(#anon_lts),*> {
            const N_MEMBERS: ::core::primitive::usize = #n_members;
            fn visit<'group>(&'group self, visitor: &mut dunge::set::Visitor<'group>) {
                #(#group_visit_members);*;
            }
        }

        #projection

        impl<#(#lts),*> dunge::group::Projection for #projection_name<#(#lts),*> {
            fn projection(id: ::core::primitive::u32, out: dunge::sl::GlobalOut) -> Self {
                Self {
                    #(#group_member_projections),*,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_group() {
        let input = quote::quote! {
            struct Map<'tx> {
                tex: BoundTexture<'tx>,
                sam: &'tx Sampler,
            }
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'tx> dunge::Group for Map<'tx> {
                type Projection = MapProjection<'static>;
                const DEF: dunge::sl::Define<dunge::types::MemberData> = dunge::sl::Define::new(&[
                    <BoundTexture<'tx> as dunge::group::MemberProjection>::MEMBER,
                    <&'tx Sampler as dunge::group::MemberProjection>::MEMBER,
                ]);
            }

            impl dunge::set::Visit for Map<'_> {
                const N_MEMBERS: ::core::primitive::usize = 2usize;
                fn visit<'group>(&'group self, visitor: &mut dunge::set::Visitor<'group>) {
                    dunge::set::VisitMember::visit_member(self.tex, visitor);
                    dunge::set::VisitMember::visit_member(self.sam, visitor);
                }
            }

            pub struct MapProjection<'tx> {
                tex: <BoundTexture<'tx> as dunge::group::MemberProjection>::Field,
                sam: <&'tx Sampler as dunge::group::MemberProjection>::Field,
            }

            impl<'tx> dunge::group::Projection for MapProjection<'tx> {
                fn projection(id: ::core::primitive::u32, out: dunge::sl::GlobalOut) -> Self {
                    Self {
                        tex: <BoundTexture<'tx> as dunge::group::MemberProjection>::member_projection(id, 0u32, out.clone()),
                        sam: <&'tx Sampler as dunge::group::MemberProjection>::member_projection(id, 1u32, out.clone()),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn derive_tuple_group() {
        let input = quote::quote! {
            struct Map<'tx>(BoundTexture<'tx>, &'tx Sampler);
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'tx> dunge::Group for Map<'tx> {
                type Projection = MapProjection<'static>;
                const DEF: dunge::sl::Define<dunge::types::MemberData> = dunge::sl::Define::new(&[
                    <BoundTexture<'tx> as dunge::group::MemberProjection>::MEMBER,
                    <&'tx Sampler as dunge::group::MemberProjection>::MEMBER,
                ]);
            }

            impl dunge::set::Visit for Map<'_> {
                const N_MEMBERS: ::core::primitive::usize = 2usize;
                fn visit<'group>(&'group self, visitor: &mut dunge::set::Visitor<'group>) {
                    dunge::set::VisitMember::visit_member(self.0, visitor);
                    dunge::set::VisitMember::visit_member(self.1, visitor);
                }
            }

            pub struct MapProjection<'tx>(
                <BoundTexture<'tx> as dunge::group::MemberProjection>::Field,
                <&'tx Sampler as dunge::group::MemberProjection>::Field,
            );

            impl<'tx> dunge::group::Projection for MapProjection<'tx> {
                fn projection(id: ::core::primitive::u32, out: dunge::sl::GlobalOut) -> Self {
                    Self {
                        0: <BoundTexture<'tx> as dunge::group::MemberProjection>::member_projection(id, 0u32, out.clone()),
                        1: <&'tx Sampler as dunge::group::MemberProjection>::member_projection(id, 1u32, out.clone()),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
