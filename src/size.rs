use {
    crate::texture::FrameFilter,
    std::num::{NonZeroU32, NonZeroU8},
};

#[derive(Clone, Copy)]
pub(crate) struct Size {
    pub(crate) width: NonZeroU32,
    pub(crate) height: NonZeroU32,
    pub(crate) pixel_size: NonZeroU8,
    pub(crate) filter: FrameFilter,
}

impl Size {
    pub fn as_physical(&self) -> (f32, f32) {
        (self.width.get() as _, self.height.get() as _)
    }

    pub fn as_virtual(&self) -> (u32, u32) {
        let pixel_size = NonZeroU32::from(self.pixel_size);
        (
            self.width.get() / pixel_size,
            self.height.get() / pixel_size,
        )
    }
}

impl Default for Size {
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
