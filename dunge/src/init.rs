use {
    crate::{
        context::{self, Context},
        state::State,
    },
    wgpu::Instance,
};

#[cfg(feature = "winit")]
use crate::window::{self, Window, WindowBuilder};

fn instance() -> Instance {
    use wgpu::{Backends, InstanceDescriptor};

    let desc = InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    };

    Instance::new(desc)
}

pub async fn context() -> Result<Context, context::Error> {
    let instance = instance();
    let state = State::new(&instance).await?;
    Ok(Context::new(state))
}

#[cfg(feature = "winit")]
pub fn window() -> WindowBuilder {
    WindowBuilder::new()
}

#[cfg(feature = "winit")]
impl WindowBuilder {
    pub async fn make(self) -> Result<Window, window::Error> {
        let instance = instance();
        let state = State::new(&instance).await?;
        let cx = Context::new(state);
        self.build(cx, &instance)
    }
}
