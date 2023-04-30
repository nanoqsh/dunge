use {
    crate::{layout::Plain, shader, texture::Error, transform::IntoMat},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture},
};

/// Parameters of a light space.
pub struct Space<'a, M> {
    pub data: Data<'a>,
    pub transform: M,
    pub col: [f32; 3],
    pub mono: bool,
}

impl<'a, M> Space<'a, M> {
    pub(crate) fn into_mat(self) -> Space<'a, [[f32; 4]; 4]>
    where
        M: IntoMat,
    {
        Space {
            data: self.data,
            transform: self.transform.into_mat(),
            col: self.col,
            mono: self.mono,
        }
    }
}

/// A data struct for a light space creation.
#[derive(Clone, Copy)]
#[must_use]
pub struct Data<'a> {
    data: &'a [u8],
    size: (u8, u8, u8),
}

impl<'a> Data<'a> {
    /// Creates a new [`SpaceData`](crate::SpaceData).
    ///
    /// # Errors
    /// See [`TextureError`](crate::TextureError) for detailed info.
    pub const fn new(data: &'a [u8], size: (u8, u8, u8)) -> Result<Self, Error> {
        if data.is_empty() {
            return Err(Error::EmptyData);
        }

        let (width, height, depth) = size;
        if data.len() != width as usize * height as usize * depth as usize * 4 {
            return Err(Error::SizeDoesNotMatch);
        }

        Ok(Self { data, size })
    }
}

pub(crate) struct LightSpace {
    space_buffer: Buffer,
    texture: Texture,
    bind_group: BindGroup,
}

impl LightSpace {
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
    pub fn new(space: &Space<[[f32; 4]; 4]>) -> Self {
        use glam::{Mat4, Quat, Vec3};

        let (width, height, depth) = space.data.size;
        let texture_space = Mat4::from_scale_rotation_translation(
            Vec3::new(1. / width as f32, 1. / depth as f32, 1. / height as f32),
            Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        );

        let model = Mat4::from_cols_array_2d(&space.transform.into_mat());
        let model = texture_space * model;

        Self {
            model: model.to_cols_array_2d(),
            col: space.col,
            flags: space.mono as u32,
        }
    }
}

unsafe impl Plain for SpaceModel {}
