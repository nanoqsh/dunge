use {
    crate::time::Time,
    dunge::{
        AsTarget, Context, FailedMakeContext, Target,
        buffer::Format,
        surface::{Action, CreateSurfaceError, Output, Surface, SurfaceError, WindowOps},
    },
    std::{
        cell::{Cell, OnceCell, RefCell},
        collections::{HashMap, hash_map::Entry},
        convert::Infallible,
        error, fmt, future,
        num::NonZeroU32,
        pin::{self, Pin},
        rc::Rc,
        sync::Arc,
        task::{self, Poll, Waker},
        time::Duration,
    },
    winit::{application::ApplicationHandler, event, event_loop, keyboard, window},
};

enum Request {
    MakeWindow {
        out: Rc<OnceCell<Result<Window, Error>>>,
        attr: Box<Attributes>,
    },
    RemoveWindow(window::WindowId),
    RecreateSurface(window::WindowId),
    Exit(Box<Error>),
}

struct App<'app, F> {
    cx: Context,
    proxy: event_loop::EventLoopProxy<Request>,
    lifecycle: &'app Lifecycle,
    windows: HashMap<window::WindowId, Window>,
    res: Result<(), Error>,
    need_to_poll: bool,
    fu: F,
}

impl<F> App<'_, Pin<&mut F>> {
    const WAIT_TIME: Duration = Duration::from_millis(100);

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

    fn exit_with_error(&mut self, el: &event_loop::ActiveEventLoop, e: Error)
    where
        F: Future,
    {
        self.res = Err(e);
        el.exit();
    }
}

impl<F> ApplicationHandler<Request> for App<'_, Pin<&mut F>>
where
    F: Future,
{
    fn new_events(&mut self, el: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        match cause {
            event::StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");

                for window in self.windows.values() {
                    window.inner.surface.window().request_redraw();
                }
            }
            event::StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                log::debug!("wait cancelled");
                let flow = match requested_resume {
                    Some(resume) => event_loop::ControlFlow::WaitUntil(resume),
                    None => event_loop::ControlFlow::wait_duration(Self::WAIT_TIME),
                };

                el.set_control_flow(flow);
            }
            event::StartCause::Poll => log::debug!("poll"),
            event::StartCause::Init => log::debug!("init"),
        }
    }

    fn resumed(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("resumed");
        self.lifecycle.set(LifecycleState::Active);

        for window in self.windows.values() {
            window.inner.surface.window().request_redraw();
        }

        self.schedule();
    }

    fn suspended(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("suspended");
        self.lifecycle.set(LifecycleState::Inactive);
        self.schedule();
    }

    fn user_event(&mut self, el: &event_loop::ActiveEventLoop, req: Request) {
        match req {
            Request::MakeWindow { out, attr } => {
                log::debug!("make window");
                let res = Window::new(&self.cx, self.proxy.clone(), el, attr.winit());
                if let Ok(window) = &res {
                    let id = window.inner.surface.window().id();
                    self.windows.insert(id, window.clone());
                    window.inner.surface.window().request_redraw();
                }

                _ = out.set(res);
                self.schedule();
            }
            Request::RemoveWindow(id) => {
                log::debug!("remove window {id:?}");
                _ = self.windows.remove(&id);
            }
            Request::RecreateSurface(id) => {
                log::debug!("recreate surface {id:?}");
                let Some(window) = self.windows.get(&id) else {
                    return;
                };

                window.inner.surface.resize(&self.cx);
                self.schedule();
            }
            Request::Exit(e) => {
                log::debug!("exit with error: {e}");
                self.exit_with_error(el, *e);
            }
        }
    }

    fn window_event(
        &mut self,
        el: &event_loop::ActiveEventLoop,
        id: window::WindowId,
        event: event::WindowEvent,
    ) {
        let Some(window) = self.windows.get(&id) else {
            return;
        };

        match event {
            event::WindowEvent::Resized(size) => {
                let (width, height): (u32, u32) = size.into();
                log::debug!("resized {id:?}: {width} {height}");

                window.inner.surface.resize(&self.cx);
                window.inner.resize.set();
                self.schedule();
            }
            event::WindowEvent::CloseRequested => {
                log::debug!("close requested {id:?}");
                window.inner.close.set();
                self.schedule();
            }
            event::WindowEvent::Focused(focused) => {
                if focused {
                    log::debug!("focused {id:?}");
                    window.inner.surface.window().request_redraw();
                    self.schedule();
                } else {
                    log::debug!("unfocused {id:?}");
                }
            }
            event::WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        physical_key,
                        state,
                        repeat: false,
                        ..
                    },
                is_synthetic: false,
                ..
            } => {
                let code = match physical_key {
                    keyboard::PhysicalKey::Code(code) => {
                        log::debug!("keyboard input {id:?}: {state:?} {code:?}");
                        code
                    }
                    keyboard::PhysicalKey::Unidentified(code) => {
                        log::debug!("keyboard input {id:?}: {state:?} (unidentified) {code:?}");
                        return;
                    }
                };

                match state {
                    event::ElementState::Pressed => window.inner.press_keys.active(code),
                    event::ElementState::Released => window.inner.release_keys.active(code),
                }

                self.schedule();
            }
            event::WindowEvent::RedrawRequested => {
                if let LifecycleState::Active = self.lifecycle.state.get() {
                    log::debug!("redraw requested {id:?}");
                } else {
                    log::debug!("redraw requested {id:?} (inactive)");

                    // Wait a while to become active
                    el.set_control_flow(event_loop::ControlFlow::wait_duration(Self::WAIT_TIME));
                    return;
                }

                let delta_time = window.inner.time.borrow_mut().delta();
                let min_delta_time = window.inner.min_delta_time.get();
                if delta_time < min_delta_time {
                    let wait = min_delta_time - delta_time;
                    el.set_control_flow(event_loop::ControlFlow::wait_duration(wait));
                    return;
                }

                window.inner.time.borrow_mut().reset();
                window.inner.redraw.set_value(delta_time);
                self.schedule();
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
        state: Cell::new(LifecycleState::Inactive),
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
        res: Ok(()),
        need_to_poll: true, // an initial poll
        fu: pin::pin!(async {
            let out = f(ctrl).await;
            res.set(Some(out));
        }),
    };

    el.run_app(&mut app).map_err(Error::EventLoop)?;
    app.res?;
    Ok(res.take().expect("take result of async function"))
}

