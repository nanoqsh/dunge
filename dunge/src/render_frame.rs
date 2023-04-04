use {
    crate::shader,
    wgpu::{BindGroup, BindGroupLayout, Device, TextureFormat, TextureView},
};

/// Describes a frame render filter mode.
#[derive(Clone, Copy, Default)]
pub enum FrameFilter {
    #[default]
    Nearest,
    Linear,
}

pub(crate) struct RenderFrame {
    view: TextureView,
    bind_group: BindGroup,
}

impl RenderFrame {
    pub const RENDER_FORMAT: TextureFormat = if cfg!(target_arch = "wasm32") {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Bgra8UnormSrgb
    };

    pub fn new(
        (width, height): (u32, u32),
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
            format: Self::RENDER_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST
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
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::POST_TDIFF_BINDING,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: shader::POST_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture bind group"),
        });

        Self { view, bind_group }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
