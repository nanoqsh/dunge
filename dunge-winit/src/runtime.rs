use {
    crate::{
        reactor::{Process, Reactor, Timer},
        window::{Attributes, Shared, Window, WindowBuilder},
    },
    dunge::{
        Context, FailedMakeContext, buffer, mesh,
        surface::{CreateSurfaceError, SurfaceError},
    },
    futures_lite::Stream,
    std::{
        cell::{Cell, OnceCell, RefCell},
        collections::HashMap,
        convert::Infallible,
        error, fmt, future,
        pin::Pin,
        rc::Rc,
        task::{self, Poll, Waker},
        time::Duration,
    },
    winit::{application::ApplicationHandler, dpi, event, event_loop, keyboard, window},
};

#[cfg(target_family = "wasm")]
use web_time::Instant;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

enum Message {
    MakeWindow {
        cx: Context,
        attr: Box<window::WindowAttributes>,
        out: Rc<OnceCell<Result<Window, Error>>>,
    },
    RemoveWindow(window::WindowId),
    RecreateSurface(window::WindowId),
    SetFps(window::WindowId, Duration),
    Exit(Box<Error>),
}

#[derive(Clone, Debug)]
pub(crate) struct Request(event_loop::EventLoopProxy<Message>);

impl Request {
    #[inline]
    pub(crate) async fn make_window(&self, cx: Context, attr: Attributes) -> Result<Window, Error> {
        let mut out = Rc::new(OnceCell::new());
        _ = self.0.send_event(Message::MakeWindow {
            cx,
            attr: attr.winit(),
            out: out.clone(),
        });

        let mut active_poll = || {
            Rc::get_mut(&mut out).map_or(Poll::Pending, |out| {
                Poll::Ready(out.take().expect("take window"))
            })
        };

        future::poll_fn(|_| active_poll()).await
    }

    #[inline]
    pub(crate) fn remove_window(&self, id: window::WindowId) {
        _ = self.0.send_event(Message::RemoveWindow(id));
    }

    #[inline]
    pub(crate) fn recreate_surface(&self, id: window::WindowId) {
        _ = self.0.send_event(Message::RecreateSurface(id));
    }

    #[inline]
    pub(crate) fn set_fps(&self, id: window::WindowId, period: Duration) {
        _ = self.0.send_event(Message::SetFps(id, period));
    }

    #[inline]
    pub(crate) fn exit(&self, e: Error) {
        _ = self.0.send_event(Message::Exit(Box::new(e)));
    }
}

struct AppWindow {
    shared: Rc<Shared>,
    timer: Timer,
    last_render: Instant,
    set_fps: Option<Duration>,
}

enum Out<R> {
    Empty,
    Done(R),
    Fail(Error),
}

struct Return<R> {
    out: Cell<Out<R>>,
    waker: RefCell<Option<Waker>>,
}

impl<R> Return<R> {
    const fn new() -> Self {
        Self {
            out: Cell::new(Out::Empty),
            waker: RefCell::new(None),
        }
    }

    #[cold]
    fn set(&self, res: Result<R, Error>) {
        let out = match res {
            Ok(r) => Out::Done(r),
            Err(e) => Out::Fail(e),
        };

        self.out.set(out);
        if let Some(waker) = self.waker.borrow().as_ref() {
            waker.wake_by_ref();
        }
    }

    fn try_poll(&self) -> Poll<Result<R, Error>> {
        match self.out.replace(Out::Empty) {
            Out::Empty => Poll::Pending,
            Out::Done(r) => Poll::Ready(Ok(r)),
            Out::Fail(e) => Poll::Ready(Err(e)),
        }
    }

    #[cfg(target_family = "wasm")]
    fn poll(&self, cx: &mut task::Context<'_>) -> Poll<Result<R, Error>> {
        let poll = self.try_poll();
        if poll.is_pending() {
            if let Some(waker) = self.waker.borrow_mut().as_mut() {
                waker.clone_from(cx.waker());
            }
        }

        poll
    }
}

enum Action {
    Tick,
    Process,
}

struct App<F, R> {
    req: Request,
    lifecycle: Rc<Lifecycle>,
    windows: HashMap<window::WindowId, AppWindow>,
    action: Action,
    context: task::Context<'static>,
    fu: Pin<Box<F>>,
    ret: Rc<Return<R>>,
}

