use {
    crate::{
        context::Context,
        element::Element,
        r#loop::{Input, Keys, Loop, Mouse},
        render::State,
        screen::Screen,
        time::{Fps, Time},
    },
    wgpu::AdapterInfo,
    winit::{
        error::EventLoopError,
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
        window::{Window, WindowBuilder},
    },
};

/// The render canvas.
pub struct Canvas {
    event_loop: EventLoop<CanvasEvent>,
    window: Window,
    el: Element,
}

impl Canvas {
    /// Calls [`run`](crate::Canvas::run) but blocking instead of async.
    ///
    /// # Errors
    /// Returns [`CanvasError`](crate::error::CanvasError) if backend selection or request device failed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_blocking<M, L>(self, config: CanvasConfig, make: M) -> Result<(), Error>
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        pollster::block_on(self.run(config, make))
    }

    /// Runs the main loop.
    ///
    /// This function construct a [`Context`] instance and
    /// calls a `make` by passing the context in it.
    /// The `make` needs to return an object which
    /// implements the [`Loop`] trait.
    ///
    /// # Errors
    /// Returns [`CanvasError`](crate::error::CanvasError) if backend selection or request device failed.
    pub async fn run<M, L>(self, config: CanvasConfig, make: M) -> Result<(), Error>
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        let Self {
            event_loop,
            window,
            el,
        } = self;

        // Create the context
        let mut context = {
            // Create the render state
            let state = State::new(config, &window).await?;
            Context::new(window, event_loop.create_proxy(), state)
        };

        // Create the loop object
        let mut lp = make(&mut context);

        // Initial state
        let mut active = false;
        let mut time = Time::now();
        let mut fps = Fps::default();
        let mut deferred_screen = None;
        let mut cursor_position = None;
        let mut last_touch = None;
        let mut mouse = Mouse::default();
        let mut pressed_keys = vec![];
        let mut released_keys = vec![];

        let handler = move |ev, target: &EventLoopWindowTarget<_>| {
            use {
                std::{num::NonZeroU32, time::Duration},
                wgpu::SurfaceError,
                winit::{
                    dpi::{PhysicalPosition, PhysicalSize},
                    event::{
                        DeviceEvent, ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta,
                        StartCause, Touch, TouchPhase, WindowEvent,
                    },
                    keyboard::PhysicalKey,
                },
            };

            const WAIT_TIME: Duration = Duration::from_millis(100);

            match ev {
                Event::NewEvents(cause) => match cause {
                    StartCause::ResumeTimeReached { .. } => {
                        log::info!("resume time reached");

                        el.set_window_size(&context.window);
                        context.window.request_redraw();
                    }
                    StartCause::WaitCancelled {
                        requested_resume, ..
                    } => {
                        log::info!("wait cancelled");

                        target.set_control_flow(match requested_resume {
                            Some(resume) => ControlFlow::WaitUntil(resume),
                            None => ControlFlow::wait_duration(WAIT_TIME),
                        });
                    }
                    StartCause::Poll => log::info!("poll"),
                    StartCause::Init => log::info!("init"),
                },
                Event::WindowEvent { event, window_id } if window_id == context.window.id() => {
                    log::info!("window event: {event:?}");

                    match event {
                        WindowEvent::Resized(size) => context.render.resize(size.into()),
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            let PhysicalSize { width, height } = context.window.inner_size();
                            let size = (
                                (width as f64 * scale_factor) as u32,
                                (height as f64 * scale_factor) as u32,
                            );

                            context.render.resize(size);
                        }
                        WindowEvent::CloseRequested if lp.close_requested() => target.exit(),
                        WindowEvent::Focused(true) => context.window.request_redraw(),
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(key),
                                    state,
                                    ..
                                },
                            ..
                        } => match state {
                            ElementState::Pressed => pressed_keys.push(key),
                            ElementState::Released => released_keys.push(key),
                        },
                        WindowEvent::CursorMoved {
                            position: PhysicalPosition { x, y },
                            ..
                        } => {
                            let PhysicalSize { width, height } = context.window.inner_size();
                            let nx = 1. - x as f32 * 2. / width as f32;
                            let ny = 1. - y as f32 * 2. / height as f32;
                            cursor_position = Some((nx, ny));
                        }
                        WindowEvent::CursorLeft { .. } => {
                            cursor_position = None;
                        }
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(x, y),
                            ..
                        } => {
                            mouse.wheel_delta.0 += x;
                            mouse.wheel_delta.1 += y;
                        }
                        WindowEvent::MouseInput { state, button, .. } => match button {
                            MouseButton::Left => {
                                mouse.pressed_left = state == ElementState::Pressed;
                            }
                            MouseButton::Right => {
                                mouse.pressed_right = state == ElementState::Pressed;
                            }
                            MouseButton::Middle => {
                                mouse.pressed_middle = state == ElementState::Pressed;
                            }
                            _ => {}
                        },
                        WindowEvent::Touch(Touch {
                            phase,
                            location: PhysicalPosition { x, y },
                            ..
                        }) => match phase {
                            TouchPhase::Started => {}
                            TouchPhase::Moved => {
                                let (nx, ny) = (x as f32, y as f32);
                                if let Some((lx, ly)) = last_touch {
                                    mouse.motion_delta.0 = lx - nx;
                                    mouse.motion_delta.1 = ly - ny;
                                }

                                last_touch = Some((nx, ny));
                            }
                            TouchPhase::Ended | TouchPhase::Cancelled => last_touch = None,
                        },
                        WindowEvent::RedrawRequested => {
                            if active {
                                log::info!("redraw requested (active)");
                            } else {
                                log::info!("redraw requested");

                                // Wait a while to become active
                                target.set_control_flow(ControlFlow::wait_duration(WAIT_TIME));
                                return;
                            }

                            // Measure the delta time
                            let delta_time = time.delta();

                            // If frame rate is limited, skip drawing until it's time
                            let min_delta_time = context.limits.min_delta_time;
                            if delta_time < min_delta_time {
                                let wait = Duration::from_secs_f32(min_delta_time - delta_time);
                                target.set_control_flow(ControlFlow::wait_duration(wait));
                                return;
                            }

                            // Count number of frames
                            if let Some(fps) = fps.count(delta_time) {
                                context.fps = fps;
                            }

                            // Create an user's input data
                            let input = Input {
                                delta_time,
                                cursor_position,
                                mouse,
                                pressed_keys: Keys {
                                    keys: &pressed_keys[..],
                                },
                                released_keys: Keys {
                                    keys: &released_keys[..],
                                },
                            };

                            // Reset delta time
                            time.reset();

                            // Update the loop
                            lp.update(&mut context, &input);

                            // Reset mouse delta
                            mouse = Mouse::default();

                            // Reset keys
                            pressed_keys.clear();
                            released_keys.clear();

                            match context.render.draw_frame(&lp) {
                                Ok(()) => {}
                                Err(SurfaceError::Timeout) => {
                                    log::info!("suface error: timeout");
                                }
                                Err(SurfaceError::Outdated | SurfaceError::Lost) => {
                                    context.render.resize(context.window.inner_size().into());
                                }
                                Err(SurfaceError::OutOfMemory) => {
                                    log::error!("suface error: out of memory");
                                    target.exit();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (x, y) },
                    ..
                } => {
                    log::info!("device event: mouse motion");

                    mouse.motion_delta.0 += x as f32;
                    mouse.motion_delta.1 += y as f32;
                }
                Event::UserEvent(CanvasEvent::SetScreen(screen)) => {
                    log::info!("user event: set screen");
                    if active {
                        context.render.set_screen(screen);
                    } else {
                        deferred_screen = Some(screen);
                    }
                }
                Event::UserEvent(CanvasEvent::Close) if lp.close_requested() => {
                    log::info!("user event: close");
                    target.exit();
                }
                Event::Suspended => {
                    log::info!("suspended");
                    context.render.drop_surface();
                    active = false;
                }
                Event::Resumed => {
                    log::info!("resumed");
                    context.render.recreate_surface(&context.window);

                    // Set render screen on application start and resume
                    let (width, height) = context.window.inner_size().into();
                    match deferred_screen.take() {
                        Some(screen) => context.render.set_screen(Screen {
                            width: NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
                            height: NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN),
                            ..screen
                        }),
                        None => context.render.resize((width, height)),
                    }

                    active = true;
                    context.window.request_redraw();

                    if let Some(screen) = deferred_screen.take() {
                        context.render.set_screen(screen);
                    }

                    // Reset the timer before start the loop
                    time.reset();
                }
                _ => {}
            }
        };

        event_loop.run(handler).map_err(Error::EventLoop)
    }
}

