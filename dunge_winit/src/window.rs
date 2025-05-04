use {
    crate::{
        canvas::Canvas,
        runtime::{Error, Request},
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
        ops,
        rc::Rc,
        sync::Arc,
        task::Poll,
        time::Duration,
    },
    winit::{event_loop, keyboard, window},
};

/// [Window] attributes.
pub struct Attributes {
    title: Cow<'static, str>,
    canvas: Option<Canvas>,
}

impl Attributes {
    /// Sets the window title.
    #[inline]
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Cow::Owned(title.into());
        self
    }

    /// Sets the window [canvas](Canvas).
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
    pub(crate) fn add_value(&self, value: T)
    where
        T: Copy + ops::AddAssign,
    {
        match self.0.get() {
            Some(mut curr) => {
                curr += value;
                self.0.set(Some(curr));
            }
            None => self.0.set(Some(value)),
        }
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
    cx: Context,
    surface: Surface<window::Window, Ops>,
    events: Events,
}

impl Shared {
    #[inline]
    pub(crate) fn window(&self) -> &window::Window {
        self.surface.window()
    }

    #[inline]
    pub(crate) fn resize(&self) {
        self.surface.resize(&self.cx);
    }

    #[inline]
    pub(crate) fn events(&self) -> &Events {
        &self.events
    }
}

/// A window within the running event loop.
#[derive(Clone)]
pub struct Window {
    shared: Rc<Shared>,
    req: Request,
}

impl Window {
    #[inline]
    pub(crate) fn new(
        cx: Context,
        req: Request,
        el: &event_loop::ActiveEventLoop,
        attr: Attributes,
    ) -> Result<Self, Error> {
        let window = el.create_window(attr.winit()).map_err(Error::Os)?;
        let surface = Surface::new(&cx, window).map_err(Error::CreateSurface)?;

        let shared = Rc::new(Shared {
            cx,
            surface,
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

    /// Returns the internal `winit` window.
    #[inline]
    pub fn winit(&self) -> &Arc<window::Window> {
        self.shared.surface.window()
    }

    /// Returns the surface format of the window.
    #[inline]
    pub fn format(&self) -> Format {
        self.shared.surface.format()
    }

    /// Returns the size of the window in pixels.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.shared.surface.size()
    }

    /// Waits for a key press event.
    #[inline]
    pub async fn pressed(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.press_keys;
        keys.wait(code);
        future::poll_fn(|_| keys.active_poll(code)).await;
    }

    /// Waits for a key release event.
    #[inline]
    pub async fn released(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.release_keys;
        keys.wait(code);
        future::poll_fn(|_| keys.active_poll(code)).await;
    }

    /// Waits for a window resize event.
    #[inline]
    pub async fn resized(&self) -> (u32, u32) {
        future::poll_fn(|_| self.shared.events.resize.active_poll()).await;
        self.shared.surface.size()
    }

    /// Waits for a redraw event.
    #[inline]
    pub async fn redraw(&self) -> Redraw<'_> {
        loop {
            let delta_time =
                future::poll_fn(|_| self.shared.events.redraw.active_poll_value()).await;

            let e = match self.shared.surface.output() {
                Ok(output) => break Redraw { output, delta_time },
                Err(e) => e,
            };

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

    /// Waits for a window close request event.
    #[inline]
    pub async fn close_requested(&self) {
        future::poll_fn(|_| self.shared.events.close.active_poll()).await;
    }

    #[inline]
    pub fn set_fps(&self, fps: NonZeroU32) {
        const NANO: u32 = 1_000_000_000;

        let id = self.shared.surface.window().id();
        let duration = Duration::from_nanos(u64::from(NANO / fps));
        self.req.set_fps(id, duration);
    }
}

impl Drop for Window {
    #[inline]
    fn drop(&mut self) {
        let id = self.shared.surface.window().id();
        self.req.remove_window(id);
    }
}

/// An object for frame redrawing.
pub struct Redraw<'surface> {
    output: Output<'surface>,
    delta_time: Duration,
}

impl Redraw<'_> {
    /// Returns the delta time since the last redraw.
    #[inline]
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Presents the redrawed frame on the screen.
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
