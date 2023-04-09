use {
    crate::framebuffer::{depth_frame::DepthFrame, render_frame::RenderFrame, FrameFilter},
    std::num::NonZeroU32,
    wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureFormat, TextureView},
};

pub(crate) struct Framebuffer {
    depth: DepthFrame,
    render: RenderFrame,
}

impl Framebuffer {
    pub const DEPTH_FORMAT: TextureFormat = DepthFrame::FORMAT;
    pub const RENDER_FORMAT: TextureFormat = RenderFrame::FORMAT;

    pub fn new_default(device: &Device, layout: &BindGroupLayout) -> Self {
        let one = NonZeroU32::new(1).expect("non zero");
        Self::new((one, one), FrameFilter::Nearest, device, layout)
    }

    pub fn new(
        size: (NonZeroU32, NonZeroU32),
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
