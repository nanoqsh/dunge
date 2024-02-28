use crate::state::Frame;

/// The draw stage.
pub trait Draw {
    fn draw(&self, frame: Frame);
}

impl<D> Draw for &D
where
    D: Draw + ?Sized,
{
    fn draw(&self, frame: Frame) {
        (**self).draw(frame);
    }
}

/// Helper function to create a [`Draw`]
/// implementer from a function.
pub fn draw<D>(draw: D) -> impl Draw
where
    D: Fn(Frame),
{
    struct Func<D>(D);

    impl<D> Draw for Func<D>
    where
        D: Fn(Frame),
    {
        fn draw(&self, frame: Frame) {
            (self.0)(frame);
        }
    }

    Func(draw)
}
