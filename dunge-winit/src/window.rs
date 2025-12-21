use {
    crate::{
        canvas::Canvas,
        runtime::{Error, Request},
    },
    dunge::{
        AsTarget, Context, Target,
        color::Format,
        surface::{Action, Output, Surface, WindowOps},
    },
    futures_lite::Stream,
    glam::{DVec2, UVec2},
    std::{
        cell::{Cell, RefCell},
        collections::{HashMap, VecDeque, hash_map::Entry},
        future,
        hash::Hash,
        iter, mem,
        num::NonZeroU32,
        ops,
        pin::Pin,
        rc::Rc,
        slice,
        sync::Arc,
        task::{self, Poll, Waker},
        time::Duration,
    },
    winit::{
        dpi, event, event_loop,
        keyboard::{self, SmolStr},
        window,
    },
};

/// The [window](Window) builder returned from
/// [`make_window`](crate::Control::make_window) method.
///
/// This builder provides shortcuts for some commonly used properties.
/// However, for more fine-grained configuration, you can directly specify
/// the [winit attributes](window::WindowAttributes) using the
/// [`with_winit`](WindowBuilder::with_winit) method.
pub struct WindowBuilder<'req> {
    req: &'req Request,
    cx: Context,
    attr: Attributes,
}

impl<'req> WindowBuilder<'req> {
    pub(crate) fn new(req: &'req Request, cx: Context) -> Self {
        Self {
            req,
            cx,
            attr: Attributes {
                canvas: None,
                winit: Box::new(window::WindowAttributes::default()),
            },
        }
    }

    /// Sets the window title.
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.attr.winit.title = title.into();
        self
    }

    /// Sets the window inner physical size.
    pub fn with_physical_size(mut self, width: u32, height: u32) -> Self {
        self.attr.winit.inner_size = Some(dpi::Size::Physical(dpi::PhysicalSize { width, height }));
        self
    }

    /// Sets the window [canvas](Canvas).
    pub fn with_canvas<C>(mut self, canvas: C) -> Self
    where
        C: Into<Option<Canvas>>,
    {
        self.attr.canvas = canvas.into();
        self
    }

    /// Sets the [winit attributes](window::WindowAttributes).
    pub fn with_winit<A>(mut self, winit: A) -> Self
    where
        A: Into<Box<window::WindowAttributes>>,
    {
        self.attr.winit = winit.into();
        self
    }
}

impl<'req> IntoFuture for WindowBuilder<'req> {
    type Output = Result<Window, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Window, Error>> + 'req>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.req.make_window(self.cx, self.attr))
    }
}

/// [Window] attributes.
pub(crate) struct Attributes {
    canvas: Option<Canvas>,
    // boxed to reduse sizeof
    winit: Box<window::WindowAttributes>,
}

impl Attributes {
    pub(crate) fn winit(mut self) -> Box<window::WindowAttributes> {
        let mut winit = *self.winit;
        if let Some(canvas) = self.canvas.take() {
            winit = canvas.set(winit);
        }

        Box::new(winit)
    }
}

pub(crate) struct Event<T = bool>(Cell<T>);

impl<T> Event<T> {
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

enum State {
    Wait,
    Active,
}

struct WaitState {
    state: State,
    waker: Option<Waker>,
}

impl WaitState {
    fn active(&mut self) {
        self.state = State::Active;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    fn poll(&mut self, cx: &mut task::Context<'_>) -> Poll<()> {
        match self.state {
            State::Wait => {
                match &mut self.waker {
                    Some(waker) => waker.clone_from(cx.waker()),
                    None => self.waker = Some(cx.waker().clone()),
                }

                Poll::Pending
            }
            State::Active => Poll::Ready(()),
        }
    }
}

enum Ids {
    Inline(u64),
    Vec(Vec<u64>),
}

impl Ids {
    fn push(&mut self, new: u64) {
        *self = match mem::replace(self, Self::Inline(0)) {
            Self::Inline(id) => Self::Vec(vec![id, new]),
            Self::Vec(mut ids) => {
                ids.push(new);
                Self::Vec(ids)
            }
        };
    }

    fn get(&self) -> &[u64] {
        match self {
            Self::Inline(id) => slice::from_ref(id),
            Self::Vec(ids) => ids,
        }
    }
}

pub(crate) struct EventMap<K> {
    waits: RefCell<HashMap<u64, WaitState>>,
    codes: RefCell<HashMap<K, Ids>>,
    id_counter: Cell<u64>,
}

impl<K> EventMap<K>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        Self {
            waits: RefCell::default(),
            codes: RefCell::default(),
            id_counter: Cell::default(),
        }
    }

