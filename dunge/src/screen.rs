use {
    crate::{
        context::PixelSize,
        framebuffer::BufferSize,
        postproc::{FrameFilter, Parameters},
        render::State,
    },
    glam::{UVec2, Vec2},
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
    /// The physical size of the frame.
    pub fn physical_size(&self) -> UVec2 {
        UVec2::new(self.width.get(), self.height.get())
    }

    /// The virtual size of the frame without antialiasing factor.
    pub fn virtual_size(&self) -> UVec2 {
        let size = self.physical_size();
        let size = match self.pixel_size {
            PixelSize::Antialiasing | PixelSize::X1 => size,
            PixelSize::X2 => size / 2,
            PixelSize::X3 => size / 3,
            PixelSize::X4 => size / 4,
        };

        size.max(UVec2::new(1, 1))
    }

    fn is_antialiasing_enabled(&self) -> bool {
        matches!(self.pixel_size, PixelSize::Antialiasing)
    }

    /// The virtual size of the frame.
    pub fn virtual_size_with_antialiasing(&self) -> UVec2 {
        let size = self.virtual_size();
        if self.is_antialiasing_enabled() {
            size * 2
        } else {
            size
        }
    }

    /// Factor of physical size relative to virtual aligned size.
    pub fn size_factor(&self) -> Vec2 {
        let aligned = self.virtual_size_with_antialiasing().as_vec2();
        let aligned = match self.pixel_size {
            PixelSize::Antialiasing => aligned / 2.,
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
            filter: FrameFilter::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RenderScreen {
    max_texture_size: u32,
    screen: Screen,
}

impl RenderScreen {
    pub fn new(state: &State) -> Self {
        Self {
            max_texture_size: state.device().limits().max_texture_dimension_2d,
            screen: Screen::default(),
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn virtual_size(&self) -> UVec2 {
        self.screen.virtual_size()
    }

    pub fn virtual_size_with_antialiasing(&self) -> UVec2 {
        self.screen.virtual_size_with_antialiasing()
    }

    /// The buffer size of the frame.
    pub fn buffer_size(&self) -> BufferSize {
        let size = self.screen.virtual_size_with_antialiasing();
        let (width, height) = size.into();

        let max = self.max_texture_size;
        if width > max {
            log::warn!("maximum screen buffer width ({max}) exceeded");
        }

        if height > max {
            log::warn!("maximum screen buffer height ({max}) exceeded");
        }

        BufferSize::new(width, height, max)
    }

    pub fn frame_parameters(&self) -> Parameters {
        Parameters {
            buffer_size: self.buffer_size(),
            factor: self.screen.size_factor(),
            filter: self.screen.filter,
            antialiasing: self.screen.is_antialiasing_enabled(),
            ..Default::default()
        }
    }
}
