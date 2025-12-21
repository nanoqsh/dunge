//! Color types.

use bytemuck::{Pod, Zeroable};

/// A linear RGB color.
pub type Rgb = Color<3>;

/// A linear RGBA color.
pub type Rgba = Color<4>;

/// A linear RGB(A) color.
#[repr(transparent)]
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

impl Color<4> {
    pub(crate) fn wgpu(self) -> wgpu::Color {
        let [r, g, b, a] = self.0.map(f64::from);
        wgpu::Color { r, g, b, a }
    }
}

// SAFETY: `Color` is repr transparent
// so if `[f32; N]: Zeroable` then `Color<N>: Zeroable`.
unsafe impl<const N: usize> Zeroable for Color<N> where [f32; N]: Zeroable {}

// SAFETY: `Color` is repr transparent
// so if `[f32; N]: Pod` then `Color<N>: Pod`.
unsafe impl<const N: usize> Pod for Color<N> where [f32; N]: Pod {}

fn to_f32_color(c: u8) -> f32 {
    f32::from(c) / f32::from(u8::MAX)
}

/// The texture format type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    SrgbAlpha,
    SbgrAlpha,
    RgbAlpha,
    BgrAlpha,
    Depth,
    Byte,
}

impl Format {
    /// Returns the number of bytes per pixel for this format.
    pub const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
    }

    /// Returns `true` if the format is a standard sRGB variant.
    pub const fn is_standard(self) -> bool {
        matches!(self, Self::SrgbAlpha | Self::SbgrAlpha)
    }

    pub(crate) const fn wgpu(self) -> wgpu::TextureFormat {
        match self {
            Self::SrgbAlpha => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::SbgrAlpha => wgpu::TextureFormat::Bgra8UnormSrgb,
            Self::RgbAlpha => wgpu::TextureFormat::Rgba8Unorm,
            Self::BgrAlpha => wgpu::TextureFormat::Bgra8Unorm,
            Self::Depth => wgpu::TextureFormat::Depth32Float,
            Self::Byte => wgpu::TextureFormat::R8Uint,
        }
    }

    pub(crate) const fn from_wgpu(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => Self::SrgbAlpha,
            wgpu::TextureFormat::Bgra8UnormSrgb => Self::SbgrAlpha,
            wgpu::TextureFormat::Rgba8Unorm => Self::RgbAlpha,
            wgpu::TextureFormat::Bgra8Unorm => Self::BgrAlpha,
            wgpu::TextureFormat::Depth32Float => Self::Depth,
            wgpu::TextureFormat::R8Uint => Self::Byte,
            _ => panic!("unsupported format"),
        }
    }
}

pub trait ColorExt {
    fn rgb_from_bytes(self, rgb: [u8; 3]) -> Rgb;
    fn rgba_from_bytes(self, rgba: [u8; 4]) -> Rgba;
}

impl ColorExt for Format {
    fn rgb_from_bytes(self, rgb: [u8; 3]) -> Rgb {
        if self.is_standard() {
            Rgb::from_standard_bytes(rgb)
        } else {
            Rgb::from_bytes(rgb)
        }
    }

    fn rgba_from_bytes(self, rgba: [u8; 4]) -> Rgba {
        if self.is_standard() {
            Rgba::from_standard_bytes(rgba)
        } else {
            Rgba::from_bytes(rgba)
        }
    }
}
