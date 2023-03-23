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
    pub fn physical_size(&self) -> (u32, u32) {
        let width = self.width.get();
        let height = self.height.get();
        match self.pixel_size {
            PixelSize::XHalf | PixelSize::X1 => (width, height),
            PixelSize::X2 => (width - width % 2, height - height % 2),
            PixelSize::X3 => (width - width % 3, height - height % 3),
            PixelSize::X4 => (width - width % 4, height - height % 4),
        }
    }

    pub fn virtual_size(&self) -> (u32, u32) {
        let width = self.width.get();
        let height = self.height.get();
        match self.pixel_size {
            PixelSize::XHalf => (width * 2 + width % 2, height * 2 + height % 2),
            PixelSize::X1 => (width, height),
            PixelSize::X2 => (width / 2, height / 2),
            PixelSize::X3 => (width / 3, height / 3),
            PixelSize::X4 => (width / 4, height / 4),
        }
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
