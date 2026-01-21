use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn dunge(attr: TokenStream, item: TokenStream) -> TokenStream {
    dunge_irgen::shader(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn render(code: TokenStream) -> TokenStream {
    dunge_irgen::make_render(code.into()).into()
}

#[proc_macro_derive(Bytes)]
pub fn derive_bytes(item: TokenStream) -> TokenStream {
    dunge_irgen::derive_bytes(item.into()).into()
}

#[proc_macro_derive(Value)]
pub fn derive_value(item: TokenStream) -> TokenStream {
    dunge_irgen::derive_value(item.into()).into()
}

#[proc_macro_derive(Input)]
pub fn derive_input(item: TokenStream) -> TokenStream {
    dunge_irgen::derive_input(item.into()).into()
}
