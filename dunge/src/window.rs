use {
    crate::{
        context::{self, Context},
        state::{Render, RenderView, State},
        texture::Format,
        time::{Fps, Time},
        update::Update,
    },
    std::{error, fmt},
    wgpu::{
        CreateSurfaceError, Instance, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture,
        TextureView,
    },
    winit::{
        error::{EventLoopError, OsError},
        event,
        event_loop::{self, EventLoop},
        keyboard, window,
    },
};

pub struct WindowBuilder {
    title: String,
    size: Option<(u32, u32)>,
    show_cursor: bool,
}

impl WindowBuilder {
    pub(crate) fn new() -> Self {
        Self {
            title: String::default(),
            size: Some((600, 600)),
            show_cursor: true,
        }
    }

    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_fullscreen(mut self) -> Self {
        self.size = None;
        self
    }

    pub fn with_show_cursor(mut self, show_cursor: bool) -> Self {
        self.show_cursor = show_cursor;
        self
    }

    pub(crate) fn build(self, cx: Context, instance: &Instance) -> Result<Window, Error> {
        use winit::{dpi::PhysicalSize, window::Fullscreen};

        let el = EventLoop::new()?;
        let inner = {
            let builder = window::WindowBuilder::new().with_title(self.title);
            let builder = match self.size {
                Some((width, height)) => builder.with_inner_size(PhysicalSize::new(width, height)),
                None => builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
            };

            builder.build(&el)?
        };

        let view = View::new(cx.state(), instance, inner)?;
        Ok(Window { cx, el, view })
    }
}

type Event = event::Event<()>;
type Target = event_loop::EventLoopWindowTarget<()>;

pub struct Window {
    cx: Context,
    el: EventLoop<()>,
    view: View,
}

impl Window {
    pub fn context(&self) -> Context {
        self.cx.clone()
    }

    pub fn run<U>(self, mut update: U) -> Result<(), LoopError>
    where
        U: Update,
    {
        use {
            event::{KeyEvent, StartCause, WindowEvent},
            event_loop::ControlFlow,
            keyboard::{KeyCode, PhysicalKey},
            std::time::Duration,
        };

        let Self { cx, el, mut view } = self;
        let mut render = Render::default();
        let mut time = Time::now();
        let mut fps = Fps::default();
        let handler = |ev: Event, target: &Target| match ev {
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => view.inner.request_redraw(),
                StartCause::WaitCancelled {
                    requested_resume, ..
                } => target.set_control_flow(match requested_resume {
                    Some(resume) => ControlFlow::WaitUntil(resume),
                    None => {
                        const WAIT_TIME: Duration = Duration::from_millis(100);

                        ControlFlow::wait_duration(WAIT_TIME)
                    }
                }),
                StartCause::Poll => {}
                StartCause::Init => {}
            },
            Event::WindowEvent { event, window_id } if window_id == view.inner.id() => {
                match event {
                    WindowEvent::Resized(_) => view.resize(cx.state()),
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => target.exit(),
                    WindowEvent::RedrawRequested => {
                        let delta_time = time.delta();
                        let min_delta_time = 1. / 60.;
                        if delta_time < min_delta_time {
                            let wait = Duration::from_secs_f32(min_delta_time - delta_time);
                            target.set_control_flow(ControlFlow::wait_duration(wait));
                            return;
                        }

                        time.reset();
                        if let Some(fps) = fps.count(delta_time) {
                            println!("fps: {fps}");
                        }

                        update.update();
                        match view.output() {
                            Ok(output) => {
                                let view = RenderView::from_output(&output);
                                cx.state().draw(&mut render, view, &update);
                                output.surface.present();
                            }
                            Err(SurfaceError::Lost) => view.resize(cx.state()),
                            Err(SurfaceError::OutOfMemory) => target.exit(),
                            Err(err) => eprintln!("{err:?}"),
                        }
                    }
                    _ => {}
                }
            }
            Event::Suspended => {}
            Event::Resumed => view.inner.request_redraw(),
            _ => {}
        };

        el.run(handler).map_err(LoopError)
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

struct View {
    conf: SurfaceConfiguration,
    surface: Surface,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    inner: window::Window,
}

impl View {
    const FORMAT: Format = Format::RgbAlpha;

    fn new(state: &State, instance: &Instance, inner: window::Window) -> Result<Self, Error> {
        use wgpu::*;

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&inner)? };
        let conf = {
            let caps = surface.get_capabilities(state.adapter());
            let format = Self::FORMAT.wgpu();
            if !caps.formats.contains(&format) {
                return Err(ErrorKind::UnsupportedSurface.into());
            }

            let size = inner.inner_size();
            SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: PresentMode::default(),
                alpha_mode: CompositeAlphaMode::default(),
                view_formats: vec![],
            }
        };

        surface.configure(state.device(), &conf);
        Ok(Self {
            conf,
            surface,
            inner,
        })
    }

    fn output(&self) -> Result<Output, SurfaceError> {
        use wgpu::TextureViewDescriptor;

        let output = self.surface.get_current_texture()?;
        let view = {
            let desc = TextureViewDescriptor::default();
            output.texture.create_view(&desc)
        };

        Ok(Output {
            view,
            format: Self::FORMAT,
            surface: output,
        })
    }

    fn resize(&mut self, state: &State) {
        let size = self.inner.inner_size();
        if size.width > 0 && size.height > 0 {
            self.conf.width = size.width;
            self.conf.height = size.height;
            self.surface.configure(state.device(), &self.conf);
        }
    }
}

pub(crate) struct Output {
    view: TextureView,
    format: Format,
    surface: SurfaceTexture,
}

impl Output {
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn format(&self) -> Format {
        self.format
    }
}

#[derive(Debug)]
pub struct Error(ErrorKind);

impl From<ErrorKind> for Error {
    fn from(v: ErrorKind) -> Self {
        Self(v)
    }
}

impl From<EventLoopError> for Error {
    fn from(v: EventLoopError) -> Self {
        Self(ErrorKind::EventLoop(v))
    }
}

impl From<OsError> for Error {
    fn from(v: OsError) -> Self {
        Self(ErrorKind::Os(v))
    }
}

impl From<CreateSurfaceError> for Error {
    fn from(v: CreateSurfaceError) -> Self {
        Self(ErrorKind::Surface(v))
    }
}

impl From<context::Error> for Error {
    fn from(v: context::Error) -> Self {
        Self(ErrorKind::Context(v))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            ErrorKind::UnsupportedSurface => write!(f, "unsupported surface"),
            ErrorKind::EventLoop(err) => err.fmt(f),
            ErrorKind::Os(err) => err.fmt(f),
            ErrorKind::Surface(err) => err.fmt(f),
            ErrorKind::Context(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            ErrorKind::UnsupportedSurface => None,
            ErrorKind::EventLoop(err) => Some(err),
            ErrorKind::Os(err) => Some(err),
            ErrorKind::Surface(err) => Some(err),
            ErrorKind::Context(err) => Some(err),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    UnsupportedSurface,
    EventLoop(EventLoopError),
    Os(OsError),
    Surface(CreateSurfaceError),
    Context(context::Error),
}
