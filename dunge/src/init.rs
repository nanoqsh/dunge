use crate::context::{Context, FailedMakeContext};

#[cfg(feature = "winit")]
use crate::{element::Element, window::WindowBuilder};

/// Creates the context instance.
///
/// If you need a window call the [`window`] function.
///
/// # Errors
/// Returns an error when the context could not be created.
/// See [`FailedMakeContext`] for details.
pub async fn context() -> Result<Context, FailedMakeContext> {
    Context::new().await
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
