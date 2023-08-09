use {
    crate::{
        framebuffer::{depth_frame::DepthFrame, render_frame::RenderFrame},
        render::State,
    },
    wgpu::{TextureFormat, TextureView},
};

pub(crate) struct Framebuffer {
    depth: DepthFrame,
    render: RenderFrame,
    size: BufferSize,
}

impl Framebuffer {
    pub const DEPTH_FORMAT: TextureFormat = DepthFrame::FORMAT;
    pub const RENDER_FORMAT: TextureFormat = RenderFrame::FORMAT;

    pub fn new(state: &State) -> Self {
        const DEFAULT_SIZE: BufferSize = BufferSize::MIN;

        Self {
            depth: DepthFrame::new(DEFAULT_SIZE, state),
            render: RenderFrame::new(DEFAULT_SIZE, state),
            size: DEFAULT_SIZE,
        }
    }

    pub fn set_size(&mut self, size: BufferSize, state: &State) {
        if self.size == size {
            return;
        }

        self.depth = DepthFrame::new(size, state);
        self.render = RenderFrame::new(size, state);
        self.size = size;
    }

    pub fn render_view(&self) -> &TextureView {
        self.render.view()
    }

    pub fn depth_view(&self) -> &TextureView {
        self.depth.view()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct BufferSize(pub u32, pub u32);

impl BufferSize {
    pub(crate) const MIN: Self = Self(1, 1);

    pub(crate) fn new(width: u32, height: u32, max_size: u32) -> Self {
        Self(width.clamp(1, max_size), height.clamp(1, max_size))
    }
}

impl Default for BufferSize {
    fn default() -> Self {
        Self::MIN
    }
}

impl From<BufferSize> for (f32, f32) {
    fn from(BufferSize(width, height): BufferSize) -> Self {
        (width as f32, height as f32)
    }
}
