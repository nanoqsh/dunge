use crate::{
    draw::Draw,
    el::{Control, Flow},
    state::Frame,
};

/// The update stage.
pub trait Update: Draw {
    type Flow: Flow;
    fn update(&mut self, ctrl: &Control) -> Self::Flow;
}

/// Helper function to create a [`Update`]
/// implementer from functions.
pub fn update<U, F, D>(mut upd: U, draw: D) -> impl Update
where
    U: FnMut(&Control) -> F,
    F: Flow,
    D: Fn(Frame),
{
    update_with((), move |(), ctrl| upd(ctrl), move |(), frame| draw(frame))
}

/// Same as [`undate`](fn@crate::update) but with
/// a state shared between two handlers.
pub fn update_with<S, U, F, D>(state: S, upd: U, draw: D) -> impl Update
where
    U: FnMut(&mut S, &Control) -> F,
    F: Flow,
    D: Fn(&S, Frame),
{
    struct Func<S, U, D>(S, U, D);

    impl<S, U, D> Draw for Func<S, U, D>
    where
        D: Fn(&S, Frame),
    {
        fn draw(&self, frame: Frame) {
            (self.2)(&self.0, frame);
        }
    }

    impl<S, U, F, D> Update for Func<S, U, D>
    where
        U: FnMut(&mut S, &Control) -> F,
        F: Flow,
        D: Fn(&S, Frame),
    {
        type Flow = F;

        fn update(&mut self, ctrl: &Control) -> Self::Flow {
            (self.1)(&mut self.0, ctrl)
        }
    }

    Func(state, upd, draw)
}