    fn wait(&self, code: K) -> WaitFuture<'_> {
        let id = self.id_counter.get();
        self.id_counter.update(|id| id + 1);

        self.waits.borrow_mut().insert(
            id,
            WaitState {
                state: State::Wait,
                waker: None,
            },
        );

        self.codes
            .borrow_mut()
            .entry(code)
            .and_modify(|ids| ids.push(id))
            .or_insert(Ids::Inline(id));

        WaitFuture {
            waits: &self.waits,
            id,
        }
    }

    pub(crate) fn active(&self, code: K) {
        let mut waits = self.waits.borrow_mut();
        if let Some(ids) = self.codes.borrow_mut().get_mut(&code) {
            for id in ids.get() {
                if let Some(state) = waits.get_mut(id) {
                    state.active();
                }
            }
        }
    }
}

struct WaitFuture<'map> {
    waits: &'map RefCell<HashMap<u64, WaitState>>,
    id: u64,
}

impl Future for WaitFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        let mut waits = me.waits.borrow_mut();
        let Entry::Occupied(mut en) = waits.entry(me.id) else {
            debug_assert!(false, "polling after complition");
            return Poll::Ready(());
        };

        en.get_mut().poll(cx)
    }
}

impl Drop for WaitFuture<'_> {
    fn drop(&mut self) {
        self.waits.borrow_mut().remove(&self.id);
    }
}

struct StreamState<E> {
    queue: VecDeque<E>,
    waker: Option<Waker>,
}

impl<E> StreamState<E> {
    fn active(&mut self, new: E) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }

        self.queue.push_back(new);
    }

    fn poll(&mut self, cx: &mut task::Context<'_>) -> Poll<E> {
        if let Some(event) = self.queue.pop_front() {
            Poll::Ready(event)
        } else {
            match &mut self.waker {
                Some(waker) => waker.clone_from(cx.waker()),
                None => self.waker = Some(cx.waker().clone()),
            };

            Poll::Pending
        }
    }
}

pub(crate) struct EventStream<E> {
    waits: RefCell<HashMap<u64, StreamState<E>>>,
    id_counter: Cell<u64>,
}

impl<E> EventStream<E> {
    fn new() -> Self {
        Self {
            waits: RefCell::default(),
            id_counter: Cell::default(),
        }
    }

    fn wait(&self) -> WaitStream<'_, E> {
        let id = self.id_counter.get();
        self.id_counter.update(|id| id + 1);

        self.waits.borrow_mut().insert(
            id,
            StreamState {
                queue: VecDeque::new(),
                waker: None,
            },
        );

        WaitStream {
            waits: &self.waits,
            id,
        }
    }

    pub(crate) fn active(&self, event: E)
    where
        E: Clone,
    {
        let mut waits = self.waits.borrow_mut();
        let events = iter::repeat_n(event, waits.len());
        for (state, event) in iter::zip(waits.values_mut(), events) {
            state.active(event);
        }
    }
}

struct WaitStream<'map, E> {
    waits: &'map RefCell<HashMap<u64, StreamState<E>>>,
    id: u64,
}

impl<E> Stream for WaitStream<'_, E> {
    type Item = E;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();
        let mut waits = me.waits.borrow_mut();
        let Entry::Occupied(mut en) = waits.entry(me.id) else {
            return Poll::Ready(None);
        };

        en.get_mut().poll(cx).map(Some)
    }
}

impl<E> Drop for WaitStream<'_, E> {
    fn drop(&mut self) {
        self.waits.borrow_mut().remove(&self.id);
    }
}

pub(crate) type Buttons = EventMap<event::MouseButton>;
pub(crate) type Keys = EventMap<keyboard::KeyCode>;
pub(crate) type Text = EventStream<SmolStr>;

pub(crate) struct Events {
    pub(crate) press_buttons: Buttons,
    pub(crate) release_buttons: Buttons,
    pub(crate) press_keys: Keys,
    pub(crate) release_keys: Keys,
    pub(crate) text: Text,
    pub(crate) resize: Event,
    pub(crate) redraw: Event<Option<Duration>>,
    pub(crate) close: Event,
}

