use {dunge_shader::SpaceKind, std::fmt, wgpu::TextureFormat};

/// A data struct for a light space creation.
#[derive(Clone, Copy)]
pub struct SpaceData<'a>(Data<'a, (u8, u8, u8)>);

impl<'a> SpaceData<'a> {
    /// Creates a new [`SpaceData`](crate::SpaceData).
    ///
    /// # Errors
    /// Will return
    /// - [`TextureError::EmptyData`](crate::error::DataError::EmptyData)
    ///   if the data is empty.
    /// - [`TextureError::SizeDoesNotMatch`](crate::error::DataError::SizeDoesNotMatch)
    ///   if the data length doesn't match with size * number of channels.
    pub fn new(data: &'a [u8], size: (u8, u8, u8), format: Format) -> Result<Self, Error> {
        let inner = Data::new(data, size, format)?;
        Ok(Self(inner))
    }

    pub(crate) fn get(self) -> Data<'a, (u8, u8, u8)> {
        self.0
    }
}

/// A data struct for a texture creation.
#[derive(Clone, Copy)]
pub struct TextureData<'a>(Data<'a, (u32, u32)>);

impl<'a> TextureData<'a> {
    /// Creates a new [`TextureData`](crate::TextureData).
    ///
    /// # Errors
    /// Will return
    /// - [`TextureError::EmptyData`](crate::error::DataError::EmptyData)
    ///   if the data is empty.
    /// - [`TextureError::SizeDoesNotMatch`](crate::error::DataError::SizeDoesNotMatch)
    ///   if the data length doesn't match with size * number of channels.
    pub fn new(data: &'a [u8], size: (u32, u32), format: Format) -> Result<Self, Error> {
        let inner = Data::new(data, size, format)?;
        Ok(Self(inner))
    }

    pub(crate) fn get(self) -> Data<'a, (u32, u32)> {
        self.0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Data<'a, S> {
    pub data: &'a [u8],
    pub size: S,
    pub format: Format,
}

impl<'a, S> Data<'a, S> {
    fn new(data: &'a [u8], size: S, format: Format) -> Result<Self, Error>
    where
        S: Size,
    {
        if data.is_empty() {
            return Err(Error::EmptyData);
        }

        if data.len() != size.size() * format.n_channels() {
            return Err(Error::SizeDoesNotMatch);
        }

        Ok(Self { data, size, format })
    }
}

/// The data error.
#[derive(Debug)]
pub enum Error {
    /// The data is empty.
    EmptyData,

    /// The data length doesn't match with size * number of channels.
    SizeDoesNotMatch,
}

trait Size: Copy {
    fn size(self) -> usize;
}

impl Size for (u32, u32) {
    fn size(self) -> usize {
        let (width, height) = self;
        width as usize * height as usize
    }
}

impl Size for (u8, u8, u8) {
    fn size(self) -> usize {
        let (width, height, depth) = self;
        width as usize * height as usize * depth as usize
    }
}

/// The texture format.
#[derive(Clone, Copy)]
pub enum Format {
    Srgba,
    Rgba,
    Gray,
}

impl Format {
    pub(crate) const fn matches(self, kind: SpaceKind) -> bool {
        matches!(
            (self, kind),
            (Self::Srgba | Self::Rgba, SpaceKind::Rgba) | (Self::Gray, SpaceKind::Gray)
        )
    }

    pub(crate) const fn n_channels(self) -> usize {
        match self {
            Self::Srgba | Self::Rgba => 4,
            Self::Gray => 1,
        }
    }

    pub(crate) const fn texture_format(self) -> TextureFormat {
        match self {
            Self::Srgba => TextureFormat::Rgba8UnormSrgb,
            Self::Rgba => TextureFormat::Rgba8Unorm,
            Self::Gray => TextureFormat::R8Unorm,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Srgba => write!(f, "srgba"),
            Self::Rgba => write!(f, "rgba"),
            Self::Gray => write!(f, "gray"),
        }
    }
}
