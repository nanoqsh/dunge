use {
    proc_macro2::{Span, TokenStream},
    std::{convert::Infallible, fmt},
};

pub(crate) type Parse<T> = Result<T, Error>;

pub(crate) enum Error {
    Syn(syn::Error),
    Unsupported(Span),
    UnknownAttribute(Span),
}

impl Error {
    pub(crate) fn into_compile_error(self) -> TokenStream {
        match self {
            Self::Syn(e) => e.into_compile_error(),
            Self::Unsupported(span) => quote::quote_spanned! { span =>
                ::std::compile_error!("this syntax is not supported");
            },
            Self::UnknownAttribute(span) => quote::quote_spanned! { span =>
                ::std::compile_error!("unknown attribute");
            },
        }
    }
}

pub(crate) trait SpanExt: Sized {
    fn err(self) -> Result<Infallible, Error>;
}

impl SpanExt for Span {
    fn err(self) -> Result<Infallible, Error> {
        Err(Error::Unsupported(self))
    }
}

pub(crate) fn into_compile_error<E>(e: E) -> TokenStream
where
    E: fmt::Display,
{
    let message = e.to_string();
    quote::quote! {
        ::std::compile_error!(#message);
    }
}
