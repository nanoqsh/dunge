use {
    crate::{
        context::Context,
        state::{Render, RenderView},
        time::{Fps, Time},
        update::Update,
        window::View,
    },
    std::{error, fmt},
    wgpu::SurfaceError,
    winit::{
        error::EventLoopError,
        event,
        event_loop::{self, EventLoop},
        keyboard,
    },
};

pub(crate) struct Loop(pub EventLoop<()>);

impl Loop {
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
        event::{KeyEvent, StartCause, WindowEvent},
        event_loop::ControlFlow,
        keyboard::{KeyCode, PhysicalKey},
        std::time::Duration,
    };

    let mut render = Render::default();
    let mut time = Time::now();
    let mut fps = Fps::default();
    move |ev: Event, target: &Target| match ev {
        Event::NewEvents(cause) => match cause {
            StartCause::ResumeTimeReached { .. } => view.request_redraw(),
            StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                let flow = match requested_resume {
                    Some(resume) => ControlFlow::WaitUntil(resume),
                    None => {
                        const WAIT_TIME: Duration = Duration::from_millis(100);

                        ControlFlow::wait_duration(WAIT_TIME)
                    }
                };

                target.set_control_flow(flow)
            }
            StartCause::Poll => {}
            StartCause::Init => {}
        },
        Event::WindowEvent { event, window_id } if window_id == view.id() => match event {
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
                        output.present();
                    }
                    Err(SurfaceError::Lost) => view.resize(cx.state()),
                    Err(SurfaceError::OutOfMemory) => target.exit(),
                    Err(err) => eprintln!("{err:?}"),
                }
            }
            _ => {}
        },
        Event::Suspended => {}
        Event::Resumed => view.request_redraw(),
        _ => {}
    }
}
