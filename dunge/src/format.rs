use wgpu::TextureFormat;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    RgbAlpha,
    BgrAlpha,
    RgbAlphaLin,
    BgrAlphaLin,
    Depth,
}

impl Format {
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::RgbAlpha
            | Self::BgrAlpha
            | Self::RgbAlphaLin
            | Self::BgrAlphaLin
            | Self::Depth => 4,
        }
    }

    pub(crate) const fn wgpu(self) -> TextureFormat {
        match self {
            Self::RgbAlpha => TextureFormat::Rgba8UnormSrgb,
            Self::BgrAlpha => TextureFormat::Bgra8UnormSrgb,
            Self::RgbAlphaLin => TextureFormat::Rgba8Unorm,
            Self::BgrAlphaLin => TextureFormat::Bgra8Unorm,
            Self::Depth => TextureFormat::Depth32Float,
        }
    }

    pub(crate) const fn from_wgpu(format: TextureFormat) -> Self {
        match format {
            TextureFormat::Rgba8UnormSrgb => Self::RgbAlpha,
            TextureFormat::Bgra8UnormSrgb => Self::BgrAlpha,
            TextureFormat::Rgba8Unorm => Self::RgbAlphaLin,
            TextureFormat::Bgra8Unorm => Self::BgrAlphaLin,
            TextureFormat::Depth32Float => Self::Depth,
            _ => panic!("supported format"),
        }
    }
}
