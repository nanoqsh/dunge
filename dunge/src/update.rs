use crate::{
    draw::Draw,
    el::{Control, Flow},
    state::Frame,
};

pub trait Update: Draw {
    type Flow: Flow;
    fn update(&mut self, ctrl: &Control) -> Self::Flow;
}

pub fn from_fn<U, F, D>(update: U, draw: D) -> impl Update
where
    U: FnMut(&Control) -> F,
    F: Flow,
    D: FnMut(Frame),
{
    struct Func<U, D>(U, D);

    impl<U, D> Draw for Func<U, D>
    where
        D: FnMut(Frame),
    {
        fn draw(&mut self, frame: Frame) {
            (self.1)(frame);
        }
    }

    impl<U, F, D> Update for Func<U, D>
    where
        U: FnMut(&Control) -> F,
        F: Flow,
        D: FnMut(Frame),
    {
        type Flow = F;

        fn update(&mut self, ctrl: &Control) -> Self::Flow {
            (self.0)(ctrl)
        }
    }

    Func(update, draw)
}
