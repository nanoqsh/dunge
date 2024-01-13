use wgpu::TextureFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    RgbAlpha,
    BgrAlpha,
}

impl Format {
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::RgbAlpha | Self::BgrAlpha => 4,
        }
    }

    pub(crate) const fn wgpu(self) -> TextureFormat {
        match self {
            Self::RgbAlpha => TextureFormat::Rgba8UnormSrgb,
            Self::BgrAlpha => TextureFormat::Bgra8UnormSrgb,
        }
    }

    pub(crate) const fn from_wgpu(format: TextureFormat) -> Option<Self> {
        match format {
            TextureFormat::Rgba8UnormSrgb => Some(Self::RgbAlpha),
            TextureFormat::Bgra8UnormSrgb => Some(Self::BgrAlpha),
            _ => None,
        }
    }
}
