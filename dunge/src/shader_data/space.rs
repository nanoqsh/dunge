use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture},
};

/// A data struct for a texture creation.
#[derive(Clone, Copy)]
#[must_use]
pub struct Data<'a> {
    data: &'a [u8],
    size: (u8, u8, u8),
}

impl<'a> Data<'a> {
    /// Creates a new [`SpaceData`](crate::SpaceData).
    ///
    /// Returns `Some` if a data is not empty and matches with a size * 4 bytes,
    /// otherwise returns `None`.
    pub const fn new(data: &'a [u8], size @ (width, height, depth): (u8, u8, u8)) -> Option<Self> {
        if data.is_empty() || data.len() != width as usize * height as usize * depth as usize * 4 {
            None
        } else {
            Some(Self { data, size })
        }
    }

    pub(crate) fn size(&self) -> (u8, u8, u8) {
        self.size
    }
}

pub(crate) struct Space {
    space_buffer: Buffer,
    texture: Texture,
    bind_group: BindGroup,
}

impl Space {
    pub fn new(
        space: SpaceModel,
        data: Data,
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> Self {
        use {
            std::num::NonZeroU32,
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                *,
            },
        };

        let space_buffer = {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("space buffer"),
                contents: space.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let (width, height, depth) = data.size;
        let size = Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: depth as u32,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
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
                bytes_per_row: NonZeroU32::new(4 * width as u32),
                rows_per_image: NonZeroU32::new(height as u32),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::SPACE_BINDING,
                    resource: space_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::SPACE_TDIFF_BINDING,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: shader::SPACE_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("space bind group"),
        });

        Self {
            space_buffer,
            texture,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(crate) struct SpaceModel {
    model: [[f32; 4]; 4],
    col: [f32; 3],
    flags: u32,
}

impl SpaceModel {
    pub fn new(model: [[f32; 4]; 4], col: [f32; 3], mono: bool) -> Self {
        Self {
            model,
            col,
            flags: mono as u32,
        }
    }
}

unsafe impl Plain for SpaceModel {}
