use crate::state::Frame;

pub trait Draw {
    fn draw(&self, frame: Frame);
}

impl<D> Draw for &D
where
    D: Draw + ?Sized,
{
    fn draw(&self, frame: Frame) {
        (**self).draw(frame)
    }
}

pub fn from_fn<F>(f: F) -> impl Draw
where
    F: Fn(Frame),
{
    struct Func<F>(F);

    impl<F> Draw for Func<F>
    where
        F: Fn(Frame),
    {
        fn draw(&self, frame: Frame) {
            (self.0)(frame);
        }
    }

    Func(f)
}
