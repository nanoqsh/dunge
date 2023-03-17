use {
    crate::render_frame::FrameFilter,
    std::num::{NonZeroU32, NonZeroU8},
};

#[derive(Clone, Copy)]
pub(crate) struct Screen {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub pixel_size: NonZeroU8,
    pub filter: FrameFilter,
}

impl Screen {
    pub fn as_virtual_size(&self) -> (u32, u32) {
        let pixel_size = NonZeroU32::from(self.pixel_size);
        (
            self.width.get() / pixel_size,
            self.height.get() / pixel_size,
        )
    }
}

impl Default for Screen {
    fn default() -> Self {
        let n = 1.try_into().expect("1 is non zero");
        Self {
            width: n,
            height: n,
            pixel_size: 1.try_into().expect("1 is non zero"),
            filter: FrameFilter::Nearest,
        }
    }
}
