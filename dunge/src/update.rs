use crate::{draw::Draw, el::Control, state::Frame};

pub trait Close {
    fn close(self) -> bool;
}

impl Close for () {
    fn close(self) -> bool {
        false
    }
}

pub trait Update: Draw {
    type Close: Close;
    fn update(&mut self, ctrl: &Control) -> Self::Close;
}

pub fn from_fn<U, C, D>(update: U, draw: D) -> impl Update
where
    U: FnMut(&Control) -> C,
    C: Close,
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

    impl<U, C, D> Update for Func<U, D>
    where
        U: FnMut(&Control) -> C,
        C: Close,
        D: Fn(Frame),
    {
        type Close = C;

        fn update(&mut self, ctrl: &Control) -> Self::Close {
            (self.0)(ctrl)
        }
    }

    Func(update, draw)
}
