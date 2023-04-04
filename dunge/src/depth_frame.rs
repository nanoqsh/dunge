use wgpu::{Device, Sampler, TextureFormat, TextureView};

pub(crate) struct DepthFrame {
    view: TextureView,
    _sampler: Sampler,
}

impl DepthFrame {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth24Plus;

    pub fn new((width, height): (u32, u32), device: &Device) -> Self {
        use wgpu::*;

        let desc = TextureDescriptor {
            label: None,
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
            view_formats: &[],
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
            lod_min_clamp: 0.,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            view,
            _sampler: sampler,
        }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }
}