pub fn try_block_on<F, R, U>(f: F) -> Result<R, Error<U>>
where
    F: AsyncFnMut(Control<'_>) -> Result<R, U>,
{
    match block_on(f) {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(u)) => Err(Error::Custom(u)),
        Err(e) => Err(e.map(|never| match never {})),
    }
}

#[derive(Debug)]
pub enum Error<U = Infallible> {
    Context(FailedMakeContext),
    EventLoop(winit::error::EventLoopError),
    Os(winit::error::OsError),
    CreateSurface(CreateSurfaceError),
    Surface(SurfaceError),
    Custom(U),
}

impl<U> Error<U> {
    pub fn map<F, V>(self, f: F) -> Error<V>
    where
        F: FnOnce(U) -> V,
    {
        match self {
            Self::Context(e) => Error::Context(e),
            Self::EventLoop(e) => Error::EventLoop(e),
            Self::Os(e) => Error::Os(e),
            Self::CreateSurface(e) => Error::CreateSurface(e),
            Self::Surface(e) => Error::Surface(e),
            Self::Custom(u) => Error::Custom(f(u)),
        }
    }
}

impl<U> fmt::Display for Error<U>
where
    U: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(e) => e.fmt(f),
            Self::EventLoop(e) => e.fmt(f),
            Self::Os(e) => e.fmt(f),
            Self::CreateSurface(e) => e.fmt(f),
            Self::Surface(e) => e.fmt(f),
            Self::Custom(e) => e.fmt(f),
        }
    }
}

impl<U> error::Error for Error<U>
where
    U: error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Context(e) => Some(e),
            Self::EventLoop(e) => Some(e),
            Self::Os(e) => Some(e),
            Self::CreateSurface(e) => Some(e),
            Self::Surface(e) => Some(e),
            Self::Custom(e) => Some(e),
        }
    }
}

#[derive(Clone, Copy)]
enum LifecycleState {
    Active,
    Inactive,
}

struct Lifecycle {
    state: Cell<LifecycleState>,
}

impl Lifecycle {
    fn set(&self, state: LifecycleState) {
        self.state.set(state);
    }

