use crate::r#loop::Mouse;

use {
    crate::{
        context::Context,
        r#loop::{Input, Loop},
        render::{Render, RenderResult},
    },
    std::{num::NonZeroU32, time::Instant},
    winit::{
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    },
};

pub struct Canvas {
    event_loop: EventLoop<()>,
    window: Window,
}

impl Canvas {
    pub fn run<M, L>(self, make_loop: M) -> !
    where
        M: FnOnce(&mut Context) -> L,
        L: Loop + 'static,
    {
        let Self { event_loop, window } = self;

        // Create the render
        let mut render = pollster::block_on(Render::new(&window));

        // Initial resize
        render.resize({
            let (width, height): (u32, u32) = window.inner_size().into();
            (
                NonZeroU32::new(width.max(1)).expect("non zero"),
                NonZeroU32::new(height.max(1)).expect("non zero"),
            )
        });

        // Create the context
        let mut context = Context { window, render };

        // Create the loop object
        let mut lp = make_loop(&mut context);

        // Set an initial state
        let mut time = Time::new();
        let mut cursor_position = None;
        let mut mouse = Mouse::default();

        event_loop.run(move |ev, _, flow| {
            use {
                wgpu::SurfaceError,
                winit::{
                    event::{
                        DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta,
                        StartCause, VirtualKeyCode, WindowEvent,
                    },
                    event_loop::ControlFlow,
                },
            };

            match ev {
                Event::WindowEvent { event, window_id } if window_id == context.window.id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *flow = ControlFlow::Exit,
                        WindowEvent::Resized(size)
                        | WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        } => context.render.resize({
                            let (width, height): (u32, u32) = size.into();
                            (
                                NonZeroU32::new(width.max(1)).expect("non zero"),
                                NonZeroU32::new(height.max(1)).expect("non zero"),
                            )
                        }),
                        WindowEvent::CursorMoved { position, .. } => {
                            cursor_position = Some(position.into());
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
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == context.window.id() => {
                    // Measure a delta time
                    let delta_time = time.delta();

                    // Create an input
                    let input = Input {
                        delta_time,
                        cursor_position,
                        mouse,
                    };

                    // Reset mouse delta
                    mouse = Mouse::default();

                    if let Err(err) = lp.update(&mut context, &input) {
                        lp.error_occurred(err);
                    }

                    match context.render.draw_frame(&lp) {
                        RenderResult::Ok => {}
                        RenderResult::SurfaceError(SurfaceError::Timeout) => {
                            log::error!("suface error: timeout")
                        }
                        RenderResult::SurfaceError(SurfaceError::Outdated) => {
                            log::error!("suface error: outdated")
                        }
                        RenderResult::SurfaceError(SurfaceError::Lost) => {
                            context.render.resize(context.size())
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

pub fn make_window(state: InitialState) -> Canvas {
    use winit::{dpi::PhysicalSize, window::Fullscreen};

    let builder = WindowBuilder::new().with_title(state.title);
    let builder = match state.mode {
        WindowMode::Fullscreen => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
        WindowMode::Windowed { width, height } => {
            builder.with_inner_size(PhysicalSize::new(width.max(1), height.max(1)))
        }
    };

    let event_loop = EventLoop::new();
    let window = builder.build(&event_loop).expect("build window");
    window.set_cursor_visible(state.show_cursor);

    Canvas { event_loop, window }
}

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

pub enum WindowMode {
    Fullscreen,
    Windowed { width: u32, height: u32 },
}

pub fn from_canvas(_tag: &str) -> Canvas {
    todo!()
}

struct Time {
    last: Instant,
}

impl Time {
    fn new() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        delta.as_secs_f32()
    }
}
