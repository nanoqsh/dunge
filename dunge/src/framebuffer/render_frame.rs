use {
    crate::{framebuffer::buffer::BufferSize, render::State},
    wgpu::{TextureFormat, TextureView},
};

pub(crate) struct RenderFrame {
    view: TextureView,
}

impl RenderFrame {
    pub const FORMAT: TextureFormat = if cfg!(target_os = "linux") || cfg!(target_os = "windows") {
        TextureFormat::Bgra8UnormSrgb
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    pub fn new(BufferSize(width, height): BufferSize, state: &State) -> Self {
        use wgpu::*;

        let texture = state.device().create_texture(&TextureDescriptor {
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
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());
        Self { view }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }
}