    fn active_poll_resumed(&self) -> Poll<()> {
        if let LifecycleState::Active = self.state.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn active_poll_suspended(&self) -> Poll<()> {
        if let LifecycleState::Inactive = self.state.get() {
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

        let mut active_poll = || {
            Rc::get_mut(&mut out).map_or(Poll::Pending, |out| {
                Poll::Ready(out.take().expect("take window"))
            })
        };

        future::poll_fn(|_| active_poll()).await
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

struct Event<T = bool>(Cell<T>);

impl<T> Event<T> {
    #[inline]
    fn new() -> Self
    where
        T: Default,
    {
        Self(Cell::new(T::default()))
    }
}

impl Event {
    #[inline]
    fn set(&self) {
        self.0.set(true);
    }

    #[inline]
    fn active_poll(&self) -> Poll<()> {
        if self.0.take() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<T> Event<Option<T>> {
    #[inline]
    fn set_value(&self, value: T) {
        self.0.set(Some(value));
    }

    #[inline]
    fn active_poll_value(&self) -> Poll<T> {
        if let Some(value) = self.0.take() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

enum KeyState {
    Wait,
    Active,
}

struct Keys(RefCell<HashMap<keyboard::KeyCode, KeyState>>);

impl Keys {
    #[inline]
    fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    #[inline]
    fn wait(&self, code: keyboard::KeyCode) {
        self.0.borrow_mut().insert(code, KeyState::Wait);
    }

    #[inline]
    fn active(&self, code: keyboard::KeyCode) {
        if let Some(state @ KeyState::Wait) = self.0.borrow_mut().get_mut(&code) {
            *state = KeyState::Active;
        }
    }

    #[inline]
    fn active_poll(&self, code: keyboard::KeyCode) -> Poll<()> {
        if let Entry::Occupied(en) = self.0.borrow_mut().entry(code) {
            if let KeyState::Active = en.get() {
                en.remove();
                return Poll::Ready(());
            }
        }

        Poll::Pending
    }
}

struct Inner {
    surface: Surface<window::Window, Ops>,
    time: RefCell<Time>,
    min_delta_time: Cell<Duration>,
    press_keys: Keys,
    release_keys: Keys,
    resize: Event,
    redraw: Event<Option<Duration>>,
    close: Event,
}

#[derive(Clone)]
pub struct Window {
    inner: Rc<Inner>,
    proxy: event_loop::EventLoopProxy<Request>,
}

impl Window {
    const DEFAULT_FPS: u32 = 60;

    #[inline]
    fn new(
        cx: &Context,
        proxy: event_loop::EventLoopProxy<Request>,
        el: &event_loop::ActiveEventLoop,
        attr: window::WindowAttributes,
    ) -> Result<Self, Error> {
        let window = el.create_window(attr).map_err(Error::Os)?;
        let inner = Rc::new(Inner {
            time: RefCell::new(Time::now()),
            min_delta_time: Cell::new(Duration::from_secs_f32(1. / Self::DEFAULT_FPS as f32)),
            surface: Surface::new(cx, window).map_err(Error::CreateSurface)?,
            press_keys: Keys::new(),
            release_keys: Keys::new(),
            resize: Event::new(),
            redraw: Event::new(),
            close: Event::new(),
        });

        Ok(Self { inner, proxy })
    }

    #[inline]
    pub fn winit(&self) -> &Arc<window::Window> {
        self.inner.surface.window()
    }

    #[inline]
    pub fn format(&self) -> Format {
        self.inner.surface.format()
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.inner.surface.size()
    }

    #[inline]
    pub fn set_fps(&self, fps: NonZeroU32) {
        self.inner
            .min_delta_time
            .set(Duration::from_secs_f32(1. / fps.get() as f32));
    }

    #[inline]
    pub async fn pressed(&self, code: keyboard::KeyCode) {
        self.inner.press_keys.wait(code);
        future::poll_fn(|_| self.inner.press_keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn released(&self, code: keyboard::KeyCode) {
        self.inner.release_keys.wait(code);
        future::poll_fn(|_| self.inner.release_keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn resized(&self) -> (u32, u32) {
        future::poll_fn(|_| self.inner.resize.active_poll()).await;
        self.inner.surface.size()
    }

    #[inline]
    pub async fn redraw(&self) -> Redraw<'_> {
        loop {
            let delta_time = future::poll_fn(|_| self.inner.redraw.active_poll_value()).await;
            let e = match self.inner.surface.output() {
                Ok(output) => break Redraw { output, delta_time },
                Err(e) => e,
            };

            log::warn!("surface error: {e}");
            match e.action() {
                Action::Run => {}
                Action::Recreate => {
                    let id = self.inner.surface.window().id();
                    _ = self.proxy.send_event(Request::RecreateSurface(id));
                }
                Action::Exit => {
                    let e = Box::new(Error::Surface(e));
                    _ = self.proxy.send_event(Request::Exit(e));
                }
            }
        }
    }

    #[inline]
    pub async fn close_requested(&self) {
        future::poll_fn(|_| self.inner.close.active_poll()).await;
    }
}

impl Drop for Window {
    #[inline]
    fn drop(&mut self) {
        let id = self.inner.surface.window().id();
        _ = self.proxy.send_event(Request::RemoveWindow(id));
    }
}

pub struct Redraw<'surface> {
    output: Output<'surface>,
    delta_time: Duration,
}

impl Redraw<'_> {
    #[inline]
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    #[inline]
    pub fn present(self) {
        self.output.present();
    }
}

impl AsTarget for Redraw<'_> {
    #[inline]
    fn as_target(&self) -> Target<'_> {
        self.output.as_target()
    }
}

struct Ops;

impl WindowOps<window::Window> for Ops {
    #[inline]
    fn size(window: &window::Window) -> (u32, u32) {
        window.inner_size().into()
    }
}
