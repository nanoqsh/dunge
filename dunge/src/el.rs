use {
    crate::{
        _window::{self, View, WindowState},
        context::Context,
        state::State,
        time::{Fps, Time},
        update::{IntoUpdate, Update},
    },
    std::{cell::Cell, error, fmt, ops, time::Duration},
    winit::{
        application::ApplicationHandler,
        error::EventLoopError,
        event::{self, StartCause, WindowEvent},
        event_loop::{self, ActiveEventLoop, ControlFlow},
        keyboard,
        window::WindowId,
    },
};

/// Code representing the location of a physical key.
pub type KeyCode = keyboard::KeyCode;

/// String type from `winit` crate.
pub type SmolStr = keyboard::SmolStr;

/// Describes a button of a mouse controller.
pub type MouseButton = event::MouseButton;

pub(crate) fn run<U>(ws: WindowState<U::Event>, cx: Context, upd: U) -> Result<(), LoopError>
where
    U: IntoUpdate + 'static,
{
    #[cfg(not(target_family = "wasm"))]
    {
        run_local(ws, cx, upd)
    }

    #[cfg(target_family = "wasm")]
    {
        spawn(ws, cx, upd)
    }
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn run_local<U>(ws: WindowState<U::Event>, cx: Context, upd: U) -> Result<(), LoopError>
where
    U: IntoUpdate,
{
    let (view, lu) = ws.into_view_and_loop();
    let mut handler = Handler::new(cx, view, upd);
    let out = lu.run_app(&mut handler).map_err(LoopError::EventLoop);
    out.or(handler.out)
}

#[cfg(target_family = "wasm")]
fn spawn<U>(ws: WindowState<U::Event>, cx: Context, upd: U) -> Result<(), LoopError>
where
    U: IntoUpdate + 'static,
{
    use winit::platform::web::EventLoopExtWebSys;

    let (view, lu) = ws.into_view_and_loop();
    let handler = Handler::new(cx, view, upd);
    lu.spawn_app(handler);
    Ok(())
}

/// The event loop error.
#[derive(Debug)]
pub enum LoopError {
    Window(_window::Error),
    EventLoop(EventLoopError),
    Failed(Box<dyn error::Error>),
}

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(e) => e.fmt(f),
            Self::EventLoop(e) => e.fmt(f),
            Self::Failed(e) => e.fmt(f),
        }
    }
}

impl error::Error for LoopError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Window(e) => Some(e),
            Self::EventLoop(e) => Some(e),
            Self::Failed(e) => Some(e.as_ref()),
        }
    }
}

enum Deferred<U>
where
    U: IntoUpdate,
{
    Empty,
    Uninit(U),
    Init(U::Update),
}

impl<U> Deferred<U>
where
    U: IntoUpdate,
{
    fn init(&mut self, cx: &Context, view: &View) {
        use std::mem;

        let upd = match mem::replace(self, Self::Empty) {
            Self::Empty => unreachable!(),
            Self::Uninit(into_upd) => into_upd.into_update(cx, view),
            Self::Init(upd) => upd,
        };

        *self = Self::Init(upd);
    }

    fn get(&mut self) -> &mut U::Update {
        match self {
            Self::Init(upd) => upd,
            _ => panic!("the handler must be initialized"),
        }
    }
}

struct Handler<U>
where
    U: IntoUpdate,
{
    cx: Context,
    ctrl: Control,
    upd: Deferred<U>,
    active: bool,
    time: Time,
    fps: Fps,
    out: Result<(), LoopError>,
}

impl<U> Handler<U>
where
    U: IntoUpdate,
{
    const WAIT_TIME: Duration = Duration::from_millis(100);

    fn new(cx: Context, view: View, into_upd: U) -> Self {
        let ctrl = Control {
            view,
            resized: None,
            min_delta_time: Cell::new(Duration::from_secs_f32(1. / 60.)),
            delta_time: Duration::ZERO,
            fps: 0,
            pressed_keys: vec![],
            released_keys: vec![],
            cursor_position: None,
            mouse: Mouse {
                wheel_delta: (0., 0.),
                pressed_buttons: Buttons(vec![]),
                released_buttons: Buttons(vec![]),
            },
        };

        Self {
            cx,
            ctrl,
            upd: Deferred::Uninit(into_upd),
            active: false,
            time: Time::now(),
            fps: Fps::default(),
            out: Ok(()),
        }
    }
}

