use {
    crate::{
        render::State,
        shader_data::data::{Data, TextureData},
    },
    std::sync::Arc,
    wgpu::{Queue, Texture as Tx, TextureView},
};

/// The texture object.
#[derive(Clone)]
pub struct Texture(Arc<Inner>);

impl Texture {
    pub(crate) fn new(data: TextureData, state: &State) -> Self {
        let inner = Inner::new(data.get(), state);
        Self(Arc::new(inner))
    }

    pub(crate) fn view(&self) -> &TextureView {
        &self.0.view
    }

    /// Updates the texture with a new [data](`TextureData`).
    ///
    /// # Errors
    /// Will return [`InvalidSize`] if the size of the [data](`TextureData`)
    /// doesn't match the current texture size.
    pub fn update(&self, data: TextureData) -> Result<(), InvalidSize> {
        self.0.update(data.get())
    }
}

struct Inner {
    texture: Tx,
    view: TextureView,
    queue: Arc<Queue>,
}

impl Inner {
    fn new(data: Data<(u32, u32)>, state: &State) -> Self {
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
            format: data.format.texture_format(),
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
                bytes_per_row: Some(width * data.format.n_channels() as u32),
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

    fn update(&self, data: Data<(u32, u32)>) -> Result<(), InvalidSize> {
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
