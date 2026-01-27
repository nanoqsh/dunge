use {
    crate::error::*,
    proc_macro2::{Ident, TokenStream},
    std::fmt,
    syn::spanned::Spanned,
};

pub(crate) fn parse(stream: TokenStream) -> Parse<Struct> {
    let input: syn::DeriveInput = syn::parse2(stream).map_err(Error::Syn)?;

    for param in input.generics.params {
        param.span().err()?;
    }

    let data = match input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(syn::DataEnum { enum_token, .. }) => match enum_token.span().err()? {},
        syn::Data::Union(syn::DataUnion { union_token, .. }) => match union_token.span().err()? {},
    };

    let fields = data
        .fields
        .into_iter()
        .map(|f| {
            let Some(ident) = f.ident else {
                match f.span().err()? {}
            };

            let attr = match f.attrs.as_slice() {
                [attr] => {
                    let path = attr.path();
                    if path.is_ident("position") {
                        Attribute::Position
                    } else if path.is_ident("index") {
                        Attribute::Index
                    } else {
                        Attribute::None
                    }
                }
                _ => Attribute::None,
            };

            Ok(Field {
                attr,
                vis: f.vis,
                ident,
                ty: f.ty,
            })
        })
        .collect::<Parse<_>>()?;

    Ok(Struct {
        vis: input.vis,
        ident: input.ident,
        fields,
    })
}

fn non_empty_fields(fields: &[Field]) -> Derive<impl Iterator<Item = &Field>> {
    if fields.is_empty() {
        Err(DeriveError::NoFields)
    } else {
        Ok(fields.iter())
    }
}

pub(crate) struct Struct {
    vis: syn::Visibility,
    ident: Ident,
    fields: Vec<Field>,
}

enum Attribute {
    None,
    Position,
    Index,
}

struct Field {
    attr: Attribute,
    vis: syn::Visibility,
    ident: Ident,
    ty: syn::Type,
}

pub(crate) fn derive_bytes(input: &Struct, path: &TokenStream) -> Derive<TokenStream> {
    let bytes = quote::quote! { #path::bytes };

    let ident = &input.ident;
    let field_types = non_empty_fields(&input.fields)?.map(|f| &f.ty);

    let message = format!("type `{ident}` must not contain any paddings");
    let assert = quote::quote! {
        const _: () = {
            let size = #(#bytes::bytes_size::<#field_types>())+*;
            ::std::assert!(size == ::std::mem::size_of::<#ident>(), #message);
        };
    };

    let derive = quote::quote! {
        unsafe impl #bytes::Bytes for #ident {}
    };

    Ok(quote::quote! {
        #assert
        #derive
    })
}

pub(crate) fn derive_value(input: &Struct, path: &TokenStream) -> Derive<TokenStream> {
    let attr = quote::quote! { #path::attr };
    let irc = quote::quote! { #path::irc };

    let ident = &input.ident;
    let members = non_empty_fields(&input.fields)?.map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        let ident_string = ident.to_string();

        let binding = match f.attr {
            Attribute::None => quote::quote! { #irc::Binding::None },
            Attribute::Position => quote::quote! {{
                #attr::is_position::<#ty>();
                #irc::Binding::Position
            }},
            Attribute::Index => quote::quote! {{
                #attr::is_index::<#ty>();
                #irc::Binding::Index
            }},
        };

        quote::quote! {
            .add_member::<#ty>(
                #ident_string,
                #binding,
                ::std::mem::offset_of!(Self, #ident) as ::std::primitive::u32,
            )
        }
    });

    let naga = quote::quote! {
        const NAGA: #irc::Type = #irc::Type::dynamic::<Self>(
            |b| b.build_struct::<Self>()
                #(#members)*
                .build(),
            #irc::ArraySize::No,
        );
    };

    let member_idents: Vec<_> = non_empty_fields(&input.fields)?.map(|f| &f.ident).collect();
    let member_offsets = non_empty_fields(&input.fields)?.map(|f| {
        let ident = &f.ident;
        quote::quote! { ::std::mem::offset_of!(Self, #ident) }
    });

    let expr = quote::quote! {
        fn expr(self, fnc: &mut #irc::Fnc<'_>) -> #irc::Expr<Self> {
            #(
                let #member_idents = #irc::Value::expr(self.#member_idents, fnc);
            )*

            fnc.do_compose_tuple_with_permutation(
                (#(#member_idents,)*),
                const {
                    #irc::inverse_permutation(#irc::indices([
                        #(#member_offsets,)*
                    ]))
                },
            )
        }
    };

    Ok(quote::quote! {
        impl #irc::Value for #ident {
            #naga
            #expr
        }
    })
}

