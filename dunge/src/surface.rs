use {
    crate::{AsTarget, Context, Target, prelude::Format},
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
    out: Cell<bool>,
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
            out: Cell::new(false),
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
        assert!(!self.out.get(), "cannot resize while present to output");

        let (width, height) = O::size(&self.window);
        if width > 0 && height > 0 {
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

        self.out.set(true);

        Ok(Output {
            out: &self.out,
            surface,
            view,
            format,
        })
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

pub struct Output<'surface> {
    out: &'surface Cell<bool>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    format: Format,
}

impl Output<'_> {
    #[inline]
    pub fn present(self) {
        self.surface.present();
        self.out.set(false);
    }
}

impl AsTarget for Output<'_> {
    #[inline]
    fn as_target(&self) -> Target<'_> {
        Target::new(self.format, &self.view)
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
            wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Outdated => Action::Run,
            wgpu::SurfaceError::Lost => Action::Recreate,
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
