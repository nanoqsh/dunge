use {
    crate::{context::PixelSize, render_frame::FrameFilter},
    std::num::NonZeroU32,
};

#[derive(Clone, Copy)]
pub(crate) struct Screen {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub pixel_size: PixelSize,
    pub filter: FrameFilter,
}

impl Screen {
    pub const MAX_SIZE: u32 = 8192;

    pub fn physical_size(&self) -> (u32, u32) {
        (self.width.get(), self.height.get())
    }

    pub fn virtual_size(&self) -> (u32, u32) {
        let (width, height) = self.physical_size();
        match self.pixel_size {
            PixelSize::XHalf => (width * 2, height * 2),
            PixelSize::X1 => (width, height),
            PixelSize::X2 => (width / 2, height / 2),
            PixelSize::X3 => (width / 3, height / 3),
            PixelSize::X4 => (width / 4, height / 4),
        }
    }

    pub fn virtual_size_aligned(&self) -> (u32, u32) {
        const N_COLOR_CHANNELS: u32 = 4;
        const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / N_COLOR_CHANNELS;

        let (mut width, height) = self.virtual_size();
        width += ALIGNMENT - width % ALIGNMENT;
        (width, height)
    }

    pub fn buffer_size(&self) -> (u32, u32) {
        let (width, height) = self.virtual_size_aligned();
        let (width, height) = match self.pixel_size {
            PixelSize::XHalf => (width + 1, height + 1),
            _ => (width, height),
        };

        (width.min(Self::MAX_SIZE), height.min(Self::MAX_SIZE))
    }

    pub fn size_factor(&self) -> (f32, f32) {
        let (width, height) = self.virtual_size_aligned();
        let (width, height) = match self.pixel_size {
            PixelSize::XHalf => (width / 2, height / 2),
            PixelSize::X1 => (width, height),
            PixelSize::X2 => (width * 2, height * 2),
            PixelSize::X3 => (width * 3, height * 3),
            PixelSize::X4 => (width * 4, height * 4),
        };

        let (pw, ph) = self.physical_size();
        (pw as f32 / width as f32, ph as f32 / height as f32)
    }
}

impl Default for Screen {
    fn default() -> Self {
        let n = 1.try_into().expect("1 is non zero");
        Self {
            width: n,
            height: n,
            pixel_size: PixelSize::default(),
            filter: FrameFilter::default(),
        }
    }
}
