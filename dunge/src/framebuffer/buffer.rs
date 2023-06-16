use {
    crate::framebuffer::{depth_frame::DepthFrame, render_frame::RenderFrame, FrameFilter},
    wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureFormat, TextureView},
};

pub(crate) struct Framebuffer {
    depth: DepthFrame,
    render: RenderFrame,
    params: Parameters,
}

impl Framebuffer {
    pub const DEPTH_FORMAT: TextureFormat = DepthFrame::FORMAT;
    pub const RENDER_FORMAT: TextureFormat = RenderFrame::FORMAT;

    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        const DEFAULT_SIZE: BufferSize = BufferSize::MIN;
        const DEFAULT_FILTER: FrameFilter = FrameFilter::Nearest;

        Self {
            depth: DepthFrame::new(DEFAULT_SIZE, device),
            render: RenderFrame::new(DEFAULT_SIZE, DEFAULT_FILTER, device, layout),
            params: Parameters {
                size: DEFAULT_SIZE,
                filter: DEFAULT_FILTER,
            },
        }
    }

    pub fn set_params(&mut self, params: Parameters, device: &Device, layout: &BindGroupLayout) {
        if self.params == params {
            return;
        }

        self.depth = DepthFrame::new(params.size, device);
        self.render = RenderFrame::new(params.size, params.filter, device, layout);
        self.params = params;
    }

    pub fn render_texture(&self) -> &Texture {
        self.render.texture()
    }

    pub fn render_bind_group(&self) -> &BindGroup {
        self.render.bind_group()
    }

    pub fn render_view(&self) -> &TextureView {
        self.render.view()
    }

    pub fn depth_view(&self) -> &TextureView {
        self.depth.view()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Parameters {
    pub size: BufferSize,
    pub filter: FrameFilter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct BufferSize(pub u32, pub u32);

impl BufferSize {
    pub(crate) const MIN: Self = Self(1, 1);

    pub(crate) fn new(width: u32, height: u32, max_size: u32) -> Self {
        Self(width.clamp(1, max_size), height.clamp(1, max_size))
    }
}

impl From<BufferSize> for (f32, f32) {
    fn from(BufferSize(width, height): BufferSize) -> Self {
        (width as f32, height as f32)
    }
}
