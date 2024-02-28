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
}

impl Format {
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
        }
    }

    pub(crate) const fn wgpu(self) -> TextureFormat {
        match self {
            Self::SrgbAlpha => TextureFormat::Rgba8UnormSrgb,
            Self::SbgrAlpha => TextureFormat::Bgra8UnormSrgb,
            Self::RgbAlpha => TextureFormat::Rgba8Unorm,
            Self::BgrAlpha => TextureFormat::Bgra8Unorm,
            Self::Depth => TextureFormat::Depth32Float,
        }
    }

    pub(crate) const fn from_wgpu(format: TextureFormat) -> Self {
        match format {
            TextureFormat::Rgba8UnormSrgb => Self::SrgbAlpha,
            TextureFormat::Bgra8UnormSrgb => Self::SbgrAlpha,
            TextureFormat::Rgba8Unorm => Self::RgbAlpha,
            TextureFormat::Bgra8Unorm => Self::BgrAlpha,
            TextureFormat::Depth32Float => Self::Depth,
            _ => panic!("unsupported format"),
        }
    }
}
