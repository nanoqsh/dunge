use {
    crate::{framebuffer::buffer::BufferSize, postproc::PostProcessor},
    wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureFormat, TextureView},
};

/// Describes a frame render filter mode.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum FrameFilter {
    #[default]
    Nearest,
    Linear,
}

pub(crate) struct RenderFrame {
    bind_group: BindGroup,
    texture: Texture,
    view: TextureView,
}

impl RenderFrame {
    pub const FORMAT: TextureFormat = if cfg!(target_os = "linux") || cfg!(target_os = "windows") {
        TextureFormat::Bgra8UnormSrgb
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    pub fn new(
        BufferSize(width, height): BufferSize,
        filter: FrameFilter,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        use wgpu::*;

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
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
            label: None,
            layout,
            entries: &[
                BindGroupEntry {
                    binding: PostProcessor::TEXTURE_TDIFF_BINDING,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: PostProcessor::TEXTURE_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            bind_group,
            texture,
            view,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
