use {
    crate::{
        context::Context,
        r#loop::{Input, Keys, Loop, Mouse},
        render::State,
        screen::Screen,
        time::Time,
    },
    wgpu::AdapterInfo,
    winit::{
        event_loop::{EventLoop, EventLoopBuilder},
        window::{Window, WindowBuilder},
    },
};

/// The render canvas.
pub struct Canvas {
    event_loop: EventLoop<CanvasEvent>,
    window: Window,
}

impl Canvas {
    /// Calls [`run`](crate::Canvas::run) but blocking instead of async.
    ///
    /// # Errors
    /// Returns [`CanvasError`](crate::error::CanvasError) if backend selection or request device failed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_blocking<M, L>(self, config: CanvasConfig, make_loop: M) -> Error
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        pollster::block_on(self.run(config, make_loop))
    }

    /// Runs the main loop.
    ///
    /// This function construct a [`Context`] instance and
    /// calls a `make_loop` by passing the context in it.
    /// The `make_loop` needs to return an object which
    /// implements the [`Loop`] trait.
    ///
    /// # Errors
    /// Returns [`CanvasError`](crate::error::CanvasError) if backend selection or request device failed.
    pub async fn run<M, L>(self, config: CanvasConfig, make_loop: M) -> Error
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        let Self { event_loop, window } = self;

        // Create the context
        let mut context = {
            // Create the render state
            let state = match State::new(config, &window).await {
                Ok(render) => render,
                Err(err) => return err,
            };

            Box::new(Context::new(window, event_loop.create_proxy(), state))
        };

        // Create the loop object
        let mut lp = make_loop(&mut context);

        // Set an initial state
        let mut active = false;
        let mut time = Time::new();
        let mut deferred_screen = None;
        let mut cursor_position = None;
        let mut last_touch = None;
        let mut mouse = Mouse::default();
        let mut pressed_keys = vec![];
        let mut released_keys = vec![];

        event_loop.run(move |ev, _, flow| {
            use {
                std::{num::NonZeroU32, time::Duration},
                wgpu::SurfaceError,
                winit::{
                    dpi::PhysicalPosition,
                    event::{
                        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton,
                        MouseScrollDelta, StartCause, Touch, TouchPhase, WindowEvent,
                    },
                },
            };

            const WAIT_TIME: f32 = 0.1;

            match ev {
                Event::NewEvents(cause) => match cause {
                    StartCause::ResumeTimeReached { .. } => {
                        log::info!("resume time reached");
                        context.window.request_redraw();
                    }
                    StartCause::WaitCancelled {
                        requested_resume, ..
                    } => {
                        log::info!("wait cancelled");
                        if let Some(resume) = requested_resume {
                            flow.set_wait_until(resume);
                        }
                    }
                    StartCause::Poll => {
                        log::info!("poll");
                        flow.set_wait_timeout(Duration::from_secs_f32(WAIT_TIME));
                    }
                    StartCause::Init => log::info!("init"),
                },
                Event::WindowEvent { event, window_id } if window_id == context.window.id() => {
                    log::info!("window event: {event:?}");

                    match event {
                        WindowEvent::Resized(size)
                        | WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        } => context.render.resize(size.into()),
                        WindowEvent::CloseRequested if lp.close_requested() => flow.set_exit(),
                        WindowEvent::Focused(true) => context.window.request_redraw(),
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        } => match state {
                            ElementState::Pressed => pressed_keys.push(key),
                            ElementState::Released => released_keys.push(key),
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            cursor_position = Some(position.into());
                        }
                        WindowEvent::CursorLeft { .. } => {
                            cursor_position = None;
                        }
                        WindowEvent::MouseWheel { delta, .. } => match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                mouse.wheel_delta.0 += x;
                                mouse.wheel_delta.1 += y;
                            }
                            MouseScrollDelta::PixelDelta(PhysicalPosition { .. }) => {}
                        },
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
                            MouseButton::Other(_) => {}
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
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == context.window.id() => {
                    log::info!(
                        "redraw requested {active}",
                        active = if active { "(active)" } else { "" },
                    );

                    if !active {
                        // Wait a while to become active
                        flow.set_wait_timeout(Duration::from_secs_f32(WAIT_TIME));
                        return;
                    }

                    // Measure the delta time
                    let delta_time = time.delta();

                    // If frame rate is limited, skip drawing until it's time
                    if let Some(min_delta_time) = context.limits.min_frame_delta_time {
                        if delta_time < min_delta_time {
                            let wait = min_delta_time - delta_time;
                            flow.set_wait_timeout(Duration::from_secs_f32(wait));
                            return;
                        }
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
                            flow.set_exit();
                        }
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
                    flow.set_exit();
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
        })
    }
}

/// An error returned from the [`Context`] constructors.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub enum Error {
    BackendSelection(Backend),
    RequestDevice,
}

impl Error {
    /// Turns the error into panic.
    ///
    /// # Panics
    /// Yes.
    pub fn into_panic(self) {
        panic!("{self:?}");
    }
}

pub(crate) enum CanvasEvent {
    SetScreen(Screen),
    Close,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod window {
    use super::*;

    /// Creates a canvas in the window with given initial state.
    #[must_use]
    pub fn make_window(state: InitialState) -> Canvas {
        use winit::{dpi::PhysicalSize, window::Fullscreen};

        let mut builder = EventLoopBuilder::with_user_event();

        #[cfg(target_os = "linux")]
        {
            use {std::env, winit::platform::x11::EventLoopBuilderExtX11};

            builder.with_x11();
            env::remove_var("WAYLAND_DISPLAY"); // Temporary force x11
        }

        let event_loop = builder.build();

        let builder = WindowBuilder::new().with_title(state.title);
        let builder = match state.mode {
            WindowMode::Fullscreen => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
            WindowMode::Windowed { width, height } => {
                builder.with_inner_size(PhysicalSize::new(width.max(1), height.max(1)))
            }
        };

        let window = builder.build(&event_loop).expect("build window");
        window.set_cursor_visible(state.show_cursor);

        Canvas { event_loop, window }
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
#[must_use]
pub fn from_element(id: &str) -> Canvas {
    use {
        web_sys::Window,
        winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys},
    };

    let event_loop = EventLoopBuilder::with_user_event().build();
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

    window.set_inner_size({
        let width = el.client_width().max(1) as u32;
        let height = el.client_height().max(1) as u32;
        PhysicalSize { width, height }
    });

    let canvas = window.canvas();
    canvas.remove_attribute("style").expect("remove attribute");
    el.append_child(&canvas).expect("append child");

    Canvas { event_loop, window }
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
            .build();

        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("build window");

        Canvas { event_loop, window }
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
                DeviceType::Other => panic!("undefined device type"),
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
}
