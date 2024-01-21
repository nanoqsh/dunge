use crate::state::Frame;

pub trait Draw {
    fn draw(&mut self, frame: Frame);
}

impl<D> Draw for &mut D
where
    D: Draw + ?Sized,
{
    fn draw(&mut self, frame: Frame) {
        (**self).draw(frame);
    }
}

pub fn from_fn<D>(draw: D) -> impl Draw
where
    D: FnMut(Frame),
{
    struct Func<D>(D);

    impl<D> Draw for Func<D>
    where
        D: FnMut(Frame),
    {
        fn draw(&mut self, frame: Frame) {
            (self.0)(frame);
        }
    }

    Func(draw)
}
