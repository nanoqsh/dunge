//! Window types.

use {
    crate::{
        buffer::Format,
        context::{Context, FailedMakeContext},
        el::{self, LoopError},
        element::Element,
        state::{State, Target},
        update::IntoUpdate,
    },
    std::{error, fmt, sync::Arc},
    winit::{
        error::{EventLoopError, OsError},
        event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, EventLoopProxy},
        window::{self, WindowAttributes, WindowId},
    },
};

pub struct Notifier<V>(EventLoopProxy<V>)
where
    V: 'static;

impl<V> Notifier<V> {
    /// Sends a new event to the main loop.
    ///
    /// # Errors
    /// If the main loop was stopped, the event will return back.
    pub fn notify(&self, ev: V) -> Result<(), V> {
        self.0.send_event(ev).map_err(|EventLoopClosed(ev)| ev)
    }
}

pub struct WindowState<V = ()>
where
    V: 'static,
{
    attrs: WindowAttributes,
    el: Element,
    lu: EventLoop<V>,
}

impl<V> WindowState<V> {
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

    /// Creates a new [notifier](Notifier).
    pub fn notifier(&self) -> Notifier<V> {
        Notifier(self.lu.create_proxy())
    }

    /// Runs an event loop.
    pub fn run<U>(self, cx: Context, upd: U) -> Result<(), LoopError>
    where
        U: IntoUpdate<Event = V> + 'static,
    {
        el::run(self, cx, upd)
    }

    /// Locally runs an event loop.
    #[cfg(not(target_family = "wasm"))]
    pub fn run_local<U>(self, cx: Context, upd: U) -> Result<(), LoopError>
    where
        U: IntoUpdate<Event = V>,
    {
        el::run_local(self, cx, upd)
    }

    pub(crate) fn into_view_and_loop(self) -> (View, EventLoop<V>) {
        let view = View {
            init: Init::Empty(Box::new(self.attrs)),
            id: WindowId::from(u64::MAX),
            el: self.el,
            format: Format::default(),
            size: (1, 1),
        };

        (view, self.lu)
    }
}

/// Creates a new [`WindowState`].
#[cfg(all(feature = "winit", not(target_family = "wasm")))]
pub fn window<V>() -> WindowState<V> {
    state(Element(()))
}

/// Creates a [`WindowState`] from an HTML element.
#[cfg(all(feature = "winit", target_family = "wasm"))]
pub fn from_element(id: &str) -> WindowState {
    use web_sys::Window;

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let Some(inner) = document.get_element_by_id(id) else {
        panic!("an element with id {id:?} not found");
    };

    state(Element(inner))
}

fn state<V>(el: Element) -> WindowState<V> {
    let attrs = WindowAttributes::default();
    let Ok(lu) = EventLoop::with_user_event().build() else {
        panic!("attempt to recreate the event loop");
    };

    WindowState { attrs, el, lu }
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

    pub(crate) fn output(&self) -> Result<Output, wgpu::SurfaceError> {
        let inner = self.init.get();
        let format = inner.format();
        let output = inner.surface.get_current_texture()?;
        let view = {
            let desc = wgpu::TextureViewDescriptor::default();
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
    conf: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    window: Arc<window::Window>,
}

impl Inner {
    fn new(state: &State, window: window::Window) -> Result<Self, Error> {
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
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::default(),
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::default(),
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
    view: wgpu::TextureView,
    format: Format,
    output: wgpu::SurfaceTexture,
}

impl Output {
    pub fn target(&self) -> Target<'_> {
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

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(v: wgpu::CreateSurfaceError) -> Self {
        Self(ErrorKind::Surface(v))
    }
}

impl From<FailedMakeContext> for Error {
    fn from(v: FailedMakeContext) -> Self {
        Self(ErrorKind::Context(v))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::UnsupportedSurface => write!(f, "unsupported surface"),
            ErrorKind::EventLoop(e) => e.fmt(f),
            ErrorKind::Os(e) => e.fmt(f),
            ErrorKind::Surface(e) => e.fmt(f),
            ErrorKind::Context(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            ErrorKind::UnsupportedSurface => None,
            ErrorKind::EventLoop(e) => Some(e),
            ErrorKind::Os(e) => Some(e),
            ErrorKind::Surface(e) => Some(e),
            ErrorKind::Context(e) => Some(e),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    UnsupportedSurface,
    EventLoop(EventLoopError),
    Os(OsError),
    Surface(wgpu::CreateSurfaceError),
    Context(FailedMakeContext),
}
