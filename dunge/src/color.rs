//! Color types.

/// A linear RGB(A) color.
#[derive(Clone, Copy)]
pub struct Color<const N: usize>(pub [f32; N]);

impl<const N: usize> Color<N> {
    /// Creates a linear `Color` from sRGB(A) color.
    pub fn from_standard(col: [f32; N]) -> Self {
        fn to_linear(c: f32) -> f32 {
            if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }
        }

        Self(col.map(to_linear))
    }

    /// Creates a linear `Color` from a linear bytes.
    pub fn from_bytes(col: [u8; N]) -> Self {
        Self(col.map(to_f32_color))
    }

    /// Creates a linear `Color` from a sRGB(A) bytes.
    pub fn from_standard_bytes(col: [u8; N]) -> Self {
        Self::from_standard(col.map(to_f32_color))
    }
}

fn to_f32_color(c: u8) -> f32 {
    f32::from(c) / u8::MAX as f32
}

/// A linear RGB color.
pub type Rgb = Color<3>;

/// A linear RGBA color.
pub type Rgba = Color<4>;
