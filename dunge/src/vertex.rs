/// Vertex type description.
///
/// # Safety
/// * The `FIELDS` const must describe correct [kinds](Kind) and [formats](Format).
/// * The [`check_format`](Field::check_format) must be true for every field.
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
    Position,
    Color,
    TextureMap,
}

/// Field format.
#[derive(Clone, Copy)]
pub enum Format {
    FloatX2,
    FloatX3,
}

pub trait Component: private::Component {}
impl<T> Component for T where T: private::Component {}

/// Takes [format](Format) from component.
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
