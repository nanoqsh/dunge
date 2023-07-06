use {
    crate::error::TooLargeSize,
    wgpu::{Device, Queue, Sampler, Texture as WgpuTexture, TextureView},
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
    /// See [`TextureError`](crate::TextureError) for detailed info.
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

#[derive(Debug)]
pub enum Error {
    /// Returns when the data is empty.
    EmptyData,

    /// Returns when the data length doesn't match with size * number of channels.
    SizeDoesNotMatch,
}

pub(crate) struct Texture {
    texture: WgpuTexture,
    view: TextureView,
    sampler: Sampler,
}

impl Texture {
    pub fn new(data: Data, device: &Device, queue: &Queue) -> Self {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

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

        queue.write_texture(
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
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn update(&self, data: Data, queue: &Queue) -> Result<(), TooLargeSize> {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        if size != self.texture.size() {
            return Err(TooLargeSize);
        }

        queue.write_texture(
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

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
