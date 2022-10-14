use {
    crate::shader_consts,
    std::num::NonZeroU32,
    wgpu::{
        BindGroup, BindGroupLayout, Device, Queue, Sampler, Texture as WgpuTexture, TextureFormat,
        TextureView,
    },
};

/// A data struct for a texture creation.
#[derive(Clone, Copy)]
pub struct TextureData<'a> {
    data: &'a [u8],
    size: (u32, u32),
}

impl<'a> TextureData<'a> {
    /// Creates a new `TextureData`.
    ///
    /// Returns `Some` if a data matches with a size * 4 bytes,
    /// otherwise returns `None`.
    pub fn new(data: &'a [u8], size @ (width, height): (u32, u32)) -> Option<Self> {
        if data.len() == width as usize * height as usize * 4 {
            Some(Self { data, size })
        } else {
            None
        }
    }
}

pub(crate) struct Texture {
    texture: WgpuTexture,
    bind_group: BindGroup,
}

impl Texture {
    pub(crate) fn new(
        data: TextureData,
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> Self {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("texture"),
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
                    binding: shader_consts::textured::T_DIFFUSE.binding,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: shader_consts::textured::S_DIFFUSE.binding,
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

    pub(crate) fn update_data(&mut self, data: TextureData, queue: &Queue) {
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

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub(crate) struct DepthFrame {
    view: TextureView,
    _sampler: Sampler,
}

impl DepthFrame {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub(crate) fn new((width, height): (u32, u32), device: &Device) -> Self {
        use wgpu::*;

        let desc = TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            view,
            _sampler: sampler,
        }
    }

    pub(crate) fn view(&self) -> &TextureView {
        &self.view
    }
}

pub(crate) struct RenderFrame {
    view: TextureView,
    bind_group: BindGroup,
}

impl RenderFrame {
    pub(crate) fn new(
        (width, height): (u32, u32),
        filter: FrameFilter,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        use wgpu::*;

        let texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING,
            label: Some("texture"),
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let filter_mode = match filter {
            FrameFilter::Nearest => FilterMode::Nearest,
            FrameFilter::Linear => FilterMode::Linear,
        };

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader_consts::post::T_DIFFUSE.binding,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: shader_consts::post::S_DIFFUSE.binding,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture bind group"),
        });

        Self { view, bind_group }
    }

    pub(crate) fn view(&self) -> &TextureView {
        &self.view
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[derive(Clone, Copy)]
pub enum FrameFilter {
    Nearest,
    Linear,
}
