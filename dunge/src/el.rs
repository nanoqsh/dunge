use {
    crate::{
        context::Context,
        state::{Render, RenderView},
        time::{Fps, Time},
        update::Update,
        window::View,
    },
    std::{error, fmt, time::Duration},
    wgpu::SurfaceError,
    winit::{
        error::EventLoopError,
        event,
        event_loop::{self, EventLoop},
        keyboard::{KeyCode, PhysicalKey, SmolStr},
    },
};

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
    pub fn run<U>(self, cx: Context, view: View, update: U) -> Result<(), LoopError>
    where
        U: Update,
    {
        let handler = handler(cx, view, update);
        self.0.run(handler).map_err(LoopError)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn<U>(self, cx: Context, view: View, update: U)
    where
        U: Update + 'static,
    {
        use winit::platform::web::EventLoopExtWebSys;

        let handler = handler(cx, view, update);
        self.0.spawn(handler)
    }
}

#[derive(Debug)]
pub struct LoopError(EventLoopError);

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for LoopError {}

type Event = event::Event<()>;
type Target = event_loop::EventLoopWindowTarget<()>;

fn handler<U>(cx: Context, mut view: View, mut update: U) -> impl FnMut(Event, &Target)
where
    U: Update,
{
    use {
        event::{ElementState, KeyEvent, StartCause, WindowEvent},
        event_loop::ControlFlow,
        winit::dpi::PhysicalSize,
    };

    const WAIT_TIME: Duration = Duration::from_millis(100);

    let mut ctrl = Control {
        close: false,
        min_delta_time: Duration::from_secs_f32(1. / 60.),
        fps: 0,
        pressed_keys: vec![],
        released_keys: vec![],
    };

    // Initial state
    let mut active = false;
    let mut render = Render::default();
    let mut time = Time::now();
    let mut fps = Fps::default();
    move |ev, target| match ev {
        Event::NewEvents(cause) => match cause {
            StartCause::ResumeTimeReached { .. } => {
                log::debug!("resume time reached");
                if ctrl.close {
                    log::debug!("close");
                    target.exit();
                    return;
                }

                view.request_redraw()
            }
            StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                log::debug!("wait cancelled");
                let flow = match requested_resume {
                    Some(resume) => ControlFlow::WaitUntil(resume),
                    None => ControlFlow::wait_duration(WAIT_TIME),
                };

                target.set_control_flow(flow)
            }
            StartCause::Poll => log::debug!("poll"),
            StartCause::Init => log::debug!("init"),
        },
        Event::WindowEvent { event, window_id } if window_id == view.id() => match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                log::debug!("resized: {width}, {height}");
                view.resize(cx.state());
            }
            WindowEvent::CloseRequested => {
                log::debug!("close requested");
                target.exit();
            }
            WindowEvent::Focused(true) => {
                log::debug!("focused");
                view.request_redraw();
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
                    ElementState::Pressed => ctrl.pressed_keys.push(key),
                    ElementState::Released => ctrl.released_keys.push(key),
                }
            }
            WindowEvent::RedrawRequested => {
                if active {
                    log::debug!("redraw requested");
                } else {
                    log::debug!("redraw requested (non-active)");

                    // Wait a while to become active
                    target.set_control_flow(ControlFlow::wait_duration(WAIT_TIME));
                    return;
                }

                let delta_time = time.delta();
                if delta_time < ctrl.min_delta_time {
                    let wait = ctrl.min_delta_time - delta_time;
                    target.set_control_flow(ControlFlow::wait_duration(wait));
                    return;
                }

                time.reset();
                if let Some(fps) = fps.count(delta_time) {
                    ctrl.fps = fps;
                }

                update.update(&mut ctrl);
                ctrl.clear_keys();
                match view.output() {
                    Ok(output) => {
                        let view = RenderView::from_output(&output);
                        cx.state().draw(&mut render, view, &update);
                        output.present();
                    }
                    Err(SurfaceError::Timeout) => log::info!("suface error: timeout"),
                    Err(SurfaceError::Outdated) => log::info!("suface error: outdated"),
                    Err(SurfaceError::Lost) => {
                        log::info!("suface error: lost");
                        view.resize(cx.state());
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
            // TODO: Drop the surface
            active = false;
        }
        Event::Resumed => {
            log::debug!("resumed");
            active = true;
            view.request_redraw();

            // Reset the timer before start the loop
            time.reset();
        }
        _ => {}
    }
}

pub struct Control {
    close: bool,
    min_delta_time: Duration,
    fps: u32,
    pressed_keys: Vec<Key>,
    released_keys: Vec<Key>,
}

impl Control {
    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn set_min_delta_time(&mut self, min_delta_time: Duration) {
        self.min_delta_time = min_delta_time;
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

    fn clear_keys(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }
}

#[derive(Clone)]
pub struct Key {
    pub code: KeyCode,
    pub text: Option<SmolStr>,
}
