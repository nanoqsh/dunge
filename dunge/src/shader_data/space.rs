use {
    crate::{
        color::{Color, Rgb},
        shader_data::{texture::Error, ModelTransform},
    },
    bytemuck::{Pod, Zeroable},
    dunge_shader::SpaceKind,
    std::fmt,
};

type Mat = [[f32; 4]; 4];

/// Parameters of the light space.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    pub data: Data<'a>,
    pub model: ModelTransform,
    pub col: Rgb,
}

impl Space<'_> {
    pub(crate) fn into_uniform(self) -> SpaceUniform {
        let Color(col) = self.col;
        SpaceUniform::new(self.data.size, self.model.into_inner(), col)
    }
}

/// A data struct for a light space creation.
#[derive(Clone, Copy)]
#[must_use]
pub struct Data<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) size: (u8, u8, u8),
    pub(crate) format: Format,
}

impl<'a> Data<'a> {
    /// Creates a new [`SpaceData`](crate::SpaceData).
    ///
    /// # Errors
    /// Will return
    /// - [`TextureError::EmptyData`](crate::error::TextureError::EmptyData)
    ///   if the data is empty.
    /// - [`TextureError::SizeDoesNotMatch`](crate::error::TextureError::SizeDoesNotMatch)
    ///   if the data length doesn't match with size * number of channels.
    pub const fn new(data: &'a [u8], size: (u8, u8, u8), format: Format) -> Result<Self, Error> {
        if data.is_empty() {
            return Err(Error::EmptyData);
        }

        let (width, height, depth) = size;
        if data.len() != width as usize * height as usize * depth as usize * format.n_channels() {
            return Err(Error::SizeDoesNotMatch);
        }

        Ok(Self { data, size, format })
    }
}

/// The light space data format.
#[derive(Clone, Copy)]
pub enum Format {
    Srgba,
    Rgba,
    Gray,
}

impl Format {
    pub(crate) fn matches(self, kind: SpaceKind) -> bool {
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct SpaceUniform {
    model: Mat,
    col: [f32; 3],
    pad: u32,
}

impl SpaceUniform {
    pub fn new(size: (u8, u8, u8), transform: Mat, col: [f32; 3]) -> Self {
        Self {
            model: {
                use glam::{Mat4, Quat, Vec3};

                let (width, height, depth) = size;
                let texture_space = Mat4::from_scale_rotation_translation(
                    Vec3::new(1. / width as f32, 1. / depth as f32, 1. / height as f32),
                    Quat::IDENTITY,
                    Vec3::new(0.5, 0.5, 0.5),
                );

                let model = Mat4::from_cols_array_2d(&transform);
                let model = texture_space * model;
                model.to_cols_array_2d()
            },
            col,
            pad: 0,
        }
    }
}