/// An error returned from the [`Context`] constructors.
#[derive(Debug)]
pub enum Error {
    BackendSelection(Backend),
    RequestDevice,
    EventLoop(EventLoopError),
}

pub(crate) enum CanvasEvent {
    SetScreen(Screen),
    Close,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod window {
    use super::*;

    /// Creates a canvas in the window with given initial state.
    ///
    /// # Panics
    /// Panics if window creation fails.
    pub fn make_window(state: InitialState) -> Canvas {
        use winit::{dpi::PhysicalSize, window::Fullscreen};

        let builder = WindowBuilder::new().with_title(state.title);
        let builder = match state.mode {
            WindowMode::Fullscreen => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
            WindowMode::Windowed { width, height } => {
                builder.with_inner_size(PhysicalSize::new(width.max(1), height.max(1)))
            }
        };

        let event_loop = EventLoopBuilder::with_user_event()
            .build()
            .expect("build event loop");

        let window = builder.build(&event_loop).expect("build window");
        window.set_cursor_visible(state.show_cursor);

        Canvas {
            event_loop,
            window,
            el: Element::default(),
        }
    }

    /// The initial window state.
    #[derive(Clone, Copy)]
    pub struct InitialState<'a> {
        pub title: &'a str,
        pub mode: WindowMode,
        pub show_cursor: bool,
    }

