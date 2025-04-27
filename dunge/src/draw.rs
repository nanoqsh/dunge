use crate::state::_Frame;

/// The draw stage.
///
/// This trait handles frame render by taking the [frame](Frame) to execute drawing commands.
///
/// # Example
/// ```rust
/// use dunge::{Draw, _Frame, layer::Layer, render::Input, color::Rgba};
///
/// struct App {
///     bg: Rgba,
///     layer: Layer<Input<(), (), ()>>,
/// }
///
/// impl Draw for App {
///     fn draw(&self, mut frame: _Frame) {
///         frame
///             // set a layer
///             ._set_layer(&self.layer, self.bg)
///             // without any binding
///             ._bind_empty()
///             // draw some triangle
///             ._draw_points(3);
///     }
/// }
/// ```
///
/// Instead of manually implementing the trait, you can use an [helper](draw)
/// function that will make an implementation from a closure:
/// ```rust
/// use dunge::{Draw, _Frame, layer::Layer, render::Input, color::Rgba};
///
/// fn make_draw(bg: Rgba, layer: Layer<Input<(), (), ()>>) -> impl Draw {
///     dunge::draw(move |mut frame: _Frame| {
///         frame
///             ._set_layer(&layer, bg)
///             ._bind_empty()
///             ._draw_points(3);
///     })
/// }
/// ```
pub trait Draw {
    fn draw(&self, frame: _Frame<'_, '_>);
}

impl<D> Draw for &D
where
    D: Draw + ?Sized,
{
    fn draw(&self, frame: _Frame<'_, '_>) {
        (**self).draw(frame);
    }
}

/// Helper function to create a [`Draw`]
/// implementer from a function.
pub fn draw<D>(draw: D) -> impl Draw
where
    D: Fn(_Frame<'_, '_>),
{
    struct Func<D>(D);

    impl<D> Draw for Func<D>
    where
        D: Fn(_Frame<'_, '_>),
    {
        fn draw(&self, frame: _Frame<'_, '_>) {
            (self.0)(frame);
        }
    }

    Func(draw)
}
