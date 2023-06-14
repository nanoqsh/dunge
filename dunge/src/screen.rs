use {
    crate::{context::PixelSize, framebuffer::FrameFilter},
    glam::{UVec2, Vec2},
    std::num::NonZeroU32,
};

#[derive(Clone, Copy)]
pub(crate) struct Screen {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub pixel_size: PixelSize,
    pub antialiasing: bool,
    pub filter: FrameFilter,
}

impl Screen {
    pub fn physical_size(&self) -> UVec2 {
        UVec2::new(self.width.get(), self.height.get())
    }

    pub fn virtual_size(&self) -> UVec2 {
        let size = self.physical_size();
        match self.pixel_size {
            PixelSize::XHalf => size * 2,
            PixelSize::X1 => size,
            PixelSize::X2 => size / 2,
            PixelSize::X3 => size / 3,
            PixelSize::X4 => size / 4,
        }
    }

    pub fn virtual_size_with_antialiasing(&self) -> UVec2 {
        let size = self.virtual_size();
        if self.antialiasing {
            size * 2
        } else {
            size
        }
    }

    pub fn virtual_size_aligned(&self) -> UVec2 {
        use wgpu::util;

        const N_COLOR_CHANNELS: u32 = 4;
        const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / N_COLOR_CHANNELS;

        let mut size = self.virtual_size();
        size.x = util::align_to(size.x, ALIGNMENT);
        size
    }

    pub fn buffer_size(&self, max_size: u32) -> (NonZeroU32, NonZeroU32) {
        let size = self.virtual_size_aligned();
        let size = if self.antialiasing { size * 2 } else { size };
        let (width, height) = size.into();

        (
            NonZeroU32::new(width.clamp(1, max_size)).expect("non zero"),
            NonZeroU32::new(height.clamp(1, max_size)).expect("non zero"),
        )
    }

    pub fn size_factor(&self) -> Vec2 {
        let aligned = self.virtual_size_aligned();
        let aligned = match self.pixel_size {
            PixelSize::XHalf => aligned / 2,
            PixelSize::X1 => aligned,
            PixelSize::X2 => aligned * 2,
            PixelSize::X3 => aligned * 3,
            PixelSize::X4 => aligned * 4,
        };

        let physical = self.physical_size();
        physical.as_vec2() / aligned.as_vec2()
    }
}

impl Default for Screen {
    fn default() -> Self {
        let n = 1.try_into().expect("1 is non zero");
        Self {
            width: n,
            height: n,
            pixel_size: PixelSize::default(),
            antialiasing: false,
            filter: FrameFilter::default(),
        }
    }
}
