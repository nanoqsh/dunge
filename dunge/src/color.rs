//! Color types and traits.

/// A linear RGBA color.
#[derive(Clone, Copy)]
pub struct Linear<C>(pub [C; 4]);

/// A sRGBA color.
#[derive(Clone, Copy)]
pub struct Srgba<C>(pub [C; 4]);

/// The trait for color conversion.
pub trait IntoLinear {
    fn into_linear(self) -> Linear<f64>;
}

impl IntoLinear for Srgba<f64> {
    fn into_linear(self) -> Linear<f64> {
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

impl IntoLinear for Srgba<f32> {
    fn into_linear(self) -> Linear<f64> {
        Srgba(self.0.map(f64::from)).into_linear()
    }
}

impl IntoLinear for Linear<f64> {
    fn into_linear(self) -> Self {
        self
    }
}

impl IntoLinear for Linear<f32> {
    fn into_linear(self) -> Linear<f64> {
        Linear(self.0.map(f64::from))
    }
}

impl IntoLinear for Srgba<u8> {
    fn into_linear(self) -> Linear<f64> {
        Srgba(self.0.map(to_f64_color)).into_linear()
    }
}

impl IntoLinear for Linear<u8> {
    fn into_linear(self) -> Linear<f64> {
        Linear(self.0.map(to_f64_color))
    }
}

fn to_f64_color(c: u8) -> f64 {
    f64::from(c) / 255.
}
