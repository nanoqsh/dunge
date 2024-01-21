use crate::{
    draw::Draw,
    el::{Control, Flow},
    state::Frame,
};

pub trait Update: Draw {
    type Flow: Flow;
    fn update(&mut self, ctrl: &Control) -> Self::Flow;
}

pub fn from_fn<U, F, D>(mut upd: U, mut draw: D) -> impl Update
where
    U: FnMut(&Control) -> F,
    F: Flow,
    D: FnMut(Frame),
{
    with_state((), move |(), ctrl| upd(ctrl), move |(), frame| draw(frame))
}

pub fn with_state<S, U, F, D>(state: S, upd: U, draw: D) -> impl Update
where
    U: FnMut(&mut S, &Control) -> F,
    F: Flow,
    D: FnMut(&S, Frame),
{
    struct Func<S, U, D>(S, U, D);

    impl<S, U, D> Draw for Func<S, U, D>
    where
        D: FnMut(&S, Frame),
    {
        fn draw(&mut self, frame: Frame) {
            (self.2)(&self.0, frame);
        }
    }

    impl<S, U, F, D> Update for Func<S, U, D>
    where
        U: FnMut(&mut S, &Control) -> F,
        F: Flow,
        D: FnMut(&S, Frame),
    {
        type Flow = F;

        fn update(&mut self, ctrl: &Control) -> Self::Flow {
            (self.1)(&mut self.0, ctrl)
        }
    }

    Func(state, upd, draw)
}
