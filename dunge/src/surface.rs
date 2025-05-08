use {
    crate::{buffer::Format, context::Context, state::Target},
    std::{
        cell::{Cell, RefCell},
        error, fmt,
        marker::PhantomData,
        sync::Arc,
    },
};

pub trait WindowOps<W> {
    fn size(window: &W) -> (u32, u32);
}

pub struct Surface<W, O> {
    conf: RefCell<wgpu::SurfaceConfiguration>,
    inner: wgpu::Surface<'static>,
    window: Arc<W>,
    output: Cell<Option<Texture>>,
    ops: PhantomData<O>,
}

impl<W, O> Surface<W, O> {
    #[inline]
    pub fn new(cx: &Context, window: W) -> Result<Self, CreateSurfaceError>
    where
        W: wgpu::WindowHandle + 'static,
        O: WindowOps<W>,
    {
        const SUPPORTED_FORMATS: [Format; 4] = [
            Format::SrgbAlpha,
            Format::SbgrAlpha,
            Format::RgbAlpha,
            Format::BgrAlpha,
        ];

        let state = cx.state();

        let window = Arc::new(window);
        let inner = state
            .instance()
            .create_surface(Arc::clone(&window))
            .map_err(CreateSurfaceError::Surface)?;

        let conf = {
            let caps = inner.get_capabilities(state.adapter());
            let format = SUPPORTED_FORMATS.into_iter().find_map(|format| {
                let format = format.wgpu();
                caps.formats.contains(&format).then_some(format)
            });

            let Some(format) = format else {
                log::error!("surface formats: {formats:?}", formats = &caps.formats);
                return Err(CreateSurfaceError::UnsupportedFormat);
            };

            let (width, height) = O::size(&window);
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: width.max(1),
                height: height.max(1),
                present_mode: wgpu::PresentMode::default(),
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::default(),
                view_formats: vec![],
            }
        };

        inner.configure(state.device(), &conf);

        Ok(Self {
            conf: RefCell::new(conf),
            inner,
            window,
            output: Cell::new(None),
            ops: PhantomData,
        })
    }

    #[inline]
    pub fn window(&self) -> &Arc<W> {
        &self.window
    }

    #[inline]
    pub fn format(&self) -> Format {
        Format::from_wgpu(self.conf.borrow().format)
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        let conf = self.conf.borrow();
        (conf.width, conf.height)
    }

    #[inline]
    pub fn resize(&self, cx: &Context)
    where
        O: WindowOps<W>,
    {
        let (width, height) = O::size(&self.window);
        if width > 0 && height > 0 {
            // drop output before reconfigure surface
            self.output.take();

            let mut conf = self.conf.borrow_mut();
            conf.width = width;
            conf.height = height;
            self.inner.configure(cx.state().device(), &conf);
        }
    }

    #[inline]
    pub fn output(&self) -> Result<Output<'_>, SurfaceError> {
        let surface = self.inner.get_current_texture().map_err(SurfaceError)?;
        let view = {
            let desc = wgpu::TextureViewDescriptor::default();
            surface.texture.create_view(&desc)
        };

        let format = self.format();

        self.output.set(Some(Texture(surface)));

        Ok(Output {
            format,
            texture: &self.output,
            view,
        })
    }
}

impl<W, O> Drop for Surface<W, O> {
    fn drop(&mut self) {
        self.output.take();
    }
}

#[derive(Debug)]
pub enum CreateSurfaceError {
    UnsupportedFormat,
    Surface(wgpu::CreateSurfaceError),
}

impl fmt::Display for CreateSurfaceError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => f.write_str("unsupported format"),
            Self::Surface(e) => e.fmt(f),
        }
    }
}

impl error::Error for CreateSurfaceError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::UnsupportedFormat => None,
            Self::Surface(e) => Some(e),
        }
    }
}

struct Texture(wgpu::SurfaceTexture);

pub struct Output<'surface> {
    format: Format,
    texture: &'surface Cell<Option<Texture>>,
    view: wgpu::TextureView,
}

impl Output<'_> {
    #[inline]
    pub fn as_target(&self) -> Target<'_> {
        Target::new(self.format, &self.view)
    }

    #[inline]
    pub fn present(self) {
        if let Some(Texture(surface)) = self.texture.take() {
            surface.present();
        }
    }
}

pub enum Action {
    Run,
    Recreate,
    Exit,
}

#[derive(Debug)]
pub struct SurfaceError(wgpu::SurfaceError);

impl SurfaceError {
    pub fn action(&self) -> Action {
        match self.0 {
            wgpu::SurfaceError::Timeout => Action::Run,
            wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => Action::Recreate,
            wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other => Action::Exit,
        }
    }
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for SurfaceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.0)
    }
}
