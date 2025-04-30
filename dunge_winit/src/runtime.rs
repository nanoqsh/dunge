use {
    crate::{canvas::Canvas, time::Time},
    dunge::{
        AsTarget, Context, FailedMakeContext, Target,
        buffer::Format,
        surface::{Action, CreateSurfaceError, Output, Surface, SurfaceError, WindowOps},
    },
    std::{
        borrow::Cow,
        cell::{Cell, OnceCell, RefCell},
        collections::{HashMap, hash_map::Entry},
        convert::Infallible,
        error, fmt, future,
        num::NonZeroU32,
        pin::Pin,
        rc::Rc,
        sync::Arc,
        task::{self, Poll, Waker},
        time::Duration,
    },
    winit::{application::ApplicationHandler, event, event_loop, keyboard, window},
};

enum Request {
    #[expect(dead_code)]
    Wake,
    MakeWindow {
        out: Rc<OnceCell<Result<Window, Error>>>,
        attr: Box<Attributes>,
    },
    RemoveWindow(window::WindowId),
    RecreateSurface(window::WindowId),
    Exit(Box<Error>),
}

struct AppWindow {
    shared: Rc<Shared>,
    time: Time,
}

struct App<'waker, F, R> {
    cx: Context,
    proxy: event_loop::EventLoopProxy<Request>,
    lifecycle: Rc<Lifecycle>,
    windows: HashMap<window::WindowId, AppWindow>,
    need_to_poll: bool,
    context: task::Context<'waker>,
    fu: Pin<Box<F>>,
    out: async_channel::Sender<Result<R, Error>>,
}

impl<F> App<'_, F, F::Output>
where
    F: Future,
{
    const WAIT_TIME: Duration = Duration::from_millis(100);

    fn schedule(&mut self) {
        self.need_to_poll = true;
    }

    fn active_poll(&mut self, el: &event_loop::ActiveEventLoop) {
        if !self.need_to_poll {
            return;
        }

        self.inert_poll(el);
        self.need_to_poll = false;
    }

    fn inert_poll(&mut self, el: &event_loop::ActiveEventLoop) {
        if let Poll::Ready(res) = self.fu.as_mut().poll(&mut self.context) {
            _ = self.out.force_send(Ok(res));
            el.exit();
        }
    }

    fn exit_with_error(&mut self, el: &event_loop::ActiveEventLoop, e: Error) {
        _ = self.out.force_send(Err(e));
        el.exit();
    }
}

