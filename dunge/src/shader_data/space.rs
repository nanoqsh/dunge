use {
    crate::{
        color::{IntoLinear, Linear},
        error::{Error, SpaceNotFound, TooLargeSize, TooManySpaces},
        layout::Plain,
        shader,
        shader_data::TextureError,
        transform::IntoMat,
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture, TextureView},
};

type Mat = [[f32; 4]; 4];

/// Parameters of the light space.
#[derive(Clone, Copy)]
pub struct Space<'a, M = Mat, C = Linear<f32, 3>> {
    pub data: Data<'a>,
    pub transform: M,
    pub col: C,
}

impl<'a, M, C> Space<'a, M, C> {
    pub(crate) fn into_mat_and_linear(self) -> Space<'a>
    where
        M: IntoMat,
        C: IntoLinear<3>,
    {
        Space {
            data: self.data,
            transform: self.transform.into_mat(),
            col: self.col.into_linear(),
        }
    }
}

/// A data struct for a light space creation.
#[derive(Clone, Copy)]
#[must_use]
pub struct Data<'a> {
    data: &'a [u8],
    size: (u8, u8, u8),
    format: Format,
}

impl<'a> Data<'a> {
    /// Creates a new [`SpaceData`](crate::SpaceData).
    ///
    /// # Errors
    /// See [`TextureError`](crate::TextureError) for detailed info.
    pub const fn new(
        data: &'a [u8],
        size: (u8, u8, u8),
        format: Format,
    ) -> Result<Self, TextureError> {
        if data.is_empty() {
            return Err(TextureError::EmptyData);
        }

        let (width, height, depth) = size;
        if data.len() != width as usize * height as usize * depth as usize * format.n_channels() {
            return Err(TextureError::SizeDoesNotMatch);
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
    const fn n_channels(self) -> usize {
        match self {
            Self::Srgba | Self::Rgba => 4,
            Self::Gray => 1,
        }
    }
}

pub(crate) struct LightSpace {
    space_buffer: Buffer,
    n_spaces: usize,
    textures: Box<[SpaceTexture]>,
    bind_group: BindGroup,
}

impl LightSpace {
    pub fn new(
        spaces: &[SpaceModel],
        data: &[Data],
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> Result<Self, TooManySpaces> {
        use {
            once_cell::sync::OnceCell,
            std::array,
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
            return Err(TooManySpaces);
        }

        let space_buffer = {
            let buf = Spaces::from_slice(spaces);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("space buffer"),
                contents: buf.as_bytes(),
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
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
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
                format: match dt.format {
                    Format::Srgba => TextureFormat::Rgba8UnormSrgb,
                    Format::Rgba => TextureFormat::Rgba8Unorm,
                    Format::Gray => TextureFormat::R8Unorm,
                },
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
                    bytes_per_row: Some(dt.format.n_channels() as u32 * width as u32),
                    rows_per_image: Some(height as u32),
                },
                size,
            );

            *vo = View::Faithful(texture.create_view(&TextureViewDescriptor::default()));

            textures.push(SpaceTexture {
                texture,
                buf_size: dt.size,
            });
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

    pub fn update_spaces(
        &mut self,
        spaces: &[SpaceModel],
        data: &[Data],
        queue: &Queue,
    ) -> Result<(), Error> {
        use std::mem;

        debug_assert_eq!(
            spaces.len(),
            data.len(),
            "spaces and data lengths must be equal",
        );

        if spaces.len() > shader::MAX_N_SPACES as usize {
            return Err(TooManySpaces.into());
        }

        if !spaces.is_empty() {
            queue.write_buffer(&self.space_buffer, 0, spaces.as_bytes());
        }

        if self.n_spaces != spaces.len() {
            let len = spaces.len() as u32;
            queue.write_buffer(
                &self.space_buffer,
                mem::size_of::<[SpaceModel; 4]>() as _,
                len.as_bytes(),
            );

            self.n_spaces = spaces.len();
        }

        for (n, &dt) in (0..).zip(data) {
            self.update_nth_data(n, dt, queue)?;
        }

        Ok(())
    }

    pub fn update_nth_space(
        &self,
        n: usize,
        space: SpaceModel,
        queue: &Queue,
    ) -> Result<(), SpaceNotFound> {
        use std::mem;

        if n >= self.n_spaces {
            return Err(SpaceNotFound);
        }

        queue.write_buffer(
            &self.space_buffer,
            (mem::size_of::<SpaceModel>() * n) as _,
            space.as_bytes(),
        );

        Ok(())
    }

    pub fn update_nth_color(
        &self,
        n: usize,
        col: [f32; 3],
        queue: &Queue,
    ) -> Result<(), SpaceNotFound> {
        use std::mem;

        const COL_OFFSET: usize = mem::size_of::<Mat>();

        if n >= self.n_spaces {
            return Err(SpaceNotFound);
        }

        queue.write_buffer(
            &self.space_buffer,
            (mem::size_of::<SpaceModel>() * n + COL_OFFSET) as _,
            col.as_bytes(),
        );

        Ok(())
    }

    pub fn update_nth_data(&self, n: usize, data: Data, queue: &Queue) -> Result<(), Error> {
        use wgpu::*;

        let Some(texture) = self.textures.get(n) else {
            return Err(SpaceNotFound.into());
        };

        let (width, height, depth) = data.size;
        let (buf_width, buf_height, buf_depth) = texture.buf_size;
        if width > buf_width || height > buf_height || depth > buf_depth {
            return Err(TooLargeSize.into());
        }

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(data.format.n_channels() as u32 * width as u32),
                rows_per_image: Some(height as u32),
            },
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: depth as u32,
            },
        );

        Ok(())
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

struct SpaceTexture {
    texture: Texture,
    buf_size: (u8, u8, u8),
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
    model: Mat,
    col: [f32; 3],
    flags: u32,
}

impl SpaceModel {
    pub fn new(space: &Space) -> Self {
        Self {
            model: {
                use glam::{Mat4, Quat, Vec3};

                let texture_space = {
                    let (width, height, depth) = space.data.size;
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(1. / width as f32, 1. / depth as f32, 1. / height as f32),
                        Quat::IDENTITY,
                        Vec3::new(0.5, 0.5, 0.5),
                    )
                };

                let model = Mat4::from_cols_array_2d(&space.transform);
                let model = texture_space * model;

                model.to_cols_array_2d()
            },
            col: space.col.0,
            flags: match space.data.format {
                Format::Srgba | Format::Rgba => 0,
                Format::Gray => 1,
            },
        }
    }
}

unsafe impl Plain for SpaceModel {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(crate) struct Spaces {
    data: [SpaceModel; 4],
    len: u32,
    pad: [u32; 3],
}

impl Spaces {
    fn from_slice(slice: &[SpaceModel]) -> Self {
        let mut spaces = Self::default();
        spaces.data[..slice.len()].copy_from_slice(slice);
        spaces.len = slice.len() as u32;
        spaces
    }
}

unsafe impl Plain for Spaces {}
