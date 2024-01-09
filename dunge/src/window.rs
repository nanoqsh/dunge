use {
    crate::{
        context::{self, Context},
        el::Loop,
        format::Format,
        state::{RenderView, State},
        update::Update,
    },
    std::{error, fmt},
    wgpu::{
        CreateSurfaceError, Instance, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture,
        TextureView,
    },
    winit::{
        error::{EventLoopError, OsError},
        window::{self, WindowId},
    },
};

#[cfg(not(target_arch = "wasm32"))]
use crate::el::LoopError;

pub struct WindowBuilder {
    title: String,
    size: Option<(u32, u32)>,
}

impl WindowBuilder {
    pub(crate) fn new() -> Self {
        Self {
            title: String::default(),
            size: Some((600, 600)),
        }
    }

    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_fullscreen(mut self) -> Self {
        self.size = None;
        self
    }

    pub(crate) fn build(self, cx: Context, instance: &Instance) -> Result<Window, Error> {
        use winit::{dpi::PhysicalSize, window::Fullscreen};

        let el = Loop::new()?;
        let inner = {
            let builder = window::WindowBuilder::new().with_title(self.title);
            let builder = match self.size {
                Some((width, height)) => builder.with_inner_size(PhysicalSize::new(width, height)),
                None => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
            };

            builder.build(el.inner())?
        };

        let view = View::new(cx.state(), instance, inner)?;
        Ok(Window { cx, el, view })
    }
}

pub struct Window {
    cx: Context,
    el: Loop,
    view: View,
}

impl Window {
    pub fn context(&self) -> Context {
        self.cx.clone()
    }

    pub fn format(&self) -> Format {
        self.view.format()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<U>(self, update: U) -> Result<(), LoopError>
    where
        U: Update,
    {
        self.el.run(self.cx, self.view, update)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn<U>(self, update: U)
    where
        U: Update + 'static,
    {
        self.el.spawn(self.cx, self.view, update)
    }
}

pub(crate) struct View {
    conf: SurfaceConfiguration,
    surface: Surface,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    inner: window::Window,
}

impl View {
    fn new(state: &State, instance: &Instance, inner: window::Window) -> Result<Self, Error> {
        use wgpu::*;

        const SUPPORTED_FORMATS: [Format; 2] = [Format::RgbAlpha, Format::BgrAlpha];

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&inner)? };
        let conf = {
            let caps = surface.get_capabilities(state.adapter());
            let format = SUPPORTED_FORMATS.into_iter().find_map(|format| {
                let format = format.wgpu();
                caps.formats.contains(&format).then_some(format)
            });

            let Some(format) = format else {
                log::error!("surface formats: {formats:?}", formats = &caps.formats);
                return Err(ErrorKind::UnsupportedSurface.into());
            };

            let size = inner.inner_size();
            SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: PresentMode::default(),
                alpha_mode: CompositeAlphaMode::default(),
                view_formats: vec![],
            }
        };

        surface.configure(state.device(), &conf);
        Ok(Self {
            conf,
            surface,
            inner,
        })
    }

    fn format(&self) -> Format {
        Format::from_wgpu(self.conf.format).expect("supported format")
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    pub fn output(&self) -> Result<Output, SurfaceError> {
        use wgpu::TextureViewDescriptor;

        let output = self.surface.get_current_texture()?;
        let view = {
            let desc = TextureViewDescriptor::default();
            output.texture.create_view(&desc)
        };

        Ok(Output {
            view,
            format: self.format(),
            surface: output,
        })
    }

    pub fn resize(&mut self, state: &State) {
        let size = self.inner.inner_size();
        if size.width > 0 && size.height > 0 {
            self.conf.width = size.width;
            self.conf.height = size.height;
            self.surface.configure(state.device(), &self.conf);
        }
    }
}

pub(crate) struct Output {
    view: TextureView,
    format: Format,
    surface: SurfaceTexture,
}

impl Output {
    pub fn render_view(&self) -> RenderView {
        RenderView::new(&self.view, self.format)
    }

    pub fn present(self) {
        self.surface.present();
    }
}

#[derive(Debug)]
pub struct Error(ErrorKind);

impl From<ErrorKind> for Error {
    fn from(v: ErrorKind) -> Self {
        Self(v)
    }
}

impl From<EventLoopError> for Error {
    fn from(v: EventLoopError) -> Self {
        Self(ErrorKind::EventLoop(v))
    }
}

impl From<OsError> for Error {
    fn from(v: OsError) -> Self {
        Self(ErrorKind::Os(v))
    }
}

impl From<CreateSurfaceError> for Error {
    fn from(v: CreateSurfaceError) -> Self {
        Self(ErrorKind::Surface(v))
    }
}

impl From<context::Error> for Error {
    fn from(v: context::Error) -> Self {
        Self(ErrorKind::Context(v))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            ErrorKind::UnsupportedSurface => write!(f, "unsupported surface"),
            ErrorKind::EventLoop(err) => err.fmt(f),
            ErrorKind::Os(err) => err.fmt(f),
            ErrorKind::Surface(err) => err.fmt(f),
            ErrorKind::Context(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            ErrorKind::UnsupportedSurface => None,
            ErrorKind::EventLoop(err) => Some(err),
            ErrorKind::Os(err) => Some(err),
            ErrorKind::Surface(err) => Some(err),
            ErrorKind::Context(err) => Some(err),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    UnsupportedSurface,
    EventLoop(EventLoopError),
    Os(OsError),
    Surface(CreateSurfaceError),
    Context(context::Error),
}
