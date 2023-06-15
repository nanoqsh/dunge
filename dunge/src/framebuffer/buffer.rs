use {
    crate::{
        framebuffer::{depth_frame::DepthFrame, render_frame::RenderFrame, FrameFilter},
        screen::BufferSize,
    },
    wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureFormat, TextureView},
};

pub(crate) struct Framebuffer {
    depth: DepthFrame,
    render: RenderFrame,
}

impl Framebuffer {
    pub const DEPTH_FORMAT: TextureFormat = DepthFrame::FORMAT;
    pub const RENDER_FORMAT: TextureFormat = RenderFrame::FORMAT;

    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        Self::with_size_and_filter(BufferSize::MIN, FrameFilter::Nearest, device, layout)
    }

    pub fn with_size_and_filter(
        size: BufferSize,
        filter: FrameFilter,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        Self {
            depth: DepthFrame::new(size, device),
            render: RenderFrame::new(size, filter, device, layout),
        }
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
