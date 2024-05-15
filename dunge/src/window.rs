//! Window types.

use {
    crate::{
        context::{Context, FailedMakeContext},
        el::{Loop, LoopError},
        element::Element,
        format::Format,
        state::{State, Target},
        update::Update,
    },
    std::{
        error, fmt,
        future::{Future, IntoFuture},
        marker::PhantomData,
        ops,
        pin::Pin,
        sync::Arc,
        task::{self, Poll},
    },
    wgpu::{
        CreateSurfaceError, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture,
        TextureView,
    },
    winit::{
        error::{EventLoopError, OsError},
        event_loop::{ActiveEventLoop, EventLoopClosed, EventLoopProxy},
        window::{self, WindowAttributes, WindowId},
    },
};

/// Creates the [window builder](WindowBuilder) to
/// construct the [window](Window).
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
#[deprecated]
#[cfg(all(feature = "winit", not(target_arch = "wasm32")))]
pub fn window<V>() -> WindowBuilder<V> {
    WindowBuilder::new(Element(()))
}

/// Creates the [window builder](WindowBuilder) to
/// construct the [window](Window)
/// in the given html element.
#[deprecated]
#[cfg(all(feature = "winit", target_arch = "wasm32"))]
pub fn from_element<V>(id: &str) -> WindowBuilder<V> {
    use web_sys::Window;

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let Some(inner) = document.get_element_by_id(id) else {
        panic!("an element with id {id:?} not found");
    };

    let element = Element(inner);
    WindowBuilder::new(element)
}

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
pub struct WindowBuilder<V> {
    element: Option<Element>,
    title: String,
    size: Option<(u32, u32)>,
    evty: PhantomData<V>,
}

impl<V> WindowBuilder<V> {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element: Some(element),
            title: String::default(),
            size: Some((600, 600)),
            evty: PhantomData,
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

    fn build(&mut self, cx: Context) -> Result<Window<V>, Error> {
        use {
            std::mem,
            winit::{dpi::PhysicalSize, window::Fullscreen},
        };

        let lu = Loop::new()?;
        let title = mem::take(&mut self.title);
        let attrs = window::Window::default_attributes().with_title(title);
        let attrs = match self.size {
            Some((width, height)) => attrs.with_inner_size(PhysicalSize::new(width, height)),
            None => attrs.with_fullscreen(Some(Fullscreen::Borderless(None))),
        };

        let view = {
            let el = self.element.take().expect("take the element once");
            // el.set_canvas(&inner);
            // el.set_window_size(&inner);
            View::new(WindowState { attrs, el })
        };

        Ok(Window { cx, lu, view })
    }
}

impl<V> IntoFuture for WindowBuilder<V>
where
    V: 'static,
{
    type Output = Result<Window<V>, Error>;
    type IntoFuture = Build<V>;

    fn into_future(mut self) -> Self::IntoFuture {
        let fut = async move {
            let cx = Context::new().await?;
            self.build(cx)
        };

        Build(Box::pin(fut))
    }
}

type BoxedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

pub struct Build<V>(BoxedFuture<Result<Window<V>, Error>>)
where
    V: 'static;

impl<V> Future for Build<V>
where
    V: 'static,
{
    type Output = Result<Window<V>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.as_mut().poll(cx)
    }
}

/// The application window type.
#[deprecated]
pub struct Window<V = ()>
where
    V: 'static,
{
    cx: Context,
    lu: Loop<V>,
    view: View,
}

impl<V> Window<V>
where
    V: 'static,
{
    /// Returns the main dunge context.
    pub fn context(&self) -> Context {
        self.cx.clone()
    }

    /// Creates an event notifier.
    pub fn notifier(&self) -> Notifier<V> {
        Notifier(self.lu.inner().create_proxy())
    }

    /// Runs the main event loop locally.
    ///
    /// # Errors
    /// Returns [`LoopError`] if the main loop is stopped due to an error.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_local<U>(self, upd: U) -> Result<(), LoopError>
    where
        U: Update<Event = V>,
    {
        self.lu.run(self.cx, self.view, upd)
    }

    /// Runs the main event loop.
    ///
    /// # Errors
    /// Returns [`LoopError`] if the main loop is stopped due to an error.
    /// Note, on wasm platform this method will return `Ok(())` immediately
    /// without blocking and will never return an error from here.
    pub fn run<U>(self, upd: U) -> Result<(), LoopError>
    where
        U: Update<Event = V> + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.lu.run(self.cx, self.view, upd)
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.lu.spawn(self.cx, self.view, upd);
            Ok(())
        }
    }
}

impl<V> ops::Deref for Window<V> {
    type Target = View;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

pub struct Notifier<V>(EventLoopProxy<V>)
where
    V: 'static;

impl<V> Notifier<V>
where
    V: 'static,
{
    /// Sends a new event to the main loop.
    ///
    /// # Errors
    /// If the main loop was stopped, the event will return back.
    pub fn notify(&self, ev: V) -> Result<(), V> {
        self.0.send_event(ev).map_err(|EventLoopClosed(ev)| ev)
    }
}

pub struct WindowState {
    attrs: WindowAttributes,
    el: Element,
}

impl WindowState {
    /// Set the title to the window.
    pub fn with_title<S>(self, title: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            attrs: self.attrs.with_title(title),
            ..self
        }
    }

