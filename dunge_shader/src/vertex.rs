use {
    crate::{define::Define, types::VectorType},
    std::slice,
};

/// The vertex type description.
///
/// # Safety
/// The fields of [`Define`] must exactly match the actual struct fields.
/// To do this, the fields must be ordered, so the struct must have the `#[repr(C)]`
/// attribute and the fields must have the same order as specified in [`Define`].
///
/// # Deriving
/// Although the library tries to formalize the safety invariant, you still shouldn’t
/// implement the vertex yourself. The most reliable and simple way to do this is to
/// use a derive macro:
/// ```rust
/// # mod dunge {
/// #    // fake `Vertex` derive
/// #    pub use std::default::Default as Vertex;
/// # }
/// #
/// use dunge::Vertex;
///
/// #[repr(C)]
/// #[derive(Vertex)]
/// struct Vert {
///     pos: [f32; 2],
///     col: [f32; 3],
/// }
/// ```
///
pub unsafe trait Vertex {
    type Projection: Projection + 'static;
    const DEF: Define<VectorType>;
}

/// Maps the slice of vertices to the slice of bytes.
pub fn verts_as_bytes<V>(verts: &[V]) -> &[u8]
where
    V: Vertex,
{
    // SAFETY:
    // * The `Vertex` invariant states converting a slice of vertices to bytes is safe
    unsafe { slice::from_raw_parts(verts.as_ptr().cast(), size_of_val(verts)) }
}

/// Vertex type projection in a shader.
pub trait Projection {
    fn projection(id: u32) -> Self;
}
