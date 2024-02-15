//! Window types.

use {
    crate::{
        context::{Context, MakeContextError},
        el::Loop,
        element::Element,
        format::Format,
        init,
        state::{State, Target},
        update::Update,
    },
    std::{
        error, fmt,
        future::{Future, IntoFuture},
        ops,
        pin::Pin,
        sync::Arc,
        task::{self, Poll},
    },
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

/// The [window](Window) builder.
///
/// This builder completes asynchronously, so to create
/// the window object, call `.await` at the end of configuration.
///
/// # Example
/// ```rust
/// # fn t() -> impl std::future::Future<Output = Result<dunge::window::Window, dunge::window::Error>> {
/// async {
///     let window = dunge::window().with_title("Hello").await?;
///     Ok(window)
/// }
/// # }
/// ```
pub struct WindowBuilder {
    element: Option<Element>,
    title: String,
    size: Option<(u32, u32)>,
}

impl WindowBuilder {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element: Some(element),
            title: String::default(),
            size: Some((600, 600)),
        }
    }

    /// Set the title to the window.
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    /// Set the window size.
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }

    /// Enables fullscreen for the window.
    pub fn with_fullscreen(mut self) -> Self {
        self.size = None;
        self
    }

    fn build(&mut self, cx: Context, instance: &Instance) -> Result<Window, Error> {
        use {
            std::mem,
            winit::{dpi::PhysicalSize, window::Fullscreen},
        };

        let lu = Loop::new()?;
        let inner = {
            let title = mem::take(&mut self.title);
            let builder = window::WindowBuilder::new().with_title(title);
            let builder = match self.size {
                Some((width, height)) => builder.with_inner_size(PhysicalSize::new(width, height)),
                None => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
            };

            Arc::new(builder.build(lu.inner())?)
        };

        let view = {
            let el = self.element.take().expect("take the element once");
            el.set_canvas(&inner);
            el.set_window_size(&inner);
            View::new(cx.state(), instance, inner, el)?
        };

        Ok(Window { cx, lu, view })
    }
}

impl IntoFuture for WindowBuilder {
    type Output = Result<Window, Error>;
    type IntoFuture = Build;

    fn into_future(mut self) -> Self::IntoFuture {
        let fut = async move {
            let (cx, instance) = init::make().await?;
            self.build(cx, &instance)
        };

        Build(Box::pin(fut))
    }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

pub struct Build(BoxFuture<Result<Window, Error>>);

impl Future for Build {
    type Output = Result<Window, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.as_mut().poll(cx)
    }
}

pub struct Window {
    cx: Context,
    lu: Loop,
    view: View,
}

impl Window {
    pub fn context(&self) -> Context {
        self.cx.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<U>(self, upd: U) -> Result<(), LoopError>
    where
        U: Update,
    {
        self.lu.run(self.cx, self.view, upd)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn<U>(self, upd: U)
    where
        U: Update + 'static,
    {
        self.lu.spawn(self.cx, self.view, upd)
    }
}

impl ops::Deref for Window {
    type Target = View;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

type Inner = Arc<window::Window>;

pub struct View {
    conf: SurfaceConfiguration,
    surface: Surface<'static>,
    inner: Inner,
    el: Element,
}

impl View {
    fn new(state: &State, instance: &Instance, inner: Inner, el: Element) -> Result<Self, Error> {
        use wgpu::*;

        const SUPPORTED_FORMATS: [Format; 4] = [
            Format::RgbAlpha,
            Format::BgrAlpha,
            Format::RgbAlphaLin,
            Format::BgrAlphaLin,
        ];

        let surface = instance.create_surface(Arc::clone(&inner))?;
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
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::default(),
                view_formats: vec![],
            }
        };

        surface.configure(state.device(), &conf);
        Ok(Self {
            conf,
            surface,
            inner,
            el,
        })
    }

    pub fn window(&self) -> Inner {
        Arc::clone(&self.inner)
    }

    pub fn format(&self) -> Format {
        Format::from_wgpu(self.conf.format)
    }

    pub fn size(&self) -> (u32, u32) {
        (self.conf.width, self.conf.height)
    }

    pub(crate) fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    pub(crate) fn output(&self) -> Result<Output, SurfaceError> {
        use wgpu::TextureViewDescriptor;

        let output = self.surface.get_current_texture()?;
        let view = {
            let desc = TextureViewDescriptor::default();
            output.texture.create_view(&desc)
        };

        Ok(Output {
            view,
            format: self.format(),
            output,
        })
    }

    pub(crate) fn set_window_size(&self) {
        self.el.set_window_size(&self.inner);
    }

    pub(crate) fn resize(&mut self, state: &State) {
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
    output: SurfaceTexture,
}

impl Output {
    pub fn target(&self) -> Target {
        Target::new(self.format, &self.view)
    }

    pub fn present(self) {
        self.output.present();
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

impl From<MakeContextError> for Error {
    fn from(v: MakeContextError) -> Self {
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
    Context(MakeContextError),
}
