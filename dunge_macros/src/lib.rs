mod group;
mod member;
mod vertex;

use proc_macro::TokenStream;

/// Derive implementation for the group type.
#[proc_macro_derive(Group)]
pub fn derive_group(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    group::derive(input).into()
}

/// Derive implementation for the vector type.
#[proc_macro_derive(Vertex)]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    vertex::derive(input).into()
}
