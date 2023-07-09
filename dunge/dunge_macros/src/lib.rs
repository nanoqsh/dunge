mod vertex;

use proc_macro::TokenStream;

/// Derive implementation for the vector type.
#[proc_macro_derive(Vertex, attributes(position, color, texture))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    use syn::DeriveInput;

    let derive = syn::parse_macro_input!(input as DeriveInput);
    vertex::impl_vertex(derive).into()
}
