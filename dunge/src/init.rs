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

/// Creates the context instance.
///
/// If you need a window call the [`window`] function.
///
/// # Errors
/// Returns an error when the context could not be created.
/// See [`Error`](context::Error) for details.
pub async fn context() -> Result<Context, context::Error> {
    make().await.map(|(cx, _)| cx)
}

/// Creates the [window builder](WindowBuilder) to
/// construct the [window](crate::window::Window).
///
/// # Example
/// ```rust
/// # fn t() -> impl std::future::Future<Output = Result<dunge::window::Window, dunge::window::Error>> {
/// async {
///     let window = dunge::window().with_title("Hello").await?;
///     Ok(window)
/// }
/// # }
/// ```
#[cfg(feature = "winit")]
pub fn window() -> WindowBuilder {
    WindowBuilder::new()
}
