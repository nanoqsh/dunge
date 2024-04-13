use crate::{
    draw::Draw,
    el::{Control, Flow},
    state::Frame,
};

/// The update stage.
pub trait Update: Draw {
    type Flow: Flow;
    type Event;
    fn update(&mut self, ctrl: &Control) -> Self::Flow;
    fn event(&mut self, _: Self::Event) {}
}

/// Helper function to create a [`Update`]
/// implementer from functions.
pub fn update<U, F, D>(mut upd: U, draw: D) -> impl Update<Event = ()>
where
    U: FnMut(&Control) -> F,
    F: Flow,
    D: Fn(Frame),
{
    update_event(
        (),
        move |(), ctrl| upd(ctrl),
        |(), ()| {},
        move |(), frame| draw(frame),
    )
}

/// Same as [`update`](fn@crate::update) but with
/// a state shared between two handlers.
pub fn update_with_state<S, U, F, D>(state: S, upd: U, draw: D) -> impl Update<Event = ()>
where
    U: FnMut(&mut S, &Control) -> F,
    F: Flow,
    D: Fn(&S, Frame),
{
    update_event(state, upd, |_, ()| {}, draw)
}

/// Same as [`update`](fn@crate::update) but with
/// a state shared between two handlers and an event handler.
pub fn update_event<S, U, E, V, F, D>(state: S, upd: U, ev: E, draw: D) -> impl Update<Event = V>
where
    U: FnMut(&mut S, &Control) -> F,
    E: FnMut(&mut S, V),
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
