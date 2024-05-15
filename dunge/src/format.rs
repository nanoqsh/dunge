use wgpu::TextureFormat;

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
    pub fn window() -> Self {
        // TODO: variants
        Self::SrgbAlpha
    }

    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
    }

    pub(crate) const fn wgpu(self) -> TextureFormat {
        match self {
            Self::SrgbAlpha => TextureFormat::Rgba8UnormSrgb,
            Self::SbgrAlpha => TextureFormat::Bgra8UnormSrgb,
            Self::RgbAlpha => TextureFormat::Rgba8Unorm,
            Self::BgrAlpha => TextureFormat::Bgra8Unorm,
            Self::Depth => TextureFormat::Depth32Float,
            Self::Byte => TextureFormat::R8Uint,
        }
    }

    pub(crate) const fn from_wgpu(format: TextureFormat) -> Self {
        match format {
            TextureFormat::Rgba8UnormSrgb => Self::SrgbAlpha,
            TextureFormat::Bgra8UnormSrgb => Self::SbgrAlpha,
            TextureFormat::Rgba8Unorm => Self::RgbAlpha,
            TextureFormat::Bgra8Unorm => Self::BgrAlpha,
            TextureFormat::Depth32Float => Self::Depth,
            TextureFormat::R8Uint => Self::Byte,
            _ => panic!("unsupported format"),
        }
    }
}
