use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture},
};

pub(crate) struct Space {
    space_buffer: Buffer,
    texture: Texture,
    bind_group: BindGroup,
}

impl Space {
    pub fn new(
        space: SpaceModel,
        texture_data: &[u8],
        texture_size: (u8, u8, u8),
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

        let (width, height, depth) = texture_size;
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
            texture_data,
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
    loc: [[f32; 4]; 4],
    col: [f32; 3],
    flags: u32,
}

impl SpaceModel {
    pub fn new(loc: [[f32; 4]; 4], col: [f32; 3], mono: bool) -> Self {
        Self {
            loc,
            col,
            flags: mono as u32,
        }
    }
}

unsafe impl Plain for SpaceModel {}
