//! Color types and traits.

/// A linear RGB(A) color.
#[derive(Clone, Copy)]
pub struct Linear<C, const N: usize = 4>(pub [C; N]);

/// A sRGB(A) color.
#[derive(Clone, Copy)]
pub struct Standard<C, const N: usize = 4>(pub [C; N]);

/// The trait for color conversion.
pub trait IntoLinear<const N: usize = 4> {
    fn into_linear(self) -> Linear<f32, N>;
}

impl<const N: usize> IntoLinear<N> for Standard<f32, N> {
    fn into_linear(self) -> Linear<f32, N> {
        fn to_linear(c: f32) -> f32 {
            if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }
        }

        Linear(self.0.map(to_linear))
    }
}

impl<const N: usize> IntoLinear<N> for Linear<f32, N> {
    fn into_linear(self) -> Self {
        self
    }
}

impl<const N: usize> IntoLinear<N> for Standard<u8, N> {
    fn into_linear(self) -> Linear<f32, N> {
        Standard(self.0.map(to_f32_color)).into_linear()
    }
}

impl<const N: usize> IntoLinear<N> for Linear<u8, N> {
    fn into_linear(self) -> Linear<f32, N> {
        Linear(self.0.map(to_f32_color))
    }
}

/// All color channels are zero.
impl<const N: usize> IntoLinear<N> for () {
    fn into_linear(self) -> Linear<f32, N> {
        use std::array;

        Linear(array::from_fn(|_| 0.))
    }
}

fn to_f32_color(c: u8) -> f32 {
    f32::from(c) / 255.
}