impl<F> App<F, F::Output>
where
    F: Future,
{
    const DEFAULT_SLEEP_DURATION: Duration = Duration::from_millis(100);

    #[inline]
    fn process_timers(&mut self, el: &event_loop::ActiveEventLoop) {
        let next = 'process: {
            for _ in 0..4 {
                match Reactor::get().process_timers() {
                    // ready to make progress
                    Process::Ready => self.poll(el),
                    // wait for next timer
                    Process::Wait(next) => break 'process next,
                    // nothing to do, sleep some time
                    Process::Sleep => break,
                }
            }

            Instant::now() + Self::DEFAULT_SLEEP_DURATION
        };

        el.set_control_flow(event_loop::ControlFlow::WaitUntil(next));
    }

    fn need_redraw(&mut self) -> bool {
        if let LifecycleState::Inactive = self.lifecycle.state.get() {
            return false;
        }

        let mut redraw = false;
        for window in self.windows.values_mut() {
            if Pin::new(&mut window.timer)
                .poll_next(&mut self.context)
                .is_ready()
            {
                window.shared.window().request_redraw();
                redraw = true;
            }
        }

        redraw
    }

    #[inline]
    fn poll(&mut self, el: &event_loop::ActiveEventLoop) {
        if let Poll::Ready(res) = self.fu.as_mut().poll(&mut self.context) {
            self.ret.set(Ok(res));
            el.exit();
        }
    }

    #[inline]
    fn exit_with_error(&mut self, el: &event_loop::ActiveEventLoop, e: Error) {
        self.ret.set(Err(e));
        el.exit();
    }
}

impl<F> ApplicationHandler<Message> for App<F, F::Output>
where
    F: Future,
{
    #[inline]
    fn new_events(&mut self, el: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        self.action = match cause {
            event::StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");
                Action::Process
            }
            event::StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                log::debug!("wait cancelled");

                match requested_resume {
                    Some(resume) => {
                        el.set_control_flow(event_loop::ControlFlow::WaitUntil(resume));
                        Action::Tick
                    }
                    None => Action::Process,
                }
            }
            event::StartCause::Poll => {
                log::debug!("poll");
                Action::Process
            }
            event::StartCause::Init => {
                log::debug!("init");

                // an initial poll
                self.poll(el);

                Action::Process
            }
        }
    }

    #[inline]
    fn resumed(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("resumed");
        self.lifecycle.set(LifecycleState::Active);

        for window in self.windows.values() {
            window.shared.window().request_redraw();
        }
    }

    #[inline]
    fn suspended(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("suspended");
        self.lifecycle.set(LifecycleState::Inactive);
    }

    #[inline]
    fn user_event(&mut self, el: &event_loop::ActiveEventLoop, req: Message) {
        match req {
            Message::MakeWindow { cx, attr, out } => {
                log::debug!("make window");
                let res = Window::new(cx, self.req.clone(), el, attr);
                if let Ok(window) = &res {
                    let shared = window.shared().clone();
                    let id = shared.window().id();

                    shared.window().request_redraw();
                    self.windows.insert(
                        id,
                        AppWindow {
                            shared,
                            timer: Timer::interval(Duration::from_secs_f32(1. / 60.)),
                            last_render: Instant::now(),
                            set_fps: None,
                        },
                    );
                }

                _ = out.set(res);
            }
            Message::RemoveWindow(id) => {
                log::debug!("remove window {id:?}");
                _ = self.windows.remove(&id);
            }
            Message::RecreateSurface(id) => {
                log::debug!("recreate surface {id:?}");
                let Some(window) = self.windows.get(&id) else {
                    return;
                };

                window.shared.resize();
            }
            Message::SetFps(id, period) => {
                log::debug!("set fps {id:?}");
                let Some(window) = self.windows.get_mut(&id) else {
                    return;
                };

                window.set_fps = Some(period);
            }
            Message::Exit(e) => {
                log::debug!("exit with error: {e}");
                self.exit_with_error(el, *e);
            }
        }
    }

    #[inline]
    fn window_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
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

                window.shared.resize();
                window.shared.events().resize.set();
                window.shared.window().request_redraw();
            }
            event::WindowEvent::CloseRequested => {
                log::debug!("close requested {id:?}");
                window.shared.events().close.set();
            }
            event::WindowEvent::Focused(focused) => {
                if focused {
                    log::debug!("focused {id:?}");
                } else {
                    log::debug!("unfocused {id:?}");
                }
            }
            event::WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        physical_key,
                        text,
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

                let events = window.shared.events();
                match state {
                    event::ElementState::Pressed => {
                        events.press_keys.active(code);

                        if let Some(s) = text {
                            events.text.active(s);
                        }
                    }
                    event::ElementState::Released => events.release_keys.active(code),
                }
            }
            event::WindowEvent::CursorMoved {
                position: dpi::PhysicalPosition { x, y },
                ..
            } => window.shared.cursor_moved(x, y),
            event::WindowEvent::CursorLeft { .. } => window.shared.cursor_left(),
            event::WindowEvent::MouseWheel { .. } => {}
            event::WindowEvent::MouseInput { state, button, .. } => {
                let events = window.shared.events();
                match state {
                    event::ElementState::Pressed => events.press_buttons.active(button),
                    event::ElementState::Released => events.release_buttons.active(button),
                }
            }
            event::WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(window.last_render);
                window.last_render = now;
                window.shared.events().redraw.add_value(delta_time);

                if let Some(period) = window.set_fps.take() {
                    window.timer = Timer::interval(period);
                }
            }
            _ => {}
        }
    }

    #[inline]
    fn about_to_wait(&mut self, el: &event_loop::ActiveEventLoop) {
        if self.need_redraw() {
            // poll on redraw event
            el.set_control_flow(event_loop::ControlFlow::Poll);
            return;
        }

        self.poll(el);
        if let Action::Process = self.action {
            self.process_timers(el);
        }
    }
}

