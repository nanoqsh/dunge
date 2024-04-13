use {
    crate::{
        context::{Context, FailedMakeContext},
        state::State,
    },
    wgpu::Instance,
};

#[cfg(feature = "winit")]
use crate::{element::Element, window::WindowBuilder};

pub(crate) async fn make() -> Result<(Context, Instance), FailedMakeContext> {
    use wgpu::{Backends, InstanceDescriptor, InstanceFlags};

    let backends;

    #[cfg(any(target_family = "unix", target_family = "windows"))]
    {
        backends = Backends::VULKAN;
    }

    #[cfg(target_family = "wasm")]
    {
        backends = Backends::BROWSER_WEBGPU;
    }

    let instance = {
        let desc = InstanceDescriptor {
            backends,
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
/// See [`FailedMakeContext`] for details.
pub async fn context() -> Result<Context, FailedMakeContext> {
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
#[cfg(all(feature = "winit", not(target_arch = "wasm32")))]
pub fn window<V>() -> WindowBuilder<V> {
    WindowBuilder::new(Element(()))
}

/// Creates the [window builder](WindowBuilder) to
/// construct the [window](crate::window::Window)
/// in the given html element.
#[cfg(all(feature = "winit", target_arch = "wasm32"))]
pub fn from_element<V>(id: &str) -> WindowBuilder<V> {
    use web_sys::Window;

    let document = web_sys::window()
        .as_ref()
        .and_then(Window::document)
        .expect("get document");

    let Some(inner) = document.get_element_by_id(id) else {
        panic!("an element with id {id:?} not found");
    };

    let element = Element(inner);
    WindowBuilder::new(element)
}
