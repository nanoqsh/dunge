use {
    proc_macro2::{Ident, Span},
    syn::{Index, Member},
};

pub(crate) fn make(index: u32, ident: Option<Ident>) -> Member {
    match ident {
        Some(ident) => Member::Named(ident),
        None => Member::Unnamed(Index {
            index,
            span: Span::call_site(),
        }),
    }
}