/// Runs an event loop on web.
#[cfg(target_family = "wasm")]
#[inline]
pub async fn run<F, R>(f: F) -> Result<R, Error>
where
    F: AsyncFnOnce(Control) -> R + 'static,
    R: 'static,
{
    use {futures_lite::future, winit::platform::web::EventLoopExtWebSys};

    let el = event_loop::EventLoop::with_user_event()
        .build()
        .map_err(Error::EventLoop)?;

    let req = Request(el.create_proxy());

    let lifecycle = Rc::new(Lifecycle {
        state: Cell::new(LifecycleState::Inactive),
    });

    let control = Control {
        req: req.clone(),
        lifecycle: lifecycle.clone(),
    };

    let ret = Rc::new(Return::new());
    let app = App {
        req,
        lifecycle,
        windows: HashMap::new(),
        action: Action::Process,
        context: task::Context::from_waker(Waker::noop()),
        fu: Box::pin(future::fuse(f(control))),
        ret: ret.clone(),
    };

    el.spawn_app(app);
    future::poll_fn(|cx| ret.poll(cx)).await
}

/// Same as [`run`], but allows returning a custom error type.
#[cfg(target_family = "wasm")]
#[inline]
pub async fn try_run<F, R, U>(f: F) -> Result<R, Error<U>>
where
    F: AsyncFnOnce(Control) -> Result<R, U> + 'static,
    R: 'static,
    U: 'static,
{
    match run(f).await {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(u)) => Err(Error::Custom(u)),
        Err(e) => Err(e.map(|never| match never {})),
    }
}

/// Runs the event loop on the current thread on desktop platform.
#[cfg(not(target_family = "wasm"))]
#[inline]
pub fn block_on<F, R>(f: F) -> Result<R, Error>
where
    F: AsyncFnOnce(Control) -> R,
{
    use futures_lite::future;

    let el = event_loop::EventLoop::with_user_event()
        .build()
        .map_err(Error::EventLoop)?;

    let req = Request(el.create_proxy());

    let lifecycle = Rc::new(Lifecycle {
        state: Cell::new(LifecycleState::Inactive),
    });

    let control = Control {
        req: req.clone(),
        lifecycle: lifecycle.clone(),
    };

    let ret = Rc::new(Return::new());
    let mut app = App {
        req,
        lifecycle,
        windows: HashMap::new(),
        action: Action::Process,
        context: task::Context::from_waker(Waker::noop()),
        fu: Box::pin(future::fuse(f(control))),
        ret: ret.clone(),
    };

    el.run_app(&mut app).map_err(Error::EventLoop)?;
    let Poll::Ready(res) = ret.try_poll() else {
        unreachable!();
    };

    res
}

