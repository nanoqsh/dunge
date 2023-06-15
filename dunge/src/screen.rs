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
    /// The physical size of the frame.
    pub fn physical_size(&self) -> UVec2 {
        UVec2::new(self.width.get(), self.height.get())
    }

    /// The virtual size of the frame without antialiasing factor.
    pub fn virtual_size(&self) -> UVec2 {
        let size = self.physical_size();
        match self.pixel_size {
            PixelSize::X1 => size,
            PixelSize::X2 => size / 2,
            PixelSize::X3 => size / 3,
            PixelSize::X4 => size / 4,
        }
    }

    /// The virtual size of the frame.
    pub fn virtual_size_with_antialiasing(&self) -> UVec2 {
        let size = self.virtual_size();
        if self.antialiasing {
            size * 2
        } else {
            size
        }
    }

    /// The virtual size of the frame without antialiasing factor, but aligned width.
    fn virtual_size_aligned(&self) -> UVec2 {
        use wgpu::util;

        const N_COLOR_CHANNELS: u32 = 4;
        const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / N_COLOR_CHANNELS;

        let mut size = self.virtual_size_with_antialiasing();
        size.x = util::align_to(size.x, ALIGNMENT);
        size
    }

    /// Factor of physical size relative to virtual aligned size.
    pub fn size_factor(&self) -> Vec2 {
        let aligned = self.virtual_size_aligned().as_vec2();
        let aligned = match self.pixel_size {
            PixelSize::X1 if self.antialiasing => aligned / 2.,
            PixelSize::X1 => aligned,
            PixelSize::X2 => aligned * 2.,
            PixelSize::X3 => aligned * 3.,
            PixelSize::X4 => aligned * 4.,
        };

        let physical = self.physical_size().as_vec2();
        physical / aligned
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

pub(crate) struct RenderScreen {
    screen: Screen,
    max_texture_size: u32,
}

impl RenderScreen {
    pub fn new(max_texture_size: u32) -> Self {
        Self {
            screen: Screen::default(),
            max_texture_size,
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// The buffer size of the frame.
    pub fn buffer_size(&self) -> BufferSize {
        let size = self.screen.virtual_size_aligned();
        let (width, height) = size.into();
        BufferSize::new(width, height, self.max_texture_size)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BufferSize(pub u32, pub u32);

impl BufferSize {
    pub(crate) const MIN: Self = Self(1, 1);

    fn new(width: u32, height: u32, max_size: u32) -> Self {
        Self(width.clamp(1, max_size), height.clamp(1, max_size))
    }
}

impl From<BufferSize> for (f32, f32) {
    fn from(BufferSize(width, height): BufferSize) -> Self {
        (width as f32, height as f32)
    }
}
