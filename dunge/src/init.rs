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
    use wgpu::{Backends, InstanceDescriptor, InstanceFlags};

    let instance = {
        let desc = InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
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
