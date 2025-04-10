use crate::state::Frame;

/// The draw stage.
///
/// This trait handles frame render by taking the [frame](Frame) to execute drawing commands.
///
/// # Example
/// ```rust
/// use dunge::{Draw, Frame, layer::Layer, color::Rgba};
///
/// struct App {
///     bg: Rgba,
///     layer: Layer<((), ())>,
/// }
///
/// impl Draw for App {
///     fn draw(&self, mut frame: Frame) {
///         frame
///             // set a layer
///             .set_layer(&self.layer, self.bg)
///             // without any binding
///             .bind_empty()
///             // draw some triangle
///             .draw_points(3);
///     }
/// }
/// ```
///
/// Instead of manually implementing the trait, you can use an [helper](draw)
/// function that will make an implementation from a closure:
/// ```rust
/// use dunge::{Draw, Frame, layer::Layer, color::Rgba};
///
/// fn make_draw(bg: Rgba, layer: Layer<((), ())>) -> impl Draw {
///     dunge::draw(move |mut frame: Frame| {
///         frame
///             .set_layer(&layer, bg)
///             .bind_empty()
///             .draw_points(3);
///     })
/// }
/// ```
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
