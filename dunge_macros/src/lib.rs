mod vertex;

use proc_macro::TokenStream;

/// Derive implementation for the vector type.
#[proc_macro_derive(Vertex)]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    vertex::impl_vertex(input).into()
}