pub(crate) fn derive_input(input: &Struct, path: &TokenStream) -> Derive<TokenStream> {
    let irc = quote::quote! { #path::irc };

    let ident = &input.ident;
    let members = non_empty_fields(&input.fields)?.map(|f| {
        let ty = &f.ty;
        quote::quote! { .add_member::<#ty>() }
    });

    Ok(quote::quote! {
        impl #irc::Input for #ident {
            const KIND: #irc::InputKind = #irc::InputKind::Group;
            type Ref = Self;

            fn input(fnc: &mut #irc::Fnc<'_>) -> impl #irc::Reference<Self::Ref> + use<> {
                fnc.build_group::<Self>()
                    #(#members)*
                    .build()
            }
        }
    })
}

pub(crate) enum Reorder {
    No,
    Yes,
}

pub(crate) fn derive_fields(
    input: &Struct,
    re: Reorder,
    path: &TokenStream,
) -> Derive<TokenStream> {
    let irc = quote::quote! { #path::irc };

    let vis = &input.vis;
    let ident = &input.ident;
    let fields_ident = quote::format_ident!("{ident}Fields");
    let fields_access = non_empty_fields(&input.fields)?.map(|f| {
        let vis = &f.vis;
        let member_ident = &f.ident;
        let ty = &f.ty;
        quote::quote! { #vis #member_ident: #irc::Access<#ident, #ty> }
    });

    let types = non_empty_fields(&input.fields)?.map(|f| &f.ty);

    let fields_value = match re {
        Reorder::No => {
            let fields = non_empty_fields(&input.fields)?
                .enumerate()
                .map(|(index, f)| {
                    let ident = &f.ident;
                    let index = index as u32;
                    quote::quote! { #ident: #irc::index(#index) }
                });

            quote::quote! {
                #fields_ident {
                    #(#fields,)*
                }
            }
        }
        Reorder::Yes => {
            let member_offsets = non_empty_fields(&input.fields)?.map(|f| {
                let ident = &f.ident;
                quote::quote! { ::std::mem::offset_of!(Self, #ident) }
            });

            let fields = non_empty_fields(&input.fields)?
                .enumerate()
                .map(|(index, f)| {
                    let ident = &f.ident;
                    quote::quote! { #ident: #irc::index(indices[#index] as ::std::primitive::u32) }
                });

            quote::quote! {
                let indices = #irc::indices([
                    #(#member_offsets,)*
                ]);

                #fields_ident {
                    #(#fields,)*
                }
            }
        }
    };

    Ok(quote::quote! {
        const _: () = {
            #vis struct #fields_ident {
                #(#fields_access,)*
            }

            impl #irc::Fields for #ident {
                type Tuple = (#(#types,)*);
                type Fields = #fields_ident;

                const FIELDS: Self::Fields = {
                    #fields_value
                };
            }
        };
    })
}

pub(crate) fn derive_group(input: &Struct, path: &TokenStream) -> Derive<TokenStream> {
    let set = quote::quote! { #path::set };

    let ident = &input.ident;
    let members = non_empty_fields(&input.fields)?.map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote::quote! { <#ty as #set::Group>::group(&self.#ident, e) }
    });

    Ok(quote::quote! {
        impl #set::Group for #ident {
            fn group<'group>(&'group self, e: &mut dunge::set::Entries<'group>) {
                #(#members;)*
            }
        }
    })
}

type Derive<T> = Result<T, DeriveError>;

pub(crate) enum DeriveError {
    NoFields,
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFields => f.write_str("the struct has no fields"),
        }
    }
}
