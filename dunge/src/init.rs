use {
    crate::{
        context::{self, Context},
        state::State,
    },
    wgpu::Instance,
};

#[cfg(feature = "winit")]
use crate::window::WindowBuilder;

pub(crate) async fn make() -> Result<(Context, Instance), context::Error> {
    use wgpu::{Backends, InstanceDescriptor};

    let instance = {
        let desc = InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        };

        Instance::new(desc)
    };

    let state = State::new(&instance).await?;
    Ok((Context::new(state), instance))
}

pub async fn context() -> Result<Context, context::Error> {
    make().await.map(|(cx, _)| cx)
}

#[cfg(feature = "winit")]
pub fn window() -> WindowBuilder {
    WindowBuilder::new()
}
