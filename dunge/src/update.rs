use crate::{
    context::Context,
    draw::Draw,
    el::{Control, Flow},
    state::Frame,
    window::View,
};

/// The update stage.
///
/// This trait handles the application's state updation by taking a [control](Control)
/// object and returns a variant of the execution [control flow](Flow).
/// It also allows you to handle arbitrary custom [events](Update::event).
///
/// # Example
/// ```rust
/// use dunge::{Draw, Control, Frame, KeyCode, Then, Update};
///
/// struct App {/***/}
///
/// impl Draw for App {
///     fn draw(&self, mut frame: Frame) {/***/}
/// }
///
/// impl Update for App {
///     type Flow = Then;
///     type Event = ();
///
///     fn update(&mut self, ctrl: &Control) -> Then {
///         for key in ctrl.pressed_keys() {
///             // Exit by pressing escape key
///             if key.code == KeyCode::Escape {
///                 return Then::Close;
///             }
///         }
///
///         // Otherwise continue running
///         Then::Run
///     }
/// }
/// ```
///
/// Instead of manually implementing the trait, you can use an [helper](update)
/// function that will make an implementation from a closure:
/// ```rust
/// use dunge::{Control, Frame, KeyCode, Then, Update};
///
/// fn make_update() -> impl Update {
///     let draw = |frame: Frame| {/***/};
///     let upd = |ctrl: &Control| -> Then {
///         for key in ctrl.pressed_keys() {
///             if key.code == KeyCode::Escape {
///                 return Then::Close;
///             }
///         }
///
///         Then::Run
///     };
///
///     dunge::update(upd, draw)
/// }
/// ```
///
/// # Shared state
/// Draw and update stages may share some state.
/// This is not a problem to implement traits manually for some type.
/// However, to be able to use helpers, dunge has the [`update_with_state`] function.
/// ```rust
/// use dunge::{Control, Frame, Update};
///
/// struct State { counter: usize }
///
/// fn make_update() -> impl Update {
///     let draw = |state: &State, frame: Frame| {
///         dbg!(state.counter);
///     };
///
///     let upd = |state: &mut State, ctrl: &Control| {
///         state.counter += 1;
///     };
///
///     let state = State { counter: 0 };
///     dunge::update_with_state(state, upd, draw)
/// }
/// ```
///
/// Also see the [`update_with_event`] function to set a custom event handler.
///
pub trait Update: Draw {
    type Flow: Flow;
    type Event: 'static;
    fn update(&mut self, ctrl: &Control) -> Self::Flow;
    fn event(&mut self, _: Self::Event) {}
}

/// Helper function to create a [`Update`]
/// implementer from functions.
pub fn update<U, F, D>(mut upd: U, draw: D) -> impl Update<Flow = F, Event = ()>
where
    U: FnMut(&Control) -> F,
    F: Flow,
    D: Fn(Frame),
{
    update_with_event(
        (),
        move |(), ctrl| upd(ctrl),
        |(), ()| {},
        move |(), frame| draw(frame),
    )
}

/// Same as [`update`](fn@crate::update) but with
/// a state shared between two handlers.
pub fn update_with_state<S, U, F, D>(state: S, upd: U, draw: D) -> impl Update<Flow = F, Event = ()>
where
    U: FnMut(&mut S, &Control) -> F,
    F: Flow,
    D: Fn(&S, Frame),
{
    update_with_event(state, upd, |_, ()| {}, draw)
}

/// Same as [`update`](fn@crate::update) but with
/// a state shared between two handlers and an event handler.
pub fn update_with_event<S, U, E, V, F, D>(
    state: S,
    upd: U,
    ev: E,
    draw: D,
) -> impl Update<Flow = F, Event = V>
where
    U: FnMut(&mut S, &Control) -> F,
    E: FnMut(&mut S, V),
    V: 'static,
    F: Flow,
    D: Fn(&S, Frame),
{
    use std::marker::PhantomData;

    struct Func<S, U, E, V, D> {
        state: S,
        upd: U,
        ev: E,
        draw: D,
        evty: PhantomData<V>,
    }

    impl<S, U, E, V, D> Draw for Func<S, U, E, V, D>
    where
        D: Fn(&S, Frame),
    {
        fn draw(&self, frame: Frame) {
            (self.draw)(&self.state, frame);
        }
    }

    impl<S, U, E, V, F, D> Update for Func<S, U, E, V, D>
    where
        U: FnMut(&mut S, &Control) -> F,
        E: FnMut(&mut S, V),
        V: 'static,
        F: Flow,
        D: Fn(&S, Frame),
    {
        type Flow = F;
        type Event = V;

        fn update(&mut self, ctrl: &Control) -> Self::Flow {
            (self.upd)(&mut self.state, ctrl)
        }

        fn event(&mut self, ev: Self::Event) {
            (self.ev)(&mut self.state, ev);
        }
    }

    Func {
        state,
        upd,
        ev,
        draw,
        evty: PhantomData,
    }
}

/// Lazy instantiation of the value that implements the [`Update`] trait.
///
/// It is usually more convenient to create a handler after the [view](View)
/// has been initialized. This will let you know the current properties of
/// the window like [size](View::size) and [format](View::format).
pub trait IntoUpdate: Sized {
    type Flow: Flow;
    type Event: 'static;
    type Update: Update<Flow = Self::Flow, Event = Self::Event>;

    /// Creates an [update](Update) handler from the [context](Context) and [view](View).
    fn into_update(self, cx: &Context, view: &View) -> Self::Update;
}

impl<U> IntoUpdate for U
where
    U: Update,
{
    type Flow = U::Flow;
    type Event = U::Event;
    type Update = Self;

    fn into_update(self, _: &Context, _: &View) -> Self::Update {
        self
    }
}

/// Creates an [update](Update) handler from the received [context](Context) and [view](View).
pub fn make<F, U>(f: F) -> impl IntoUpdate<Flow = U::Flow, Event = U::Event>
where
    F: FnOnce(&Context, &View) -> U,
    U: Update,
{
    struct FromFn<F>(F);

    impl<F, U> IntoUpdate for FromFn<F>
    where
        F: FnOnce(&Context, &View) -> U,
        U: Update,
    {
        type Flow = U::Flow;
        type Event = U::Event;
        type Update = U;

        fn into_update(self, cx: &Context, view: &View) -> Self::Update {
            self.0(cx, view)
        }
    }

    FromFn(f)
}
