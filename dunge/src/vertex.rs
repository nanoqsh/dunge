/// Vertex type description.
///
/// To use a vertex type, you need to describe its fields.
/// This implementation also requires some safety invariant, so the trait is `unsafe`.
/// You can safely implement the trait for your type using [deriving](derive@crate::Vertex).
///
/// # Safety
/// * The fields of `Self` must be ordered, so the struct must have the `#[repr(C)]` attribute.
/// * The `FIELDS` const must describe correct [kinds](Kind) and [formats](Format).
/// * The [`check_format`](Field::check_format) must be true for every field.
///
/// # Example
/// Let's say we want to create a `Vert` type with a position and a texture map.
/// Then the [`Vertex`] implementation for the type would be:
/// ```rust
/// use dunge::{
///     vertex::{self, Field, Kind},
///     Vertex,
/// };
///
/// #[repr(C)]
/// struct Vert {
///     pos: [f32; 3],
///     map: [f32; 2],
/// }
///
/// unsafe impl Vertex for Vert {
///     const FIELDS: &'static [Field] = &[
///         {   // `pos` field
///             let f = Field {
///                 kind: Kind::Position,
///                 format: vertex::component_format::<[f32; 3]>(),
///             };
///
///             assert!(f.check_format());
///             f
///         },
///         {   // `map` field
///             let f = Field {
///                 kind: Kind::Color,
///                 format: vertex::component_format::<[f32; 2]>(),
///             };
///
///             assert!(f.check_format());
///             f
///         },
///     ];
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
///     #[texture_map]
///     map: [f32; 2],
/// }
/// ```
///
pub unsafe trait Vertex {
    const FIELDS: &'static [Field];
}

/// Field description.
#[derive(Clone, Copy)]
pub struct Field {
    pub kind: Kind,
    pub format: Format,
}

impl Field {
    /// Compiletime field check.
    #[must_use]
    pub const fn check_format(self) -> bool {
        matches!(
            (self.kind, self.format),
            (Kind::Position, Format::FloatX2 | Format::FloatX3)
                | (Kind::Color, Format::FloatX3)
                | (Kind::TextureMap, Format::FloatX2)
        )
    }
}

/// Field kind.
#[derive(Clone, Copy)]
pub enum Kind {
    /// Position of the vertex in 2D or 3D space.
    Position,

    /// Color of the vertex.
    Color,

    /// Texture map of the vertex.
    TextureMap,
}

/// Field format.
#[derive(Clone, Copy)]
pub enum Format {
    FloatX2,
    FloatX3,
}

/// The component is something that a [vertex](Vertex) can consist of.
///
/// Available componets are:
///
/// | Type                 | Possible [kind](Kind)                                          |
/// | -------------------- | -------------------------------------------------------------- |
/// | `(f32, f32)`         | [`Position`](Kind::Position), [`TextureMap`](Kind::TextureMap) |
/// | `(f32, f32, f32)`    | [`Position`](Kind::Position), [`Color`](Kind::Color)           |
/// | `[f32; 2]`           | [`Position`](Kind::Position), [`TextureMap`](Kind::TextureMap) |
/// | `[f32; 3]`           | [`Position`](Kind::Position), [`Color`](Kind::Color)           |
/// | [`Vec2`](glam::Vec2) | [`Position`](Kind::Position), [`TextureMap`](Kind::TextureMap) |
/// | [`Vec3`](glam::Vec3) | [`Position`](Kind::Position), [`Color`](Kind::Color)           |
///
pub trait Component: private::Component {}
impl<T> Component for T where T: private::Component {}

/// Takes a [format](Format) from the [component](Component).
#[must_use]
pub const fn component_format<T>() -> Format
where
    T: Component,
{
    T::FORMAT
}

mod private {
    use {
        crate::vertex::Format,
        glam::{Vec2, Vec3},
    };

    pub trait Component {
        const FORMAT: Format;
    }

    impl Component for (f32, f32) {
        const FORMAT: Format = Format::FloatX2;
    }

    impl Component for (f32, f32, f32) {
        const FORMAT: Format = Format::FloatX3;
    }

    impl Component for [f32; 2] {
        const FORMAT: Format = Format::FloatX2;
    }

    impl Component for [f32; 3] {
        const FORMAT: Format = Format::FloatX3;
    }

    impl Component for Vec2 {
        const FORMAT: Format = Format::FloatX2;
    }

    impl Component for Vec3 {
        const FORMAT: Format = Format::FloatX3;
    }
}
