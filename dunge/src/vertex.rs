use crate::vertex::private::Format;

/// Vertex type description.
///
/// To use a vertex type, you need to describe its fields.
/// For example, if a component of the vertex position is `[f32; 3]`, then the same type
/// must be specified in [`Position`](Vertex::Position) of the trait implementation.
/// If some component of the vertex is not used, the corresponding type must be specified as `()`.
///
/// Because all trait types must be [`Component`] no other types can be part of the vertex.
///
/// This implementation also requires some safety invariant, so the trait is `unsafe`.
/// You can safely implement the trait for your type using [deriving](derive@crate::Vertex).
///
/// # Safety
/// * The fields descriptions must exactly match the actual component types or be the unit `()`.
/// * The fields of `Self` must be ordered, so the struct must have the `#[repr(C)]` attribute.
/// * The fields must be in the same order as they are listed in the trait.
/// * The `Self` type must not have any other fields than those described by components.
///
/// # Example
/// Let's say we want to create a `Vert` type with a position and a texture map.
/// Then the [`Vertex`] implementation for the type would be:
/// ```rust
/// use dunge::Vertex;
///
/// #[repr(C)]
/// struct Vert {
///     pos: [f32; 3],
///     map: [f32; 2],
/// }
///
/// unsafe impl Vertex for Vert {
///     type Position = [f32; 3]; // position type
///     type Color = ();          // color component is not used
///     type Texture = [f32; 2];  // texture type
/// }
/// ```
///
/// Note that the implementation of the trait requires `unsafe` code,
/// so instead of writing this yourself you can use [deriving](derive@crate::Vertex):
/// ```rust
/// use dunge::Vertex;
///
/// #[repr(C)]
/// #[derive(Vertex)]
/// struct Vert {
///     #[position]
///     pos: [f32; 3],
///     #[texture]
///     map: [f32; 2],
/// }
/// ```
///
pub unsafe trait Vertex {
    type Position: Component;
    type Color: Component3D;
    type Texture: Component2D;
}

pub(crate) fn verts_as_bytes<V>(verts: &[V]) -> &[u8]
where
    V: Vertex,
{
    use std::{mem, slice};

    // Safety: all vertices consist of components, so they can be safely cast into bytes
    unsafe { slice::from_raw_parts(verts.as_ptr().cast(), mem::size_of_val(verts)) }
}

pub(crate) struct VertexInfo {
    pub dimensions: u64,
    pub has_color: bool,
    pub has_texture: bool,
}

impl VertexInfo {
    pub const fn new<V>() -> Self
    where
        V: Vertex,
    {
        Self {
            dimensions: V::Position::N_FLOATS,
            has_color: V::Color::OPTIONAL_N_FLOATS.is_some(),
            has_texture: V::Texture::OPTIONAL_N_FLOATS.is_some(),
        }
    }
}

/// The component is something that a [vertex](Vertex) can consist of.
///
/// Available componets are:
/// * `(f32, f32)`         
/// * `(f32, f32, f32)`    
/// * `[f32; 2]`           
/// * `[f32; 3]`           
/// * [`Vec2`](glam::Vec2)
/// * [`Vec3`](glam::Vec3)
///
pub trait Component: private::Component {
    const N_FLOATS: u64;
}

impl<C> Component for C
where
    C: private::Component,
{
    const N_FLOATS: u64 = C::Format::N_FLOATS;
}

/// The 2D component.
pub trait Component2D: private::OptionalComponent {
    const OPTIONAL_N_FLOATS: Option<u64>;
}

impl<C> Component2D for C
where
    C: Component<Format = private::FormatFloatX2>,
{
    const OPTIONAL_N_FLOATS: Option<u64> = Some(C::Format::N_FLOATS);
}

/// Specify this type if a component is not used.
impl Component2D for () {
    const OPTIONAL_N_FLOATS: Option<u64> = None;
}

/// The 3D component.
pub trait Component3D: private::OptionalComponent {
    const OPTIONAL_N_FLOATS: Option<u64>;
}

impl<C> Component3D for C
where
    C: Component<Format = private::FormatFloatX3>,
{
    const OPTIONAL_N_FLOATS: Option<u64> = Some(C::Format::N_FLOATS);
}

/// Specify this type if a component is not used.
impl Component3D for () {
    const OPTIONAL_N_FLOATS: Option<u64> = None;
}

mod private {
    use glam::{Vec2, Vec3};

    pub trait Format {
        const N_FLOATS: u64;
    }

    pub struct FormatFloatX2;
    impl Format for FormatFloatX2 {
        const N_FLOATS: u64 = 2;
    }

    pub struct FormatFloatX3;
    impl Format for FormatFloatX3 {
        const N_FLOATS: u64 = 3;
    }

    pub trait Component {
        type Format: Format;
    }

    impl Component for (f32, f32) {
        type Format = FormatFloatX2;
    }

    impl Component for (f32, f32, f32) {
        type Format = FormatFloatX3;
    }

    impl Component for [f32; 2] {
        type Format = FormatFloatX2;
    }

    impl Component for [f32; 3] {
        type Format = FormatFloatX3;
    }

    impl Component for Vec2 {
        type Format = FormatFloatX2;
    }

    impl Component for Vec3 {
        type Format = FormatFloatX3;
    }

    pub trait OptionalComponent {}
    impl<C> OptionalComponent for C where C: Component {}
    impl OptionalComponent for () {}
}
