use winit::window;

/// The HTML canvas for the web platform.
pub struct Canvas(
    #[cfg(target_family = "wasm")] web_sys::HtmlCanvasElement,
    #[cfg(not(target_family = "wasm"))] std::convert::Infallible,
);

impl Canvas {
    /// Finds a HTML canvas on a web page by its id.
    ///
    /// Returns `None` if no element is found.
    /// Does nothing on non-web platforms.
    pub fn by_id(id: &str) -> Option<Self> {
        #[cfg(target_family = "wasm")]
        {
            use wasm_bindgen::JsCast;

            let document = web_sys::window()?.document()?;
            let element = document.get_element_by_id(id)?;
            let canvas = element.dyn_into().ok()?;
            Some(Self(canvas))
        }

        #[cfg(not(target_family = "wasm"))]
        {
            _ = id;
            None
        }
    }

    pub(crate) fn set(self, attr: window::WindowAttributes) -> window::WindowAttributes {
        #[cfg(target_family = "wasm")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;

            attr.with_canvas(Some(self.0))
        }

        #[cfg(not(target_family = "wasm"))]
        {
            attr
        }
    }
}

#[cfg(target_family = "wasm")]
impl From<web_sys::HtmlCanvasElement> for Canvas {
    fn from(html: web_sys::HtmlCanvasElement) -> Self {
        Self(html)
    }
}
