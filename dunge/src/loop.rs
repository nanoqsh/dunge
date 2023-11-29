pub use winit::keyboard::KeyCode as Key;

use {
    crate::{context::Context, frame::Frame},
    std::{iter, slice},
};

/// The main application loop.
pub trait Loop {
    /// Calls before render a frame to update the state.
    ///
    /// It accepts the [`Context`] and the [`Input`].
    /// The context uses to update the application state, create or delete any resources.
    /// The input contains an inputed data like a mouse position and etc.
    fn update(&mut self, context: &mut Context, input: &Input);

    /// Calls on render a frame.
    ///
    /// It accepts a [`Frame`] to draw something on the canvas.
    fn render(&self, frame: &mut Frame);

    /// Calls when a close is requested.
    ///
    /// Returns a flag whether to terminate the main loop or not.
    fn close_requested(&mut self) -> bool {
        true
    }
}

impl<L> Loop for &mut L
where
    L: Loop,
{
    fn update(&mut self, context: &mut Context, input: &Input) {
        (**self).update(context, input);
    }

    fn render(&self, frame: &mut Frame) {
        (**self).render(frame);
    }

    fn close_requested(&mut self) -> bool {
        (**self).close_requested()
    }
}

impl<L> Loop for Box<L>
where
    L: Loop,
{
    fn update(&mut self, context: &mut Context, input: &Input) {
        self.as_mut().update(context, input);
    }

    fn render(&self, frame: &mut Frame) {
        self.as_ref().render(frame);
    }

    fn close_requested(&mut self) -> bool {
        self.as_mut().close_requested()
    }
}

/// User input.
pub struct Input<'a> {
    /// Seconds since previous [update](crate::Loop::update) was called.
    pub delta_time: f32,

    /// The cursor XY position on the screen.
    /// [`None`] if the cursor out of screen.
    pub cursor_position: Option<(f32, f32)>,

    /// The mouse input.
    pub mouse: Mouse,

    /// The pressed keys.
    pub pressed_keys: Keys<'a>,

    /// The released keys.
    pub released_keys: Keys<'a>,
}

/// The mouse input.
#[derive(Clone, Copy, Default)]
pub struct Mouse {
    pub motion_delta: (f32, f32),
    pub wheel_delta: (f32, f32),
    pub pressed_left: bool,
    pub pressed_middle: bool,
    pub pressed_right: bool,
}

/// Keys input.
#[derive(Clone, Copy)]
pub struct Keys<'a> {
    pub(crate) keys: &'a [Key],
}

impl<'a> IntoIterator for Keys<'a> {
    type Item = Key;
    type IntoIter = KeysIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        KeysIterator {
            iter: self.keys.iter().copied(),
        }
    }
}

/// An iterator over [keys](Key).
pub struct KeysIterator<'a> {
    iter: iter::Copied<slice::Iter<'a, Key>>,
}

impl Iterator for KeysIterator<'_> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ExactSizeIterator for KeysIterator<'_> {}
