use {
    dunge::{
        Context, FailedMakeContext,
        surface::{Surface, SurfaceError, WindowOps},
    },
    std::{
        cell::{Cell, OnceCell},
        collections::HashMap,
        error, fmt, future,
        pin::{self, Pin},
        rc::Rc,
        task::{self, Poll, Waker},
    },
    winit::{application::ApplicationHandler, event, event_loop, keyboard, window},
};

struct App<'app, F> {
    cx: Context,
    proxy: event_loop::EventLoopProxy<Request>,
    lifecycle: &'app Lifecycle,
    windows: HashMap<window::WindowId, Window>,
    need_to_poll: bool,
    fu: F,
}

impl<F> App<'_, Pin<&mut F>> {
    fn schedule(&mut self) {
        self.need_to_poll = true;
    }

    fn active_poll(&mut self, el: &event_loop::ActiveEventLoop)
    where
        F: Future,
    {
        if !self.need_to_poll {
            return;
        }

        let mut noop = const { task::Context::from_waker(Waker::noop()) };
        if self.fu.as_mut().poll(&mut noop).is_ready() {
            el.exit();
        }

        self.need_to_poll = false;
    }
}

enum Request {
    MakeWindow {
        out: Rc<OnceCell<Result<Window, Error>>>,
        attr: Box<Attributes>,
    },
    RemoveWindow(window::WindowId),
}

