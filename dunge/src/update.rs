use crate::{draw::Draw, el::Control, state::Frame};

pub trait Update: Draw {
    fn update(&mut self, ctrl: &Control);
}

pub fn from_fn<U, D>(update: U, draw: D) -> impl Update
where
    U: FnMut(&Control),
    D: Fn(Frame),
{
    struct Func<U, D>(U, D);

    impl<U, D> Draw for Func<U, D>
    where
        D: Fn(Frame),
    {
        fn draw(&self, frame: Frame) {
            (self.1)(frame);
        }
    }

    impl<U, D> Update for Func<U, D>
    where
        U: FnMut(&Control),
        D: Fn(Frame),
    {
        fn update(&mut self, ctrl: &Control) {
            (self.0)(ctrl);
        }
    }

    Func(update, draw)
}