    impl Default for InitialState<'_> {
        fn default() -> Self {
            Self {
                title: "Dunge",
                mode: WindowMode::default(),
                show_cursor: true,
            }
        }
    }

    /// The window mode.
    #[derive(Clone, Copy, Default)]
    pub enum WindowMode {
        #[default]
        Fullscreen,
        Windowed {
            width: u32,
            height: u32,
        },
    }
}

/// Creates a canvas in the HTML element by its id.
#[cfg(target_arch = "wasm32")]
pub fn from_element(id: &str) -> Canvas {
    use {web_sys::Window, winit::platform::web::WindowExtWebSys};

    let event_loop = EventLoopBuilder::with_user_event()
        .build()
        .expect("build event loop");

    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("build window");

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let Some(el) = document.get_element_by_id(id) else {
        panic!("an element with id {id:?} not found");
    };

    let el = Element::new(el);
    el.set_window_size(&window);

    let canvas = window.canvas();
    canvas.remove_attribute("style").expect("remove attribute");
    el.append_child(&canvas).expect("append child");

    Canvas {
        event_loop,
        window,
        el,
    }
}

#[cfg(target_os = "android")]
pub(crate) mod android {
    use super::*;
    use winit::platform::android::activity::AndroidApp;

    /// Creates a canvas from the `AndroidApp` class.
    pub fn from_app(app: AndroidApp) -> Canvas {
        use winit::platform::android::EventLoopBuilderExtAndroid;

        let event_loop = EventLoopBuilder::with_user_event()
            .with_android_app(app)
            .build()
            .expect("build event loop");

        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("build window");

        Canvas {
            event_loop,
            window,
            el: Element::default(),
        }
    }
}

/// The [`Canvas`] config.
#[derive(Default)]
pub struct CanvasConfig {
    pub backend: Backend,
    pub selector: Selector,
}

/// Description of [backend](Backend) selection behavior.
#[derive(Default)]
pub enum Selector {
    #[default]
    Auto,
    #[cfg(not(target_arch = "wasm32"))]
    Callback(Box<dyn FnMut(Vec<Info>) -> Option<usize>>),
}

/// The render backend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Backend {
    #[cfg_attr(not(target_arch = "wasm32"), default)]
    Vulkan,
    #[cfg_attr(target_arch = "wasm32", default)]
    Gl,
    Dx12,
    Dx11,
    Metal,
    WebGpu,
}

/// The render context info.
#[derive(Clone, Debug)]
pub struct Info {
    pub backend: Backend,
    pub name: String,
    pub device: Device,
}

impl Info {
    pub(crate) fn from_adapter_info(info: AdapterInfo) -> Self {
        use wgpu::{Backend as Bk, DeviceType};

        Self {
            backend: match info.backend {
                Bk::Empty => unreachable!(),
                Bk::Vulkan => Backend::Vulkan,
                Bk::Metal => Backend::Metal,
                Bk::Dx12 => Backend::Dx12,
                Bk::Dx11 => Backend::Dx11,
                Bk::Gl => Backend::Gl,
                Bk::BrowserWebGpu => Backend::WebGpu,
            },
            name: info.name,
            device: match info.device_type {
                DeviceType::IntegratedGpu => Device::IntegratedGpu,
                DeviceType::DiscreteGpu => Device::DiscreteGpu,
                DeviceType::VirtualGpu => Device::VirtualGpu,
                DeviceType::Cpu => Device::Cpu,
                DeviceType::Other => Device::Other,
            },
        }
    }
}

/// The device type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
    Other,
}
