use {
    crate::{
        context::{Context, Error},
        frame::Frame,
    },
    std::fmt,
};

pub trait Loop {
    type Error: From<Error> + fmt::Debug;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error>;
    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error>;
}

pub struct Input {
    pub delta_time: f32,
    pub cursor_position: Option<(f32, f32)>,
    pub mouse: Mouse,
}

#[derive(Clone, Copy, Default)]
pub struct Mouse {
    pub motion_delta: (f32, f32),
    pub wheel_delta: (f32, f32),
}
