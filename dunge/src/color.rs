//! Color types and traits.

/// A linear RGB(A) color.
#[derive(Clone, Copy)]
pub struct Linear<C, const N: usize = 4>(pub [C; N]);

impl<const N: usize> Linear<f64, N> {
    pub(crate) fn into_f32(self) -> Linear<f32, N> {
        Linear(self.0.map(|v| v as f32))
    }
}

/// A sRGB(A) color.
#[derive(Clone, Copy)]
pub struct Standard<C, const N: usize = 4>(pub [C; N]);

/// The trait for color conversion.
pub trait IntoLinear<const N: usize = 4> {
    fn into_linear(self) -> Linear<f64, N>;
}

impl<const N: usize> IntoLinear<N> for Standard<f64, N> {
    fn into_linear(self) -> Linear<f64, N> {
        fn to_linear(c: f64) -> f64 {
            if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }
        }

        Linear(self.0.map(to_linear))
    }
}

impl<const N: usize> IntoLinear<N> for Standard<f32, N> {
    fn into_linear(self) -> Linear<f64, N> {
        Standard(self.0.map(f64::from)).into_linear()
    }
}

impl<const N: usize> IntoLinear<N> for Linear<f64, N> {
    fn into_linear(self) -> Self {
        self
    }
}

impl<const N: usize> IntoLinear<N> for Linear<f32, N> {
    fn into_linear(self) -> Linear<f64, N> {
        Linear(self.0.map(f64::from))
    }
}

impl<const N: usize> IntoLinear<N> for Standard<u8, N> {
    fn into_linear(self) -> Linear<f64, N> {
        Standard(self.0.map(to_f64_color)).into_linear()
    }
}

impl<const N: usize> IntoLinear<N> for Linear<u8, N> {
    fn into_linear(self) -> Linear<f64, N> {
        Linear(self.0.map(to_f64_color))
    }
}

/// All color channels are zero.
impl<const N: usize> IntoLinear<N> for () {
    fn into_linear(self) -> Linear<f64, N> {
        use std::array;

        Linear(array::from_fn(|_| 0.))
    }
}

fn to_f64_color(c: u8) -> f64 {
    f64::from(c) / 255.
}
