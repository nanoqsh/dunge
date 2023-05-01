use {
    crate::{layout::Plain, r#loop::Error, shader, shader_data::TextureError, transform::IntoMat},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture, TextureView},
};

/// Parameters of a light space.
pub struct Space<'a, M = [[f32; 4]; 4]> {
    pub data: Data<'a>,
    pub transform: M,
    pub col: [f32; 3],
    pub mono: bool,
}

impl<'a, M> Space<'a, M> {
    pub(crate) fn into_mat(self) -> Space<'a>
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
    pub const fn new(data: &'a [u8], size: (u8, u8, u8)) -> Result<Self, TextureError> {
        if data.is_empty() {
            return Err(TextureError::EmptyData);
        }

        let (width, height, depth) = size;
        if data.len() != width as usize * height as usize * depth as usize * 4 {
            return Err(TextureError::SizeDoesNotMatch);
        }

        Ok(Self { data, size })
    }
}

pub(crate) struct LightSpace {
    space_buffer: Buffer,
    n_spaces: usize,
    textures: Box<[Texture]>,
    bind_group: BindGroup,
}

impl LightSpace {
    pub fn new(
        spaces: &[SpaceModel],
        data: &[Data],
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> Result<Self, Error> {
        use {
            once_cell::sync::OnceCell,
            std::{array, num::NonZeroU32},
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                *,
            },
        };

        static FAKE_VIEW: OnceCell<&TextureView> = OnceCell::new();

        debug_assert_eq!(
            spaces.len(),
            data.len(),
            "spaces and data lengths must be equal",
        );

        if spaces.len() > shader::MAX_N_SPACES as usize {
            use crate::r#loop::Error;

            return Err(Error::TooManySpaces);
        }

        let space_buffer = {
            let mut buf = [SpaceModel::default(); shader::MAX_N_SPACES as usize];
            buf[..spaces.len()].copy_from_slice(spaces);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("space buffer"),
                contents: buf.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let n_spaces_buffer = {
            let len = spaces.len() as u32;
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("n spaces buffer"),
                contents: len.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let init_view = || -> &TextureView {
            const DATA: [u8; 4] = [0; 4];
            const SIZE: Extent3d = Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&TextureDescriptor {
                label: None,
                size: SIZE,
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
                &DATA,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4),
                    rows_per_image: NonZeroU32::new(1),
                },
                SIZE,
            );

            let view = texture.create_view(&TextureViewDescriptor::default());
            Box::leak(view.into())
        };

        let mut views: [View; shader::MAX_N_SPACES as usize] =
            array::from_fn(|_| View::Fake(FAKE_VIEW.get_or_init(init_view)));

        let mut textures = Vec::with_capacity(data.len());
        for (dt, vo) in data.iter().zip(&mut views) {
            let (width, height, depth) = dt.size;
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
                dt.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * width as u32),
                    rows_per_image: NonZeroU32::new(height as u32),
                },
                size,
            );

            *vo = View::Faithful(texture.create_view(&TextureViewDescriptor::default()));
            textures.push(texture);
        }

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
                    binding: shader::SPACES_BINDING,
                    resource: space_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::N_SPACES_BINDING,
                    resource: n_spaces_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::SPACE0_TDIFF_BINDING,
                    resource: BindingResource::TextureView(views[0].as_ref()),
                },
                BindGroupEntry {
                    binding: shader::SPACE1_TDIFF_BINDING,
                    resource: BindingResource::TextureView(views[1].as_ref()),
                },
                BindGroupEntry {
                    binding: shader::SPACE2_TDIFF_BINDING,
                    resource: BindingResource::TextureView(views[2].as_ref()),
                },
                BindGroupEntry {
                    binding: shader::SPACE3_TDIFF_BINDING,
                    resource: BindingResource::TextureView(views[3].as_ref()),
                },
                BindGroupEntry {
                    binding: shader::SPACE_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("space bind group"),
        });

        Ok(Self {
            space_buffer,
            n_spaces: spaces.len(),
            textures: textures.into_boxed_slice(),
            bind_group,
        })
    }

    pub fn update_spaces(&self, spaces: &[SpaceModel], data: &[Data], queue: &Queue) -> bool {
        use {std::num::NonZeroU32, wgpu::*};

        if spaces.is_empty() || self.n_spaces != spaces.len() {
            return false;
        }

        queue.write_buffer(&self.space_buffer, 0, spaces.as_bytes());

        for (dt, texture) in data.iter().zip(&self.textures[..]) {
            let (width, height, depth) = dt.size;
            let size = Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: depth as u32,
            };

            queue.write_texture(
                ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                dt.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * width as u32),
                    rows_per_image: NonZeroU32::new(height as u32),
                },
                size,
            );
        }

        true
    }

    pub fn update_nth_space(
        &self,
        n: usize,
        space: SpaceModel,
        queue: &Queue,
    ) -> Result<(), Error> {
        use std::mem;

        if n >= self.n_spaces {
            return Err(Error::NotFound);
        }

        queue.write_buffer(
            &self.space_buffer,
            (mem::size_of::<SpaceModel>() * n) as _,
            space.as_bytes(),
        );

        Ok(())
    }

    pub fn update_nth_color(&self, n: usize, col: [f32; 3], queue: &Queue) -> Result<(), Error> {
        use std::mem;

        const COL_OFFSET: usize = mem::size_of::<[[f32; 4]; 4]>();

        if n >= self.n_spaces {
            return Err(Error::NotFound);
        }

        queue.write_buffer(
            &self.space_buffer,
            (mem::size_of::<SpaceModel>() * n + COL_OFFSET) as _,
            col.as_bytes(),
        );

        Ok(())
    }

    pub fn update_nth_data(&self, n: usize, data: Data, queue: &Queue) -> Result<(), Error> {
        use {std::num::NonZeroU32, wgpu::*};

        if n >= self.n_spaces {
            use crate::r#loop::Error;

            return Err(Error::NotFound);
        }

        let (width, height, depth) = data.size;
        let size = Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: depth as u32,
        };

        queue.write_texture(
            ImageCopyTexture {
                texture: &self.textures[n],
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

        Ok(())
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

enum View {
    Fake(&'static TextureView),
    Faithful(TextureView),
}

impl View {
    fn as_ref(&self) -> &TextureView {
        match self {
            Self::Fake(v) => v,
            Self::Faithful(v) => v,
        }
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
    pub fn new(space: &Space) -> Self {
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
