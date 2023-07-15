use {
    crate::render::State,
    std::sync::Arc,
    wgpu::{Queue, Texture as Tx, TextureView},
};

/// A data struct for a texture creation.
#[derive(Clone, Copy)]
#[must_use]
pub struct Data<'a> {
    data: &'a [u8],
    size: (u32, u32),
}

impl<'a> Data<'a> {
    /// Creates a new [`TextureData`](crate::TextureData).
    ///
    /// # Errors
    /// Will return
    /// - [`TextureError::EmptyData`](crate::error::TextureError::EmptyData)
    ///   if the data is empty.
    /// - [`TextureError::SizeDoesNotMatch`](crate::error::TextureError::SizeDoesNotMatch)
    ///   if the data length doesn't match with size * number of channels.
    pub const fn new(data: &'a [u8], size: (u32, u32)) -> Result<Self, Error> {
        if data.is_empty() {
            return Err(Error::EmptyData);
        }

        let (width, height) = size;
        if data.len() != width as usize * height as usize * 4 {
            return Err(Error::SizeDoesNotMatch);
        }

        Ok(Self { data, size })
    }
}

/// Texture error.
#[derive(Debug)]
pub enum Error {
    /// The data is empty.
    EmptyData,

    /// The data length doesn't match with size * number of channels.
    SizeDoesNotMatch,
}

/// The texture object.
#[derive(Clone)]
pub struct Texture(Arc<Inner>);

impl Texture {
    pub(crate) fn new(data: Data, state: &State) -> Self {
        let inner = Inner::new(data, state);
        Self(Arc::new(inner))
    }

    pub(crate) fn view(&self) -> &TextureView {
        &self.0.view
    }

    /// Updates the texture with a new [data](`Data`).
    ///
    /// # Errors
    /// Will return [`InvalidSize`] if the size of the [data](`Data`)
    /// doesn't match the current texture size.
    pub fn update(&self, data: Data) -> Result<(), InvalidSize> {
        self.0.update(data)
    }
}

struct Inner {
    texture: Tx,
    view: TextureView,
    queue: Arc<Queue>,
}

impl Inner {
    fn new(data: Data, state: &State) -> Self {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let device = state.device();
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        state.queue().write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        Self {
            texture,
            view,
            queue: Arc::clone(state.queue()),
        }
    }

    fn update(&self, data: Data) -> Result<(), InvalidSize> {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        if size != self.texture.size() {
            return Err(InvalidSize);
        }

        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        Ok(())
    }
}

/// An error returned from the texture updation.
#[derive(Debug)]
pub struct InvalidSize;