/// Same as [`block_on`], but allows returning a custom error type.
#[cfg(not(target_family = "wasm"))]
#[inline]
pub fn try_block_on<F, R, U>(f: F) -> Result<R, Error<U>>
where
    F: AsyncFnOnce(Control) -> Result<R, U>,
{
    match block_on(f) {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(u)) => Err(Error::Custom(u)),
        Err(e) => Err(e.map(|never| match never {})),
    }
}

/// The event loop error type.
#[derive(Debug)]
pub enum Error<U = Infallible> {
    Context(FailedMakeContext),
    Mesh(mesh::Error),
    Size(buffer::ZeroSized),
    Texture(buffer::InvalidLen),
    EventLoop(winit::error::EventLoopError),
    Os(winit::error::OsError),
    CreateSurface(CreateSurfaceError),
    Surface(SurfaceError),
    Custom(U),
}

impl<U> Error<U> {
    /// Transforms the custom error type.
    pub fn map<F, V>(self, f: F) -> Error<V>
    where
        F: FnOnce(U) -> V,
    {
        match self {
            Self::Context(e) => Error::Context(e),
            Self::Mesh(e) => Error::Mesh(e),
            Self::Size(e) => Error::Size(e),
            Self::Texture(e) => Error::Texture(e),
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
            Self::Mesh(e) => e.fmt(f),
            Self::Size(e) => e.fmt(f),
            Self::Texture(e) => e.fmt(f),
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
            Self::Mesh(e) => Some(e),
            Self::Size(e) => Some(e),
            Self::Texture(e) => Some(e),
            Self::EventLoop(e) => Some(e),
            Self::Os(e) => Some(e),
            Self::CreateSurface(e) => Some(e),
            Self::Surface(e) => Some(e),
            Self::Custom(e) => Some(e),
        }
    }
}

impl<U> From<FailedMakeContext> for Error<U> {
    fn from(e: FailedMakeContext) -> Self {
        Self::Context(e)
    }
}

impl<U> From<mesh::Error> for Error<U> {
    fn from(e: mesh::Error) -> Self {
        Self::Mesh(e)
    }
}

impl<U> From<buffer::ZeroSized> for Error<U> {
    fn from(e: buffer::ZeroSized) -> Self {
        Self::Size(e)
    }
}

impl<U> From<buffer::InvalidLen> for Error<U> {
    fn from(e: buffer::InvalidLen) -> Self {
        Self::Texture(e)
    }
}

#[derive(Clone, Copy, Debug)]
enum LifecycleState {
    Active,
    Inactive,
}

#[derive(Debug)]
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

/// Passed to the user function to control and interact with the event loop.
#[derive(Clone, Debug)]
pub struct Control {
    req: Request,
    lifecycle: Rc<Lifecycle>,
}

impl Control {
    /// Creates a new [window](Window) with specified attributes
    /// configured by a [builder](WindowBuilder).
    ///
    /// # Example
    ///
    /// ```
    /// use dunge_winit::prelude::*;
    ///
    /// # async fn t(control: Control) -> Result<(), Box<dyn std::error::Error>> {
    /// let cx = dunge::context().await?;
    /// let window = control
    ///     .make_window(&cx)
    ///     .with_title("cube")
    ///     .await?;
    ///
    /// // wait for some window events
    /// loop {
    ///     let new_size = window.resized().await;
    ///     println!("resized: {new_size}");
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn make_window(&self, cx: &Context) -> WindowBuilder<'_> {
        WindowBuilder::new(&self.req, cx.clone())
    }

    /// Waits until the application is resumed.
    #[inline]
    pub async fn resumed(&self) {
        future::poll_fn(|_| self.lifecycle.active_poll_resumed()).await;
    }

    /// Waits until the application is suspended.
    #[inline]
    pub async fn suspended(&self) {
        future::poll_fn(|_| self.lifecycle.active_poll_suspended()).await;
    }
}
