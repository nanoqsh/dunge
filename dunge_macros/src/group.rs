use {
    crate::member,
    proc_macro2::{Span, TokenStream},
    syn::{spanned::Spanned, Data, DataStruct, DeriveInput, GenericParam, Ident, Lifetime},
};

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    use std::iter;

    let Data::Struct(DataStruct { fields, .. }) = input.data else {
        return quote::quote_spanned! { input.ident.span() =>
            ::std::compile_error!("the group type must be a struct");
        };
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
        quote::quote! { <#ty as ::dunge::group::MemberProjection>::TYPE }
    });

    let group_visit_members = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        quote::quote! { ::dunge::bind::VisitMember::visit_member(self.#ident, visitor) }
    });

    let group_fields = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::group::MemberProjection>::Field }
    });

    let group_member_projections = iter::zip(0.., &fields).map(|(index, field)| {
        let ident = member::make(index, field.ident.clone());
        let ty = &field.ty;
        quote::quote! { #ident: <#ty as ::dunge::group::MemberProjection>::member_projection(id, #index, out.clone()) }
    });

    quote::quote! {
        impl<#(#lts),*> ::dunge::Group for #name<#(#lts),*> {
            type Projection = #projection_name<#(#static_lts),*>;
            const DEF: ::dunge::sl::Define<::dunge::types::MemberType> = ::dunge::sl::Define::new(&[
                #(#group_types),*,
            ]);
        }

        impl ::dunge::bind::Visit for #name<#(#anon_lts),*> {
            fn visit<'a>(&'a self, visitor: &mut ::dunge::bind::Visitor<'a>) {
                #(#group_visit_members);*;
            }
        }

        struct #projection_name<#(#lts),*> {
            #(#group_fields),*,
        }

        impl<#(#lts),*> ::dunge::group::Projection for #projection_name<#(#lts),*> {
            fn projection(id: ::core::primitive::u32, out: ::dunge::sl::GlobalOut) -> Self {
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
            struct Map<'a> {
                tex: BoundTexture<'a>,
                sam: &'a Sampler,
            }
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'a> ::dunge::Group for Map<'a> {
                type Projection = MapProjection<'static>;
                const DEF: ::dunge::sl::Define<::dunge::types::MemberType> = ::dunge::sl::Define::new(&[
                    <BoundTexture<'a> as ::dunge::group::MemberProjection>::TYPE,
                    <&'a Sampler as ::dunge::group::MemberProjection>::TYPE,
                ]);
            }

            impl ::dunge::bind::Visit for Map<'_> {
                fn visit<'a>(&'a self, visitor: &mut ::dunge::bind::Visitor<'a>) {
                    ::dunge::bind::VisitMember::visit_member(self.tex, visitor);
                    ::dunge::bind::VisitMember::visit_member(self.sam, visitor);
                }
            }

            struct MapProjection<'a> {
                tex: <BoundTexture<'a> as ::dunge::group::MemberProjection>::Field,
                sam: <&'a Sampler as ::dunge::group::MemberProjection>::Field,
            }

            impl<'a> ::dunge::group::Projection for MapProjection<'a> {
                fn projection(id: ::core::primitive::u32, out: ::dunge::sl::GlobalOut) -> Self {
                    Self {
                        tex: <BoundTexture<'a> as ::dunge::group::MemberProjection>::member_projection(id, 0u32, out.clone()),
                        sam: <&'a Sampler as ::dunge::group::MemberProjection>::member_projection(id, 1u32, out.clone()),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn derive_tuple_group() {
        let input = quote::quote! {
            struct Map<'a>(BoundTexture<'a>, &'a Sampler);
        };

        let input = syn::parse2(input).expect("parse input");
        let actual = derive(input);
        let expected = quote::quote! {
            impl<'a> ::dunge::Group for Map<'a> {
                type Projection = MapProjection<'static>;
                const DEF: ::dunge::sl::Define<::dunge::types::MemberType> = ::dunge::sl::Define::new(&[
                    <BoundTexture<'a> as ::dunge::group::MemberProjection>::TYPE,
                    <&'a Sampler as ::dunge::group::MemberProjection>::TYPE,
                ]);
            }

            impl ::dunge::bind::Visit for Map<'_> {
                fn visit<'a>(&'a self, visitor: &mut ::dunge::bind::Visitor<'a>) {
                    ::dunge::bind::VisitMember::visit_member(self.0, visitor);
                    ::dunge::bind::VisitMember::visit_member(self.1, visitor);
                }
            }

            struct MapProjection<'a> {
                0: <BoundTexture<'a> as ::dunge::group::MemberProjection>::Field,
                1: <&'a Sampler as ::dunge::group::MemberProjection>::Field,
            }

            impl<'a> ::dunge::group::Projection for MapProjection<'a> {
                fn projection(id: ::core::primitive::u32, out: ::dunge::sl::GlobalOut) -> Self {
                    Self {
                        0: <BoundTexture<'a> as ::dunge::group::MemberProjection>::member_projection(id, 0u32, out.clone()),
                        1: <&'a Sampler as ::dunge::group::MemberProjection>::member_projection(id, 1u32, out.clone()),
                    }
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
