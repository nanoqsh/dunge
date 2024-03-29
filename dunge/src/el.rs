use {
    crate::{
        context::Context,
        state::State,
        time::{Fps, Time},
        update::Update,
        window::View,
    },
    std::{cell::Cell, error, fmt, ops, time::Duration},
    wgpu::SurfaceError,
    winit::{
        error::EventLoopError,
        event,
        event_loop::{self, EventLoop},
        keyboard,
    },
};

/// Code representing the location of a physical key.
pub type KeyCode = keyboard::KeyCode;

/// String type from `winit` crate.
pub type SmolStr = keyboard::SmolStr;

/// Describes a button of a mouse controller.
pub type MouseButton = event::MouseButton;

pub(crate) struct Loop(EventLoop<()>);

impl Loop {
    pub fn new() -> Result<Self, EventLoopError> {
        use winit::event_loop::EventLoopBuilder;

        let inner = EventLoopBuilder::with_user_event().build()?;
        Ok(Self(inner))
    }

    pub fn inner(&self) -> &EventLoop<()> {
        &self.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<U>(self, cx: Context, view: View, upd: U) -> Result<(), LoopError>
    where
        U: Update,
    {
        let mut fail = Ok(());
        let mut handle = handle(cx, view, upd);
        let wrap = |ev, target: &_| {
            if let Some(err) = handle(ev, target) {
                fail = Err(LoopError::Failed(err));
            }
        };

        let out = self.0.run(wrap).map_err(LoopError::EventLoop);
        fail.or(out)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn<U>(self, cx: Context, view: View, upd: U)
    where
        U: Update + 'static,
    {
        use winit::platform::web::EventLoopExtWebSys;

        let mut handle = handle(cx, view, upd);
        let wrap = move |ev, target: &_| _ = handle(ev, target);
        self.0.spawn(wrap);
    }
}

/// The event loop error.
#[derive(Debug)]
pub enum LoopError {
    EventLoop(EventLoopError),
    Failed(Box<dyn error::Error>),
}

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EventLoop(err) => err.fmt(f),
            Self::Failed(err) => err.fmt(f),
        }
    }
}

impl error::Error for LoopError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::EventLoop(err) => Some(err),
            Self::Failed(err) => Some(err.as_ref()),
        }
    }
}

type Event = event::Event<()>;
type Target = event_loop::EventLoopWindowTarget<()>;
type Failure = Option<Box<dyn error::Error>>;

fn handle<U>(cx: Context, view: View, mut upd: U) -> impl FnMut(Event, &Target) -> Failure
where
    U: Update,
{
    use {
        event::{ElementState, KeyEvent, MouseScrollDelta, StartCause, WindowEvent},
        event_loop::ControlFlow,
        keyboard::PhysicalKey,
        winit::dpi::{PhysicalPosition, PhysicalSize},
    };

    const WAIT_TIME: Duration = Duration::from_millis(100);

    let mut ctrl = Control {
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

    // Initial state
    let mut active = false;
    let mut time = Time::now();
    let mut fps = Fps::default();
    move |ev, target| {
        match ev {
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => {
                    log::debug!("resume time reached");
                    ctrl.view.set_window_size();
                    ctrl.view.request_redraw();
                }
                StartCause::WaitCancelled {
                    requested_resume, ..
                } => {
                    log::debug!("wait cancelled");
                    let flow = match requested_resume {
                        Some(resume) => ControlFlow::WaitUntil(resume),
                        None => ControlFlow::wait_duration(WAIT_TIME),
                    };

                    target.set_control_flow(flow);
                }
                StartCause::Poll => log::debug!("poll"),
                StartCause::Init => log::debug!("init"),
            },
            Event::WindowEvent { event, window_id } if window_id == ctrl.view.id() => match event {
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    log::debug!("resized: {width}, {height}");
                    ctrl.resize(cx.state());
                }
                WindowEvent::CloseRequested => {
                    log::debug!("close requested");
                    target.exit();
                }
                WindowEvent::Focused(true) => {
                    log::debug!("focused");
                    ctrl.view.request_redraw();
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
                            return None;
                        }
                    };

                    // TODO: Support key location
                    _ = location;

                    let key = Key { code, text };
                    match state {
                        ElementState::Pressed => ctrl.pressed_keys.push(key),
                        ElementState::Released => ctrl.released_keys.push(key),
                    }
                }
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => ctrl.cursor_position = Some((x as f32, y as f32)),
                WindowEvent::CursorLeft { .. } => ctrl.cursor_position = None,
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                    ..
                } => {
                    ctrl.mouse.wheel_delta.0 += x;
                    ctrl.mouse.wheel_delta.1 += y;
                }
                WindowEvent::MouseInput { state, button, .. } => match state {
                    ElementState::Pressed => ctrl.mouse.pressed_buttons.push(button),
                    ElementState::Released => ctrl.mouse.released_buttons.push(button),
                },
                WindowEvent::RedrawRequested => {
                    if active {
                        log::debug!("redraw requested");
                    } else {
                        log::debug!("redraw requested (non-active)");

                        // Wait a while to become active
                        target.set_control_flow(ControlFlow::wait_duration(WAIT_TIME));
                        return None;
                    }

                    let delta_time = time.delta();
                    let min_delta_time = ctrl.min_delta_time.get();
                    if delta_time < min_delta_time {
                        let wait = min_delta_time - delta_time;
                        target.set_control_flow(ControlFlow::wait_duration(wait));
                        return None;
                    }

                    time.reset();
                    ctrl.delta_time = delta_time;
                    if let Some(fps) = fps.count(delta_time) {
                        ctrl.fps = fps;
                    }

                    match upd.update(&ctrl).flow() {
                        Then::Run => {}
                        Then::Close => {
                            log::debug!("close");
                            target.exit();
                            return None;
                        }
                        Then::Fail(err) => {
                            log::error!("failed: {err:?}");
                            target.exit();
                            return Some(err);
                        }
                    }

                    ctrl.clear_state();
                    match ctrl.view.output() {
                        Ok(output) => {
                            let target = output.target();
                            cx.state().draw(target, &upd);
                            output.present();
                        }
                        Err(SurfaceError::Timeout) => log::info!("suface error: timeout"),
                        Err(SurfaceError::Outdated) => log::info!("suface error: outdated"),
                        Err(SurfaceError::Lost) => {
                            log::info!("suface error: lost");
                            ctrl.resize(cx.state());
                        }
                        Err(SurfaceError::OutOfMemory) => {
                            log::error!("suface error: out of memory");
                            target.exit();
                        }
                    }
                }
                _ => {}
            },
            Event::Suspended => {
                log::debug!("suspended");
                active = false;
            }
            Event::Resumed => {
                log::debug!("resumed");
                active = true;
                ctrl.view.request_redraw();

                // Reset the timer before start the loop
                time.reset();
            }
            _ => {}
        }

        None
    }
}

/// The main event loop control type.
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

/// The control flow trait for the [`Update`] event.
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
