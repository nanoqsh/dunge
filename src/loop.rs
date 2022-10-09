use {
    crate::{context::Context, frame::Frame, Error},
    std::fmt,
};

/// The main application loop.
pub trait Loop {
    type Error: From<Error> + fmt::Debug;

    /// Calls before render a frame to update the state.
    ///
    /// It accepts the [`Context`] and an [`Input`].
    /// The context uses to update the application state, create or delete any resources.
    /// The input contains an inputed data like a mouse position and etc.
    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error>;

    /// Calls on render a frame.
    ///
    /// It accepts a [`Frame`] to draw something on the canvas.
    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error>;

    /// Calls when an error has occurred.
    fn error_occurred(&mut self, err: Self::Error) {
        log::error!("{err:?}");
    }
}

/// The user input data.
pub struct Input {
    pub delta_time: f32,
    pub cursor_position: Option<(f32, f32)>,
    pub mouse: Mouse,
}

/// The mouse input data.
#[derive(Clone, Copy, Default)]
pub struct Mouse {
    pub motion_delta: (f32, f32),
    pub wheel_delta: (f32, f32),
}
