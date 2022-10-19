use {
    crate::{
        context::{Context, Limits},
        r#loop::{Input, Loop, Mouse, PressedKeys},
        render::{Render, RenderResult},
        size::Size,
        time::Time,
    },
    std::num::NonZeroU32,
    winit::{
        event_loop::EventLoop,
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_blocking<M, L>(self, make_loop: M) -> !
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        pollster::block_on(self.run(make_loop))
    }

    /// Runs the main loop.
    ///
    /// This function construct a [`Context`] instance and
    /// calls a `make_loop` by passing the context in it.
    /// The `make_loop` needs to return an object which
    /// implements the [`Loop`] trait.
    pub async fn run<M, L>(self, make_loop: M) -> !
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        let Self { event_loop, window } = self;

        // Create the render
        let mut render = Render::new(&window).await;

        // Initial resize
        render.resize({
            let (width, height): (u32, u32) = window.inner_size().into();
            Some(Size {
                width: width.max(1).try_into().expect("non zero"),
                height: height.max(1).try_into().expect("non zero"),
                ..Default::default()
            })
        });

        // Create the context
        let mut context = Context {
            window,
            proxy: event_loop.create_proxy(),
            render,
            limits: Limits::default(),
        };

        // Create the loop object
        let mut lp = make_loop(&mut context);

        // Set an initial state
        let mut time = Time::new();
        let mut cursor_position = None;
        let mut mouse = Mouse::default();
        let mut pressed_keys = vec![];

        event_loop.run(move |ev, _, flow| {
            use {
                wgpu::SurfaceError,
                winit::{
                    dpi::PhysicalPosition,
                    event::{
                        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton,
                        MouseScrollDelta, StartCause, WindowEvent,
                    },
                    event_loop::ControlFlow,
                },
            };

            match ev {
                Event::WindowEvent { event, window_id } if window_id == context.window.id() => {
                    match event {
                        WindowEvent::Resized(size)
                        | WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        } => context.render.resize({
                            let (width, height): (u32, u32) = size.into();
                            let size = context.render.size();
                            Some(Size {
                                width: NonZeroU32::new(width.max(1)).expect("non zero"),
                                height: NonZeroU32::new(height.max(1)).expect("non zero"),
                                ..size
                            })
                        }),
                        WindowEvent::CloseRequested if lp.close_requested() => {
                            *flow = ControlFlow::Exit;
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        } => match state {
                            ElementState::Pressed if !pressed_keys.contains(&key) => {
                                pressed_keys.push(key)
                            }
                            ElementState::Released => {
                                if let Some(i) = pressed_keys.iter().position(|&k| k == key) {
                                    pressed_keys.remove(i);
                                }
                            }
                            _ => {}
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
                            MouseScrollDelta::PixelDelta(PhysicalPosition { .. }) => {
                                // TODO
                            }
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
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == context.window.id() => {
                    // Measure a delta time
                    let delta_time = time.delta();
                    if let Some(min_delta_time) = context.limits.min_frame_delta_time {
                        if delta_time < min_delta_time {
                            return;
                        }
                    }

                    // Create an input
                    let input = Input {
                        delta_time,
                        cursor_position,
                        mouse,
                        pressed_keys: PressedKeys {
                            keys: &pressed_keys[..],
                        },
                    };

                    // Reset delta time
                    time.reset();

                    // Reset mouse delta
                    mouse = Mouse::default();

                    if let Err(err) = lp.update(&mut context, &input) {
                        lp.error_occurred(err);
                    }

                    match context.render.start_frame(&lp) {
                        RenderResult::Ok => {}
                        RenderResult::SurfaceError(SurfaceError::Timeout) => {
                            log::error!("suface error: timeout");
                        }
                        RenderResult::SurfaceError(SurfaceError::Outdated) => {
                            log::error!("suface error: outdated");
                        }
                        RenderResult::SurfaceError(SurfaceError::Lost) => {
                            context.render.resize(None);
                        }
                        RenderResult::SurfaceError(SurfaceError::OutOfMemory) => {
                            log::error!("suface error: out of memory");
                            *flow = ControlFlow::Exit;
                        }
                        RenderResult::Error(err) => lp.error_occurred(err),
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (x, y) },
                    ..
                } => {
                    mouse.motion_delta.0 += x as f32;
                    mouse.motion_delta.1 += y as f32;
                }
                Event::UserEvent(CanvasEvent::Close) => {
                    if lp.close_requested() {
                        *flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => context.window.request_redraw(),
                Event::NewEvents(StartCause::Init) => {
                    // Reset the timer before start the loop
                    _ = time.delta();
                }
                _ => {}
            }
        })
    }
}

pub(crate) enum CanvasEvent {
    Close,
}

/// Creates a canvas in a window with given initial state.
#[cfg(not(target_arch = "wasm32"))]
pub fn make_window(state: InitialState) -> Canvas {
    use winit::{dpi::PhysicalSize, event_loop::EventLoopBuilder, window::Fullscreen};

    let builder = WindowBuilder::new().with_title(state.title);
    let builder = match state.mode {
        WindowMode::Fullscreen => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
        WindowMode::Windowed { width, height } => {
            builder.with_inner_size(PhysicalSize::new(width.max(1), height.max(1)))
        }
    };

    let event_loop = EventLoopBuilder::with_user_event().build();
    let window = builder.build(&event_loop).expect("build window");
    window.set_cursor_visible(state.show_cursor);

    Canvas { event_loop, window }
}

/// The initial window state.
pub struct InitialState<'a> {
    pub title: &'a str,
    pub mode: WindowMode,
    pub show_cursor: bool,
}

impl Default for InitialState<'static> {
    fn default() -> Self {
        Self {
            title: "Dunge",
            mode: WindowMode::Fullscreen,
            show_cursor: true,
        }
    }
}

/// The window mode.
pub enum WindowMode {
    Fullscreen,
    Windowed { width: u32, height: u32 },
}

/// Creates a canvas in the HTML element by its id.
#[cfg(target_arch = "wasm32")]
pub fn from_element(id: &str) -> Canvas {
    use {
        web_sys::Window,
        winit::{dpi::PhysicalSize, event_loop::EventLoopBuilder, platform::web::WindowExtWebSys},
    };

    let event_loop = EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("build window");

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let el = match document.get_element_by_id(id) {
        Some(el) => el,
        None => panic!("an element with id {id:?} not found"),
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