impl<F> ApplicationHandler<Request> for App<'_, Pin<&mut F>>
where
    F: Future,
{
    fn resumed(&mut self, _: &event_loop::ActiveEventLoop) {
        self.lifecycle.set(LifecycleState::Resumed);
        self.schedule();
    }

    fn suspended(&mut self, _: &event_loop::ActiveEventLoop) {
        self.lifecycle.set(LifecycleState::Suspended);
        self.schedule();
    }

    fn user_event(&mut self, el: &event_loop::ActiveEventLoop, req: Request) {
        match req {
            Request::MakeWindow { out, attr } => {
                let res = Window::new(&self.cx, self.proxy.clone(), el, attr.winit());
                if let Ok(window) = &res {
                    self.windows.insert(window.id, window.clone());
                }

                _ = out.set(res);
                self.schedule();
            }
            Request::RemoveWindow(id) => _ = self.windows.remove(&id),
        }
    }

    fn window_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
        id: window::WindowId,
        event: event::WindowEvent,
    ) {
        let Some(window) = self.windows.get(&id) else {
            return;
        };

        match event {
            event::WindowEvent::Resized(size) => {
                let (width, height) = size.into();
                log::debug!("resized {id:?}: {width}, {height}");

                window.inner.surface.resize(&self.cx);
                window.inner.resize.set((width, height));
                self.schedule();
            }
            event::WindowEvent::CloseRequested => {
                log::debug!("close requested {id:?}");
                window.inner.close.set();
                self.schedule();
            }
            event::WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                is_synthetic: false,
                ..
            } => {
                let code = match physical_key {
                    keyboard::PhysicalKey::Code(code) => {
                        log::debug!("keyboard input: {code:?}");
                        code
                    }
                    keyboard::PhysicalKey::Unidentified(code) => {
                        log::debug!("keyboard input: (unidentified) {code:?}");
                        return;
                    }
                };

                match state {
                    event::ElementState::Pressed => todo!("{code:?} pressed"),
                    event::ElementState::Released => todo!("{code:?} released"),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &event_loop::ActiveEventLoop) {
        self.active_poll(el);
    }
}

pub fn block_on<F, R>(mut f: F) -> Result<R, Error>
where
    F: AsyncFnMut(Control<'_>) -> R,
{
    let el = event_loop::EventLoop::with_user_event()
        .build()
        .map_err(Error::EventLoop)?;

    let lifecycle = Lifecycle {
        state: Cell::new(LifecycleState::Suspended),
    };

    let cx = dunge::block_on(dunge::context()).map_err(Error::Context)?;
    let ctrl = Control {
        cx: cx.clone(),
        proxy: el.create_proxy(),
        lifecycle: &lifecycle,
    };

    let res = Cell::new(None);
    let mut app = App {
        cx,
        proxy: el.create_proxy(),
        lifecycle: &lifecycle,
        windows: HashMap::new(),
        need_to_poll: true, // an initial poll
        fu: pin::pin!(async {
            let out = f(ctrl).await;
            res.set(Some(out));
        }),
    };

    el.run_app(&mut app).map_err(Error::EventLoop)?;
    Ok(res.take().expect("take result of async function"))
}

#[derive(Debug)]
pub enum Error {
    Context(FailedMakeContext),
    EventLoop(winit::error::EventLoopError),
    Os(winit::error::OsError),
    Surface(SurfaceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(e) => e.fmt(f),
            Self::EventLoop(e) => e.fmt(f),
            Self::Os(e) => e.fmt(f),
            Self::Surface(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Context(e) => Some(e),
            Self::EventLoop(e) => Some(e),
            Self::Os(e) => Some(e),
            Self::Surface(e) => Some(e),
        }
    }
}

#[derive(Clone, Copy)]
enum LifecycleState {
    Resumed,
    Suspended,
}

struct Lifecycle {
    state: Cell<LifecycleState>,
}

impl Lifecycle {
    fn set(&self, state: LifecycleState) {
        self.state.set(state);
    }

    fn active_poll_resumed(&self) -> Poll<()> {
        if let LifecycleState::Resumed = self.state.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn active_poll_suspended(&self) -> Poll<()> {
        if let LifecycleState::Suspended = self.state.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub struct Control<'app> {
    cx: Context,
    proxy: event_loop::EventLoopProxy<Request>,
    lifecycle: &'app Lifecycle,
}

impl Control<'_> {
    #[inline]
    pub fn context(&self) -> Context {
        self.cx.clone()
    }

    #[inline]
    pub async fn resumed(&self) {
        future::poll_fn(|_| self.lifecycle.active_poll_resumed()).await;
    }

    #[inline]
    pub async fn suspended(&self) {
        future::poll_fn(|_| self.lifecycle.active_poll_suspended()).await;
    }

    #[inline]
    pub async fn make_window(&self, attr: Attributes) -> Result<Window, Error> {
        let mut out = Rc::new(OnceCell::new());
        _ = self.proxy.send_event(Request::MakeWindow {
            out: out.clone(),
            attr: Box::new(attr),
        });

        let mut poll = || {
            Rc::get_mut(&mut out).map_or(Poll::Pending, |out| {
                Poll::Ready(out.take().expect("take window"))
            })
        };

        future::poll_fn(|_| poll()).await
    }
}

#[derive(Default)]
pub struct Attributes {
    title: String,
}

impl Attributes {
    #[inline]
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    #[inline]
    fn winit(self) -> window::WindowAttributes {
        window::WindowAttributes::default().with_title(self.title)
    }
}

struct Resize(Cell<Option<(u32, u32)>>);

impl Resize {
    #[inline]
    const fn new() -> Self {
        Self(Cell::new(None))
    }

    #[inline]
    fn set(&self, size: (u32, u32)) {
        self.0.set(Some(size));
    }

    #[inline]
    fn active_poll(&self) -> Poll<(u32, u32)> {
        self.0.take().map_or(Poll::Pending, Poll::Ready)
    }
}

struct Keys {}

impl Keys {
    const fn new() -> Self {
        Self {}
    }
}

struct Close(Cell<bool>);

impl Close {
    #[inline]
    const fn new() -> Self {
        Self(Cell::new(false))
    }

    #[inline]
    fn set(&self) {
        self.0.set(true);
    }

    #[inline]
    fn active_poll(&self) -> Poll<()> {
        if self.0.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

struct Inner {
    surface: Surface<window::Window, Ops>,
    resize: Resize,
    #[expect(dead_code)]
    keys: Keys,
    close: Close,
}

#[derive(Clone)]
pub struct Window {
    id: window::WindowId,
    inner: Rc<Inner>,
    proxy: event_loop::EventLoopProxy<Request>,
}

impl Window {
    #[inline]
    fn new(
        cx: &Context,
        proxy: event_loop::EventLoopProxy<Request>,
        el: &event_loop::ActiveEventLoop,
        attr: window::WindowAttributes,
    ) -> Result<Self, Error> {
        let window = el.create_window(attr).map_err(Error::Os)?;
        let id = window.id();
        let inner = Rc::new(Inner {
            surface: Surface::new(cx, window).map_err(Error::Surface)?,
            resize: const { Resize::new() },
            keys: const { Keys::new() },
            close: const { Close::new() },
        });

        Ok(Self { id, inner, proxy })
    }

    #[inline]
    pub async fn resized(&self) -> (u32, u32) {
        future::poll_fn(|_| self.inner.resize.active_poll()).await
    }

    #[inline]
    pub async fn close_requested(&self) {
        future::poll_fn(|_| self.inner.close.active_poll()).await;
    }
}

impl Drop for Window {
    #[inline]
    fn drop(&mut self) {
        _ = self.proxy.send_event(Request::RemoveWindow(self.id));
    }
}

struct Ops;

impl WindowOps<window::Window> for Ops {
    #[inline]
    fn size(window: &window::Window) -> (u32, u32) {
        window.inner_size().into()
    }
}
