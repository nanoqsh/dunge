use std::num::NonZeroU32;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub pixel_size: NonZeroU32,
}

impl Size {
    pub fn as_f32(&self) -> (f32, f32) {
        (self.width.get() as f32, self.height.get() as f32)
    }

    pub fn pixeled(&self) -> (u32, u32) {
        (
            self.width.get() / self.pixel_size,
            self.height.get() / self.pixel_size,
        )
    }
}

impl Default for Size {
    fn default() -> Self {
        let n = NonZeroU32::new(1).expect("1 is non zero");
        Self {
            width: n,
            height: n,
            pixel_size: n,
        }
    }
}