impl<U> ApplicationHandler<U::Event> for Handler<U>
where
    U: IntoUpdate,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        log::debug!("resumed");
        self.active = true;
        self.ctrl.view.request_redraw();
        el.set_control_flow(ControlFlow::wait_duration(Self::WAIT_TIME));

        // Reset the timer before start the loop
        self.time.reset();
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        log::debug!("suspended");
        self.active = false;
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        use {
            event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
            event_loop::ControlFlow,
            keyboard::PhysicalKey,
            winit::dpi::{PhysicalPosition, PhysicalSize},
        };

        if id != self.ctrl.view.id() {
            return;
        }

        match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                log::debug!("resized: {width}, {height}");
                self.ctrl.resize(self.cx.state());
            }
            WindowEvent::CloseRequested => {
                log::debug!("close requested");
                el.exit();
            }
            WindowEvent::Focused(true) => {
                log::debug!("focused");
                self.ctrl.view.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        text,
                        location,
                        state,
                        ..
                    },
                is_synthetic: false,
                ..
            } => {
                let code = match physical_key {
                    PhysicalKey::Code(code) => {
                        log::debug!("keyboard input: {code:?}");
                        code
                    }
                    PhysicalKey::Unidentified(code) => {
                        log::debug!("keyboard input: (unidentified) {code:?}");
                        return;
                    }
                };

                // TODO: Support key location
                _ = location;

                let key = Key { code, text };
                match state {
                    ElementState::Pressed => self.ctrl.pressed_keys.push(key),
                    ElementState::Released => self.ctrl.released_keys.push(key),
                }
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => self.ctrl.cursor_position = Some((x as f32, y as f32)),
            WindowEvent::CursorLeft { .. } => self.ctrl.cursor_position = None,
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.ctrl.mouse.wheel_delta.0 += x;
                self.ctrl.mouse.wheel_delta.1 += y;
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.ctrl.mouse.pressed_buttons.push(button),
                ElementState::Released => self.ctrl.mouse.released_buttons.push(button),
            },
            WindowEvent::RedrawRequested => {
                if self.active {
                    log::debug!("redraw requested");
                } else {
                    log::debug!("redraw requested (non-active)");

                    // Wait a while to become active
                    el.set_control_flow(ControlFlow::wait_duration(Self::WAIT_TIME));
                    return;
                }

                let delta_time = self.time.delta();
                let min_delta_time = self.ctrl.min_delta_time.get();
                if delta_time < min_delta_time {
                    let wait = min_delta_time - delta_time;
                    el.set_control_flow(ControlFlow::wait_duration(wait));
                    return;
                }

                self.time.reset();
                self.ctrl.delta_time = delta_time;
                if let Some(fps) = self.fps.count(delta_time) {
                    self.ctrl.fps = fps;
                }

                let upd = self.upd.get();
                match upd.update(&self.ctrl).flow() {
                    Then::Run => {}
                    Then::Close => {
                        log::debug!("close");
                        el.exit();
                        return;
                    }
                    Then::Fail(e) => {
                        log::error!("failed: {e:?}");
                        self.out = Err(LoopError::Failed(e));
                        el.exit();
                        return;
                    }
                }

                self.ctrl.clear_state();
                match self.ctrl.view.output() {
                    Ok(output) => {
                        let target = output.target();
                        self.cx.state()._draw(target, &*upd);
                        output.present();
                    }
                    Err(wgpu::SurfaceError::Timeout) => log::info!("suface error: timeout"),
                    Err(wgpu::SurfaceError::Outdated) => log::info!("suface error: outdated"),
                    Err(wgpu::SurfaceError::Lost) => {
                        log::info!("suface error: lost");
                        self.ctrl.resize(self.cx.state());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("suface error: out of memory");
                        el.exit();
                    }
                    Err(wgpu::SurfaceError::Other) => {
                        log::error!("suface error: other error");
                        el.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn new_events(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");
                self.ctrl.view.set_window_size();
                self.ctrl.view.request_redraw();
            }
            StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                log::debug!("wait cancelled");
                let flow = match requested_resume {
                    Some(resume) => ControlFlow::WaitUntil(resume),
                    None => ControlFlow::wait_duration(Self::WAIT_TIME),
                };

                el.set_control_flow(flow);
            }
            StartCause::Poll => log::debug!("poll"),
            StartCause::Init => {
                log::debug!("init");
                self.out = self
                    .ctrl
                    .view
                    .init(self.cx.state(), el)
                    .map_err(LoopError::Window);

                if self.out.is_err() {
                    el.exit();
                }

                self.upd.init(&self.cx, &self.ctrl.view);
            }
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, ev: U::Event) {
        self.upd.get().event(ev);
    }
}

