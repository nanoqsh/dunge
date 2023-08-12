use {
    proc_macro2::TokenStream,
    syn::{spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Field, Type},
};

pub(crate) fn impl_vertex(derive: DeriveInput) -> TokenStream {
    use quote::ToTokens;

    let Data::Struct(DataStruct { fields, .. }) = derive.data else {
        return quote::quote_spanned! { derive.ident.span() =>
            ::std::compile_error!("the vertex type must be a struct");
        };
    };

    if !derive.generics.params.is_empty() {
        return quote::quote_spanned! { derive.generics.params.span() =>
            ::std::compile_error!("the vertex struct cannot have generic parameters");
        };
    }

    if !derive.attrs.iter().any(is_repr_c) {
        return quote::quote_spanned! { derive.ident.span() =>
            ::std::compile_error!("the vertex struct must have the `#[repr(C)]` attribute");
        };
    }

    if fields.is_empty() {
        return quote::quote_spanned! { fields.span() =>
            ::std::compile_error!("the vertex struct must have some fields");
        };
    }

    let mut fl = fields.iter().peekable();
    let Some(Fl { kind: Kind::Pos, ty: pos }) = fl.next().and_then(Fl::new) else {
        return quote::quote_spanned! { fields.span() =>
            ::std::compile_error!("the first field must have the `#[position]` attribute");
        };
    };

    let col = match fl.peek().copied().and_then(Fl::new) {
        Some(Fl {
            kind: Kind::Col,
            ty,
        }) => {
            _ = fl.next();
            Some(ty)
        }
        _ => None,
    };

    let tex = match fl.peek().copied().and_then(Fl::new) {
        Some(Fl {
            kind: Kind::Tex,
            ty,
        }) => {
            _ = fl.next();
            Some(ty)
        }
        _ => None,
    };

    if let Some(field) = fl.next() {
        let msg = match Kind::from_field(field) {
            Some(_) => "this field is redundant",
            None => "this field is unspecified",
        };

        return quote::quote_spanned! { field.span() => ::std::compile_error!(#msg); };
    }

    let type_name = derive.ident;
    let col = col.map_or(quote::quote!(()), ToTokens::into_token_stream);
    let tex = tex.map_or(quote::quote!(()), ToTokens::into_token_stream);
    quote::quote! {
        unsafe impl ::dunge::vertex::Vertex for #type_name {
            type Position = #pos;
            type Color = #col;
            type Texture = #tex;
        }
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

struct Fl<'a> {
    kind: Kind,
    ty: &'a Type,
}

impl<'a> Fl<'a> {
    fn new(field: &'a Field) -> Option<Self> {
        Some(Self {
            kind: Kind::from_field(field)?,
            ty: &field.ty,
        })
    }
}

enum Kind {
    Pos,
    Col,
    Tex,
}

impl Kind {
    fn from_field(field: &Field) -> Option<Self> {
        use syn::Meta;

        field.attrs.iter().find_map(|attr| match &attr.meta {
            Meta::Path(path) if path.is_ident("position") => Some(Self::Pos),
            Meta::Path(path) if path.is_ident("color") => Some(Self::Col),
            Meta::Path(path) if path.is_ident("texture") => Some(Self::Tex),
            _ => None,
        })
    }
}
