use {proc_macro2::Ident, std::borrow::Cow};

pub(crate) fn make_ident(index: u32, ident: Option<&Ident>) -> Cow<Ident> {
    match ident {
        Some(ident) => Cow::Borrowed(ident),
        None => Cow::Owned(quote::format_ident!("{index}")),
    }
}