impl<F> ApplicationHandler<Request> for App<'_, F, F::Output>
where
    F: Future,
{
    fn new_events(&mut self, el: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        match cause {
            event::StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");

                for window in self.windows.values() {
                    window.shared.surface.window().request_redraw();
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
            event::StartCause::Init => {
                log::debug!("init");

                // an initial poll
                self.schedule();
                self.active_poll(el);
            }
        }
    }

    fn resumed(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("resumed");
        self.lifecycle.set(LifecycleState::Active);

        for window in self.windows.values() {
            window.shared.surface.window().request_redraw();
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
            Request::Wake => self.inert_poll(el),
            Request::MakeWindow { out, attr } => {
                log::debug!("make window");
                let res = Window::new(&self.cx, self.proxy.clone(), el, attr.winit());
                if let Ok(window) = &res {
                    let id = window.shared.surface.window().id();
                    self.windows.insert(
                        id,
                        AppWindow {
                            shared: window.shared.clone(),
                            time: Time::now(),
                        },
                    );

                    window.shared.surface.window().request_redraw();
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

                window.shared.surface.resize(&self.cx);
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
        let Some(window) = self.windows.get_mut(&id) else {
            return;
        };

        match event {
            event::WindowEvent::Resized(size) => {
                let (width, height): (u32, u32) = size.into();
                log::debug!("resized {id:?}: {width} {height}");

                window.shared.surface.resize(&self.cx);
                window.shared.resize.set();
                self.schedule();
            }
            event::WindowEvent::CloseRequested => {
                log::debug!("close requested {id:?}");
                window.shared.close.set();
                self.schedule();
            }
            event::WindowEvent::Focused(focused) => {
                if focused {
                    log::debug!("focused {id:?}");
                    window.shared.surface.window().request_redraw();
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
                    event::ElementState::Pressed => window.shared.press_keys.active(code),
                    event::ElementState::Released => window.shared.release_keys.active(code),
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

                let delta_time = window.time.delta();
                let min_delta_time = window.shared.min_delta_time.get();
                if delta_time < min_delta_time {
                    let wait = min_delta_time - delta_time;
                    el.set_control_flow(event_loop::ControlFlow::wait_duration(wait));
                    return;
                }

                window.time.reset();
                window.shared.redraw.set_value(delta_time);
                self.schedule();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &event_loop::ActiveEventLoop) {
        self.active_poll(el);
    }
}

#[cfg(target_family = "wasm")]
#[inline]
pub async fn run<F, R>(mut f: F) -> Result<R, Error>
where
    F: AsyncFnMut(Context, Control) -> R + 'static,
    R: 'static,
{
    use winit::platform::web::EventLoopExtWebSys;

    let el = event_loop::EventLoop::with_user_event()
        .build()
        .map_err(Error::EventLoop)?;

    let lifecycle = Rc::new(Lifecycle {
        state: Cell::new(LifecycleState::Inactive),
    });

    let control = Control {
        proxy: el.create_proxy(),
        lifecycle: lifecycle.clone(),
    };

    let cx = dunge::context().await.map_err(Error::Context)?;

    let (out, take) = async_channel::bounded(1);
    let app = App {
        cx: cx.clone(),
        proxy: el.create_proxy(),
        lifecycle,
        windows: HashMap::new(),
        need_to_poll: false,
        context: task::Context::from_waker(Waker::noop()),
        fu: Box::pin(async move { f(cx, control).await }),
        out,
    };

    el.spawn_app(app);
    take.recv().await.expect("take result of async function")
}

#[cfg(target_family = "wasm")]
#[inline]
pub async fn try_run<F, R, U>(f: F) -> Result<R, Error<U>>
where
    F: AsyncFnMut(Context, Control) -> Result<R, U> + 'static,
    R: 'static,
    U: 'static,
{
    match run(f).await {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(u)) => Err(Error::Custom(u)),
        Err(e) => Err(e.map(|never| match never {})),
    }
}

#[cfg(not(target_family = "wasm"))]
#[inline]
pub fn block_on<F, R>(mut f: F) -> Result<R, Error>
where
    F: AsyncFnMut(Context, Control) -> R,
{
    let el = event_loop::EventLoop::with_user_event()
        .build()
        .map_err(Error::EventLoop)?;

    let lifecycle = Rc::new(Lifecycle {
        state: Cell::new(LifecycleState::Inactive),
    });

    let control = Control {
        proxy: el.create_proxy(),
        lifecycle: lifecycle.clone(),
    };

    let cx = dunge::block_on(dunge::context()).map_err(Error::Context)?;

    struct AppWaker {
        // it doesn't work :(
        // > cannot be sent between threads safely
        // so I need either sendable proxy from winit
        // or local wakers in std.
        #[cfg(any())]
        proxy: event_loop::EventLoopProxy<Request>,
    }

    impl task::Wake for AppWaker {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            #[cfg(any())]
            {
                _ = self.proxy.send_event(Request::Wake);
            }
        }
    }

    let waker = Waker::from(Arc::new(AppWaker {}));

    let (out, take) = async_channel::bounded(1);
    let mut app = App {
        cx: cx.clone(),
        proxy: el.create_proxy(),
        lifecycle,
        windows: HashMap::new(),
        need_to_poll: false,
        context: task::Context::from_waker(&waker),
        fu: Box::pin(f(cx, control)),
        out,
    };

    el.run_app(&mut app).map_err(Error::EventLoop)?;
    take.recv_blocking().expect("take result of async function")
}

#[cfg(not(target_family = "wasm"))]
#[inline]
pub fn try_block_on<F, R, U>(f: F) -> Result<R, Error<U>>
where
    F: AsyncFnMut(Context, Control) -> Result<R, U>,
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

pub struct Control {
    proxy: event_loop::EventLoopProxy<Request>,
    lifecycle: Rc<Lifecycle>,
}

impl Control {
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

pub struct Attributes {
    title: Cow<'static, str>,
    canvas: Option<Canvas>,
}

impl Attributes {
    #[inline]
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Cow::Owned(title.into());
        self
    }

    #[inline]
    pub fn with_canvas<C>(mut self, canvas: C) -> Self
    where
        C: Into<Option<Canvas>>,
    {
        self.canvas = canvas.into();
        self
    }

    #[inline]
    fn winit(mut self) -> window::WindowAttributes {
        let mut attr = window::WindowAttributes::default().with_title(self.title);
        if let Some(canvas) = self.canvas.take() {
            attr = canvas.set(attr);
        }

        attr
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            title: Cow::Borrowed("dunge"),
            canvas: None,
        }
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

struct Shared {
    surface: Surface<window::Window, Ops>,
    min_delta_time: Cell<Duration>,
    press_keys: Keys,
    release_keys: Keys,
    resize: Event,
    redraw: Event<Option<Duration>>,
    close: Event,
}

#[derive(Clone)]
pub struct Window {
    shared: Rc<Shared>,
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
        let shared = Rc::new(Shared {
            min_delta_time: Cell::new(Duration::from_secs_f32(1. / Self::DEFAULT_FPS as f32)),
            surface: Surface::new(cx, window).map_err(Error::CreateSurface)?,
            press_keys: Keys::new(),
            release_keys: Keys::new(),
            resize: Event::new(),
            redraw: Event::new(),
            close: Event::new(),
        });

        Ok(Self { shared, proxy })
    }

    #[inline]
    pub fn winit(&self) -> &Arc<window::Window> {
        self.shared.surface.window()
    }

    #[inline]
    pub fn format(&self) -> Format {
        self.shared.surface.format()
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.shared.surface.size()
    }

    #[inline]
    pub fn set_fps(&self, fps: NonZeroU32) {
        self.shared
            .min_delta_time
            .set(Duration::from_secs_f32(1. / fps.get() as f32));
    }

    #[inline]
    pub async fn pressed(&self, code: keyboard::KeyCode) {
        self.shared.press_keys.wait(code);
        future::poll_fn(|_| self.shared.press_keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn released(&self, code: keyboard::KeyCode) {
        self.shared.release_keys.wait(code);
        future::poll_fn(|_| self.shared.release_keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn resized(&self) -> (u32, u32) {
        future::poll_fn(|_| self.shared.resize.active_poll()).await;
        self.shared.surface.size()
    }

    #[inline]
    pub async fn redraw(&self) -> Redraw<'_> {
        loop {
            let delta_time = future::poll_fn(|_| self.shared.redraw.active_poll_value()).await;
            let e = match self.shared.surface.output() {
                Ok(output) => break Redraw { output, delta_time },
                Err(e) => e,
            };

            log::warn!("surface error: {e}");
            match e.action() {
                Action::Run => {}
                Action::Recreate => {
                    let id = self.shared.surface.window().id();
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
        future::poll_fn(|_| self.shared.close.active_poll()).await;
    }
}

impl Drop for Window {
    #[inline]
    fn drop(&mut self) {
        let id = self.shared.surface.window().id();
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
