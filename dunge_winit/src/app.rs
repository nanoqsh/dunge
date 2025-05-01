use {
    crate::{
        time::Time,
        window::{Attributes, Shared, Window},
    },
    dunge::{
        Context, FailedMakeContext,
        surface::{CreateSurfaceError, SurfaceError},
    },
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
    winit::{application::ApplicationHandler, event, event_loop, keyboard, window},
};

enum Message {
    Wake,
    MakeWindow {
        out: Rc<OnceCell<Result<Window, Error>>>,
        attr: Box<Attributes>,
    },
    RemoveWindow(window::WindowId),
    RecreateSurface(window::WindowId),
    Exit(Box<Error>),
}

#[derive(Clone)]
pub(crate) struct Request(event_loop::EventLoopProxy<Message>);

impl Request {
    #[expect(dead_code)]
    #[inline]
    fn wake(&self) {
        _ = self.0.send_event(Message::Wake);
    }

    #[inline]
    async fn make_window(&self, attr: Attributes) -> Result<Window, Error> {
        let mut out = Rc::new(OnceCell::new());
        _ = self.0.send_event(Message::MakeWindow {
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

    #[inline]
    pub(crate) fn remove_window(&self, id: window::WindowId) {
        _ = self.0.send_event(Message::RemoveWindow(id));
    }

    #[inline]
    pub(crate) fn recreate_surface(&self, id: window::WindowId) {
        _ = self.0.send_event(Message::RecreateSurface(id));
    }

    #[inline]
    pub(crate) fn exit(&self, e: Error) {
        _ = self.0.send_event(Message::Exit(Box::new(e)));
    }
}

struct AppWindow {
    shared: Rc<Shared>,
    time: Time,
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

    fn set(&self, res: Result<R, Error>) {
        let out = match res {
            Ok(r) => Out::Done(r),
            Err(e) => Out::Fail(e),
        };

        self.out.set(out);
        if let Some(waker) = &*self.waker.borrow() {
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
            if let Some(waker) = &mut *self.waker.borrow_mut() {
                waker.clone_from(cx.waker());
            }
        }

        poll
    }
}

struct App<'waker, F, R> {
    cx: Context,
    req: Request,
    lifecycle: Rc<Lifecycle>,
    windows: HashMap<window::WindowId, AppWindow>,
    need_to_poll: bool,
    context: task::Context<'waker>,
    fu: Pin<Box<F>>,
    ret: Rc<Return<R>>,
}

impl<F> App<'_, F, F::Output>
where
    F: Future,
{
    const WAIT_TIME: Duration = Duration::from_millis(100);

    #[inline]
    fn schedule(&mut self) {
        self.need_to_poll = true;
    }

    #[inline]
    fn active_poll(&mut self, el: &event_loop::ActiveEventLoop) {
        if !self.need_to_poll {
            return;
        }

        self.force_poll(el);
        self.need_to_poll = false;
    }

    #[inline]
    fn force_poll(&mut self, el: &event_loop::ActiveEventLoop) {
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

impl<F> ApplicationHandler<Message> for App<'_, F, F::Output>
where
    F: Future,
{
    fn new_events(&mut self, el: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        match cause {
            event::StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");
                for window in self.windows.values() {
                    window.shared.window().request_redraw();
                }

                self.schedule();
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
                self.force_poll(el);
            }
        }
    }

    fn resumed(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("resumed");
        self.lifecycle.set(LifecycleState::Active);

        for window in self.windows.values() {
            window.shared.window().request_redraw();
        }

        self.schedule();
    }

    fn suspended(&mut self, _: &event_loop::ActiveEventLoop) {
        log::debug!("suspended");
        self.lifecycle.set(LifecycleState::Inactive);
        self.schedule();
    }

    fn user_event(&mut self, el: &event_loop::ActiveEventLoop, req: Message) {
        match req {
            Message::Wake => self.force_poll(el),
            Message::MakeWindow { out, attr } => {
                log::debug!("make window");
                let res = Window::new(&self.cx, self.req.clone(), el, *attr);
                if let Ok(window) = &res {
                    let shared = window.shared().clone();
                    let id = shared.window().id();

                    shared.window().request_redraw();
                    self.windows.insert(
                        id,
                        AppWindow {
                            shared,
                            time: Time::now(),
                        },
                    );
                }

                _ = out.set(res);
                self.schedule();
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

                window.shared.resize(&self.cx);
                self.schedule();
            }
            Message::Exit(e) => {
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

                window.shared.resize(&self.cx);
                window.shared.events().resize.set();
                self.schedule();
            }
            event::WindowEvent::CloseRequested => {
                log::debug!("close requested {id:?}");
                window.shared.events().close.set();
                self.schedule();
            }
            event::WindowEvent::Focused(focused) => {
                if focused {
                    log::debug!("focused {id:?}");
                    window.shared.window().request_redraw();
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

                let events = window.shared.events();
                match state {
                    event::ElementState::Pressed => events.press_keys.active(code),
                    event::ElementState::Released => events.release_keys.active(code),
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
                let min_delta_time = window.shared.min_delta_time();
                if delta_time < min_delta_time {
                    let wait = min_delta_time - delta_time;
                    el.set_control_flow(event_loop::ControlFlow::wait_duration(wait));
                    return;
                }

                window.time.reset();
                window.shared.events().redraw.set_value(delta_time);
                self.force_poll(el);
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

    let req = Request(el.create_proxy());

    let lifecycle = Rc::new(Lifecycle {
        state: Cell::new(LifecycleState::Inactive),
    });

    let control = Control {
        req: req.clone(),
        lifecycle: lifecycle.clone(),
    };

    let cx = dunge::context().await.map_err(Error::Context)?;

    let ret = Rc::new(Return::new());
    let app = App {
        cx: cx.clone(),
        req,
        lifecycle,
        windows: HashMap::new(),
        need_to_poll: false,
        context: task::Context::from_waker(Waker::noop()),
        fu: Box::pin(async move { f(cx, control).await }),
        ret: ret.clone(),
    };

    el.spawn_app(app);
    future::poll_fn(|cx| ret.poll(cx)).await
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
    use std::sync::Arc;

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

    let cx = dunge::block_on(dunge::context()).map_err(Error::Context)?;

    struct AppWaker {
        // it doesn't work :(
        // > cannot be sent between threads safely
        // so I need either sendable proxy from winit
        // or local wakers in std.
        #[cfg(any())]
        proxy: event_loop::EventLoopProxy<Message>,
    }

    impl task::Wake for AppWaker {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            #[cfg(any())]
            {
                _ = self.proxy.send_event(Message::Wake);
            }
        }
    }

    let waker = Waker::from(Arc::new(AppWaker {}));

    let ret = Rc::new(Return::new());
    let mut app = App {
        cx: cx.clone(),
        req,
        lifecycle,
        windows: HashMap::new(),
        need_to_poll: false,
        context: task::Context::from_waker(&waker),
        fu: Box::pin(f(cx, control)),
        ret: ret.clone(),
    };

    el.run_app(&mut app).map_err(Error::EventLoop)?;
    let Poll::Ready(res) = ret.try_poll() else {
        unreachable!();
    };

    res
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
    req: Request,
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
        self.req.make_window(attr).await
    }
}
