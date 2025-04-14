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
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
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