/// The control type of the main event loop.
pub struct Control {
    view: View,
    resized: Option<(u32, u32)>,
    min_delta_time: Cell<Duration>,
    delta_time: Duration,
    fps: u32,
    pressed_keys: Vec<Key>,
    released_keys: Vec<Key>,
    cursor_position: Option<(f32, f32)>,
    mouse: Mouse,
}

impl Control {
    pub fn resized(&self) -> Option<(u32, u32)> {
        self.resized
    }

    fn resize(&mut self, state: &State) {
        self.view.resize(state);
        self.resized = Some(self.view.size());
    }

    pub fn set_min_delta_time(&self, min_delta_time: Duration) {
        self.min_delta_time.set(min_delta_time);
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn pressed_keys(&self) -> &[Key] {
        &self.pressed_keys
    }

    pub fn released_keys(&self) -> &[Key] {
        &self.released_keys
    }

    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.cursor_position
    }

    pub fn cursor_position_normalized(&self) -> Option<(f32, f32)> {
        let (width, height) = self.view.size();
        let norm = |(x, y)| {
            let nx = 1. - x * 2. / width as f32;
            let ny = 1. - y * 2. / height as f32;
            (nx, ny)
        };

        self.cursor_position.map(norm)
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }

    fn clear_state(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.resized = None;
        self.mouse.clear();
    }
}

impl ops::Deref for Control {
    type Target = View;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

/// Keyboard input.
#[derive(Clone)]
pub struct Key {
    pub code: KeyCode,
    pub text: Option<SmolStr>,
}

/// Mouse input.
pub struct Mouse {
    pub wheel_delta: (f32, f32),
    pub pressed_buttons: Buttons,
    pub released_buttons: Buttons,
}

impl Mouse {
    fn clear(&mut self) {
        self.wheel_delta = (0., 0.);
        self.pressed_buttons.0.clear();
        self.released_buttons.0.clear();
    }
}

/// Mouse buttons.
pub struct Buttons(Vec<MouseButton>);

impl Buttons {
    fn push(&mut self, button: MouseButton) {
        self.0.push(button);
    }
}

impl ops::Deref for Buttons {
    type Target = [MouseButton];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The control flow trait for the [`Update`] stage.
pub trait Flow {
    fn flow(self) -> Then;
}

impl Flow for () {
    fn flow(self) -> Then {
        Then::Run
    }
}

/// The control flow type.
pub enum Then {
    /// Keep running the application.
    Run,

    /// Close the application.
    Close,

    /// Exit with an error.
    Fail(Box<dyn error::Error>),
}

impl Flow for Then {
    fn flow(self) -> Self {
        self
    }
}

/// The shortcut for creation [`Then`] value
/// with an error from a [result](Result).
#[macro_export]
macro_rules! then {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(v) => v,
            ::std::result::Result::Err(e) => {
                return $crate::Then::Fail(::std::boxed::Box::from(e));
            }
        }
    };
}
