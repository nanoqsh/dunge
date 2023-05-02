use {
    crate::shader,
    std::num::NonZeroU32,
    wgpu::{BindGroup, BindGroupLayout, Device, Queue, Texture as WgpuTexture},
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

    /// Returns when the data length doesn't match with size * number of channels,
    SizeDoesNotMatch,
}

pub(crate) struct Texture {
    texture: WgpuTexture,
    bind_group: BindGroup,
}

impl Texture {
    pub fn new(data: Data, device: &Device, queue: &Queue, layout: &BindGroupLayout) -> Self {
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
                bytes_per_row: NonZeroU32::new(4 * width),
                rows_per_image: NonZeroU32::new(height),
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

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::TDIFF_BINDING,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: shader::SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture bind group"),
        });

        Self {
            texture,
            bind_group,
        }
    }

    pub fn update_data(&mut self, data: Data, queue: &Queue) {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

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
                bytes_per_row: NonZeroU32::new(4 * width),
                rows_per_image: NonZeroU32::new(height),
            },
            size,
        );
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
