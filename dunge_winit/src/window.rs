use {
    crate::{
        app::{Error, Request},
        canvas::Canvas,
    },
    dunge::{
        AsTarget, Context, Target,
        buffer::Format,
        surface::{Action, Output, Surface, WindowOps},
    },
    std::{
        borrow::Cow,
        cell::{Cell, RefCell},
        collections::{HashMap, hash_map::Entry},
        future,
        num::NonZeroU32,
        rc::Rc,
        sync::Arc,
        task::Poll,
        time::Duration,
    },
    winit::{event_loop, keyboard, window},
};

pub struct Attributes {
    title: Cow<'static, str>,
    canvas: Option<Canvas>,
}

impl Attributes {
    #[inline]
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Cow::Owned(title.into());
        self
    }

    #[inline]
    pub fn with_canvas<C>(mut self, canvas: C) -> Self
    where
        C: Into<Option<Canvas>>,
    {
        self.canvas = canvas.into();
        self
    }

    #[inline]
    fn winit(mut self) -> window::WindowAttributes {
        let mut attr = window::WindowAttributes::default().with_title(self.title);
        if let Some(canvas) = self.canvas.take() {
            attr = canvas.set(attr);
        }

        attr
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            title: Cow::Borrowed("dunge"),
            canvas: None,
        }
    }
}

pub(crate) struct Event<T = bool>(Cell<T>);

impl<T> Event<T> {
    #[inline]
    fn new() -> Self
    where
        T: Default,
    {
        Self(Cell::new(T::default()))
    }
}

impl Event {
    #[inline]
    pub(crate) fn set(&self) {
        self.0.set(true);
    }

    #[inline]
    fn active_poll(&self) -> Poll<()> {
        if self.0.take() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<T> Event<Option<T>> {
    #[inline]
    pub(crate) fn set_value(&self, value: T) {
        self.0.set(Some(value));
    }

    #[inline]
    fn active_poll_value(&self) -> Poll<T> {
        if let Some(value) = self.0.take() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

enum KeyState {
    Wait,
    Active,
}

pub(crate) struct Keys(RefCell<HashMap<keyboard::KeyCode, KeyState>>);

impl Keys {
    #[inline]
    fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    #[inline]
    fn wait(&self, code: keyboard::KeyCode) {
        self.0.borrow_mut().insert(code, KeyState::Wait);
    }

    #[inline]
    pub(crate) fn active(&self, code: keyboard::KeyCode) {
        if let Some(state @ KeyState::Wait) = self.0.borrow_mut().get_mut(&code) {
            *state = KeyState::Active;
        }
    }

    #[inline]
    fn active_poll(&self, code: keyboard::KeyCode) -> Poll<()> {
        if let Entry::Occupied(en) = self.0.borrow_mut().entry(code) {
            if let KeyState::Active = en.get() {
                en.remove();
                return Poll::Ready(());
            }
        }

        Poll::Pending
    }
}

pub(crate) struct Events {
    pub(crate) press_keys: Keys,
    pub(crate) release_keys: Keys,
    pub(crate) resize: Event,
    pub(crate) redraw: Event<Option<Duration>>,
    pub(crate) close: Event,
}

pub(crate) struct Shared {
    surface: Surface<window::Window, Ops>,
    min_delta_time: Cell<Duration>,
    events: Events,
}

impl Shared {
    #[inline]
    pub(crate) fn window(&self) -> &window::Window {
        self.surface.window()
    }

    #[inline]
    pub(crate) fn resize(&self, cx: &Context) {
        self.surface.resize(cx);
    }

    #[inline]
    pub(crate) fn min_delta_time(&self) -> Duration {
        self.min_delta_time.get()
    }

    #[inline]
    pub(crate) fn events(&self) -> &Events {
        &self.events
    }
}

#[derive(Clone)]
pub struct Window {
    shared: Rc<Shared>,
    req: Request,
}

impl Window {
    const DEFAULT_FPS: u32 = 60;

    #[inline]
    pub(crate) fn new(
        cx: &Context,
        req: Request,
        el: &event_loop::ActiveEventLoop,
        attr: Attributes,
    ) -> Result<Self, Error> {
        let window = el.create_window(attr.winit()).map_err(Error::Os)?;
        let shared = Rc::new(Shared {
            min_delta_time: Cell::new(Duration::from_secs_f32(1. / Self::DEFAULT_FPS as f32)),
            surface: Surface::new(cx, window).map_err(Error::CreateSurface)?,
            events: Events {
                press_keys: Keys::new(),
                release_keys: Keys::new(),
                resize: Event::new(),
                redraw: Event::new(),
                close: Event::new(),
            },
        });

        Ok(Self { shared, req })
    }

    #[inline]
    pub(crate) fn shared(&self) -> &Rc<Shared> {
        &self.shared
    }

    #[inline]
    pub fn winit(&self) -> &Arc<window::Window> {
        self.shared.surface.window()
    }

    #[inline]
    pub fn format(&self) -> Format {
        self.shared.surface.format()
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.shared.surface.size()
    }

    #[inline]
    pub fn set_fps(&self, fps: NonZeroU32) {
        self.shared
            .min_delta_time
            .set(Duration::from_secs_f32(1. / fps.get() as f32));
    }

    #[inline]
    pub async fn pressed(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.press_keys;
        keys.wait(code);
        future::poll_fn(|_| keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn released(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.release_keys;
        keys.wait(code);
        future::poll_fn(|_| keys.active_poll(code)).await;
    }

    #[inline]
    pub async fn resized(&self) -> (u32, u32) {
        future::poll_fn(|_| self.shared.events.resize.active_poll()).await;
        self.shared.surface.size()
    }

    #[inline]
    pub async fn redraw(&self) -> Redraw<'_> {
        loop {
            let delta_time =
                future::poll_fn(|_| self.shared.events.redraw.active_poll_value()).await;

            let e = match self.shared.surface.output() {
                Ok(output) => break Redraw { output, delta_time },
                Err(e) => e,
            };

            log::warn!("surface error: {e}");
            match e.action() {
                Action::Run => {}
                Action::Recreate => {
                    let id = self.shared.surface.window().id();
                    self.req.recreate_surface(id);
                }
                Action::Exit => self.req.exit(Error::Surface(e)),
            }
        }
    }

    #[inline]
    pub async fn close_requested(&self) {
        future::poll_fn(|_| self.shared.events.close.active_poll()).await;
    }
}

impl Drop for Window {
    #[inline]
    fn drop(&mut self) {
        let id = self.shared.surface.window().id();
        self.req.remove_window(id);
    }
}

pub struct Redraw<'surface> {
    output: Output<'surface>,
    delta_time: Duration,
}

impl Redraw<'_> {
    #[inline]
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    #[inline]
    pub fn present(self) {
        self.output.present();
    }
}

impl AsTarget for Redraw<'_> {
    #[inline]
    fn as_target(&self) -> Target<'_> {
        self.output.as_target()
    }
}

struct Ops;

impl WindowOps<window::Window> for Ops {
    #[inline]
    fn size(window: &window::Window) -> (u32, u32) {
        window.inner_size().into()
    }
}