pub(crate) struct Shared {
    cx: Context,
    surface: Surface<window::Window, Ops>,
    events: Events,
    cursor_position: Cell<Option<DVec2>>,
}

impl Shared {
    pub(crate) fn window(&self) -> &window::Window {
        self.surface.window()
    }

    pub(crate) fn resize(&self) {
        self.surface.resize(&self.cx);
    }

    pub(crate) fn events(&self) -> &Events {
        &self.events
    }

    pub(crate) fn cursor_moved(&self, x: f64, y: f64) {
        self.cursor_position.set(Some(DVec2::new(x, y)));
    }

    pub(crate) fn cursor_left(&self) {
        self.cursor_position.set(None);
    }
}

/// A window within the running event loop.
#[derive(Clone)]
pub struct Window {
    shared: Rc<Shared>,
    req: Request,
}

impl Window {
    pub(crate) fn new(
        cx: Context,
        req: Request,
        el: &event_loop::ActiveEventLoop,
        attr: Box<window::WindowAttributes>,
    ) -> Result<Self, Error> {
        let window = el.create_window(*attr).map_err(Error::Os)?;
        let surface = Surface::new(&cx, window).map_err(Error::CreateSurface)?;

        let shared = Rc::new(Shared {
            cx,
            surface,
            events: Events {
                press_buttons: Buttons::new(),
                release_buttons: Buttons::new(),
                press_keys: Keys::new(),
                release_keys: Keys::new(),
                text: Text::new(),
                resize: Event::new(),
                redraw: Event::new(),
                close: Event::new(),
            },
            cursor_position: Cell::new(None),
        });

        Ok(Self { shared, req })
    }

    pub(crate) fn shared(&self) -> &Rc<Shared> {
        &self.shared
    }

    /// Returns the internal `winit` window.
    pub fn winit(&self) -> &Arc<window::Window> {
        self.shared.surface.window()
    }

    /// Returns the surface format of the window.
    pub fn format(&self) -> Format {
        self.shared.surface.format()
    }

    /// Returns the size of the window in pixels.
    pub fn size(&self) -> UVec2 {
        self.shared.surface.size().into()
    }

    /// Returns the cursor position on the window.
    pub fn cursor_position(&self) -> Option<DVec2> {
        self.shared.cursor_position.get()
    }

    /// Waits for a button press event.
    pub async fn button_pressed(&self, button: event::MouseButton) {
        let buttons = &self.shared.events.press_buttons;
        buttons.wait(button).await;
    }

    /// Waits for a button release event.
    pub async fn button_released(&self, button: event::MouseButton) {
        let buttons = &self.shared.events.release_buttons;
        buttons.wait(button).await;
    }

    /// Waits for a key press event.
    pub async fn key_pressed(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.press_keys;
        keys.wait(code).await;
    }

    /// Waits for a key release event.
    pub async fn key_released(&self, code: keyboard::KeyCode) {
        let keys = &self.shared.events.release_keys;
        keys.wait(code).await;
    }

    /// Reads a text input from keyboard.
    pub fn text_input(&self) -> impl Stream<Item = SmolStr> {
        self.shared.events.text.wait()
    }

    /// Waits for a window resize event.
    pub async fn resized(&self) -> UVec2 {
        future::poll_fn(|_| self.shared.events.resize.active_poll()).await;

        self.shared.surface.size().into()
    }

    /// Waits for a redraw event.
    pub async fn redraw(&self) -> Redraw<'_> {
        loop {
            let delta_time = future::poll_fn(|cx| {
                cx.waker().wake_by_ref();
                self.shared.events.redraw.active_poll_value()
            })
            .await;

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
    pub async fn close_requested(&self) {
        future::poll_fn(|_| self.shared.events.close.active_poll()).await;
    }

    pub fn set_fps(&self, fps: NonZeroU32) {
        const NANO: u32 = 1_000_000_000;

        let id = self.shared.surface.window().id();
        let duration = Duration::from_nanos(u64::from(NANO / fps));
        self.req.set_fps(id, duration);
    }
}

impl Drop for Window {
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
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Presents the redrawed frame on the screen.
    pub fn present(self) {
        self.output.present();
    }
}

impl AsTarget for Redraw<'_> {
    fn as_target(&self) -> Target<'_> {
        self.output.as_target()
    }
}

struct Ops;

impl WindowOps<window::Window> for Ops {
    fn size(window: &window::Window) -> (u32, u32) {
        window.inner_size().into()
    }
}
