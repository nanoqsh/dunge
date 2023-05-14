use {
    proc_macro2::{Ident, Span, TokenStream},
    syn::{Attribute, Data, DataStruct, DeriveInput, Field},
};

pub(crate) fn impl_vertex(derive: DeriveInput) -> TokenStream {
    use quote::ToTokens;

    let Data::Struct(DataStruct { fields, .. }) = derive.data else {
        panic!("the vertex type must be a struct");
    };

    assert!(
        !fields.is_empty(),
        "the vertex struct must have some fields",
    );

    assert!(
        derive.generics.params.is_empty(),
        "the vertex struct cannot have generic parameters",
    );

    assert!(
        derive.attrs.iter().any(is_repr_c),
        "the vertex struct must have the `#[repr(C)]` attribute",
    );

    let mut has_position = false;
    let fields: Vec<_> = (0..)
        .zip(fields)
        .map(|(n, field)| {
            let Some(kind) = field_kind(&field) else {
                panic!(
                    "the `{name}` field must be specified by some vertex attribute",
                    name = match field.ident {
                        Some(ident) => ident.to_string(),
                        _ => n.to_string(),
                    },
                );
            };

            if let Kind::Position = kind {
                assert!(
                    !has_position,
                    "fields must have only one `#[position]` attribute",
                );

                has_position = true;
            }

            let ty = field.ty;
            let msg = format!(
                "wrong vertex attribute `#[{at}]` for `{ty}` type",
                at = kind.as_attr(),
                ty = ty.to_token_stream(),
            );

            let kind = kind.into_ident();
            quote::quote! {{
                let f = ::dunge::vertex::Field {
                    kind: ::dunge::vertex::Kind::#kind,
                    format: ::dunge::vertex::component_format::<#ty>(),
                };

                ::std::assert!(::dunge::vertex::Field::check_format(f), #msg);
                f
            }}
        })
        .collect();

    assert!(
        has_position,
        "some field must have the `#[position]` attribute",
    );

    let type_name = derive.ident;
    quote::quote! {
        unsafe impl ::dunge::vertex::Vertex for #type_name {
            const FIELDS: &'static [::dunge::vertex::Field] = &[#(#fields),*];
        }

        const _: &'static [::dunge::vertex::Field] = <#type_name as ::dunge::vertex::Vertex>::FIELDS;
    }
}

fn is_repr_c(attr: &Attribute) -> bool {
    attr.path().is_ident("repr")
        && attr
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    Ok(())
                } else {
                    Err(meta.error("unrecognized repr"))
                }
            })
            .is_ok()
}

enum Kind {
    Position,
    Color,
    TextureMap,
}

impl Kind {
    fn into_ident(self) -> Ident {
        let s = match self {
            Self::Position => "Position",
            Self::Color => "Color",
            Self::TextureMap => "TextureMap",
        };

        Ident::new(s, Span::call_site())
    }

    fn as_attr(&self) -> &'static str {
        match self {
            Self::Position => "position",
            Self::Color => "color",
            Self::TextureMap => "texture_map",
        }
    }
}

fn field_kind(field: &Field) -> Option<Kind> {
    use syn::Meta;

    field.attrs.iter().find_map(|attr| match &attr.meta {
        Meta::Path(path) if path.is_ident("position") => Some(Kind::Position),
        Meta::Path(path) if path.is_ident("color") => Some(Kind::Color),
        Meta::Path(path) if path.is_ident("texture_map") => Some(Kind::TextureMap),
        _ => None,
    })
}