    /// Set the window size.
    pub fn with_size(self, (width, height): (u32, u32)) -> Self {
        use winit::dpi::PhysicalSize;

        let size = PhysicalSize::new(width, height);
        Self {
            attrs: self.attrs.with_inner_size(size),
            ..self
        }
    }

    /// Enables fullscreen for the window.
    pub fn with_fullscreen(self) -> Self {
        use winit::window::Fullscreen;

        let fullscreen = Fullscreen::Borderless(None);
        Self {
            attrs: self.attrs.with_fullscreen(Some(fullscreen)),
            ..self
        }
    }
}

#[cfg(all(feature = "winit", not(target_arch = "wasm32")))]
pub fn window_state() -> WindowState {
    WindowState {
        attrs: WindowAttributes::default(),
        el: Element(()),
    }
}

#[cfg(all(feature = "winit", target_arch = "wasm32"))]
pub fn window_state_from_element(id: &str) -> WindowState {
    use web_sys::Window;

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let Some(inner) = document.get_element_by_id(id) else {
        panic!("an element with id {id:?} not found");
    };

    WindowState {
        attrs: WindowAttributes::default(),
        el: Element(inner),
    }
}

enum Init {
    Empty(Box<WindowAttributes>),
    Active(Inner),
}

impl Init {
    fn get(&self) -> &Inner {
        match self {
            Self::Empty(_) => panic!("the window should be initialized"),
            Self::Active(inner) => inner,
        }
    }

    fn get_mut(&mut self) -> &mut Inner {
        match self {
            Self::Empty(_) => panic!("the window should be initialized"),
            Self::Active(inner) => inner,
        }
    }
}

pub struct View {
    init: Init,
    id: WindowId,
    el: Element,
    format: Format,
    size: (u32, u32),
}

impl View {
    pub fn new(state: WindowState) -> Self {
        Self {
            init: Init::Empty(Box::new(state.attrs)),
            id: WindowId::from(u64::MAX),
            el: state.el,
            format: Format::default(),
            size: (1, 1),
        }
    }

    pub(crate) fn init(&mut self, state: &State, el: &ActiveEventLoop) -> Result<(), Error> {
        match &mut self.init {
            Init::Empty(attrs) => {
                let attrs = (**attrs).clone();
                let window = el.create_window(attrs)?;
                self.id = window.id();
                self.el.set_canvas(&window);
                self.el.set_window_size(&window);

                let inner = Inner::new(state, window)?;
                self.format = inner.format();
                self.size = inner.size();
                self.init = Init::Active(inner);
                Ok(())
            }
            Init::Active(_) => Ok(()),
        }
    }

    pub fn window(&self) -> &Arc<window::Window> {
        &self.init.get().window
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub(crate) fn id(&self) -> WindowId {
        self.id
    }

    pub(crate) fn request_redraw(&self) {
        self.init.get().window.request_redraw();
    }

    pub(crate) fn output(&self) -> Result<Output, SurfaceError> {
        use wgpu::TextureViewDescriptor;

        let inner = self.init.get();
        let format = Format::from_wgpu(inner.conf.format);
        let output = inner.surface.get_current_texture()?;
        let view = {
            let desc = TextureViewDescriptor::default();
            output.texture.create_view(&desc)
        };

        Ok(Output {
            view,
            format,
            output,
        })
    }

    pub(crate) fn set_window_size(&self) {
        let inner = self.init.get();
        self.el.set_window_size(&inner.window);
    }

    pub(crate) fn resize(&mut self, state: &State) {
        let inner = self.init.get_mut();
        let size = inner.window.inner_size();
        if size.width > 0 && size.height > 0 {
            inner.conf.width = size.width;
            inner.conf.height = size.height;
            inner.surface.configure(state.device(), &inner.conf);
            self.size = inner.size();
        }
    }
}

struct Inner {
    conf: SurfaceConfiguration,
    surface: Surface<'static>,
    window: Arc<window::Window>,
}

impl Inner {
    fn new(state: &State, window: window::Window) -> Result<Self, Error> {
        use wgpu::*;

        const SUPPORTED_FORMATS: [Format; 4] = [
            Format::SrgbAlpha,
            Format::SbgrAlpha,
            Format::RgbAlpha,
            Format::BgrAlpha,
        ];

        let window = Arc::new(window);
        let surface = state.instance().create_surface(Arc::clone(&window))?;
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

            let size = window.inner_size();
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
            window,
        })
    }

    fn format(&self) -> Format {
        Format::from_wgpu(self.conf.format)
    }

    fn size(&self) -> (u32, u32) {
        (self.conf.width, self.conf.height)
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

impl From<FailedMakeContext> for Error {
    fn from(v: FailedMakeContext) -> Self {
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
    Context(FailedMakeContext),
}
