use {std::ops, winit::window::Window};

#[cfg(target_arch = "wasm32")]
use web_sys::Element as El;

#[cfg(not(target_arch = "wasm32"))]
type El = ();

#[cfg_attr(not(target_arch = "wasm32"), derive(Default))]
pub(crate) struct Element(El);

impl Element {
    #[cfg(target_arch = "wasm32")]
    pub fn new(inner: El) -> Self {
        Self(inner)
    }

    pub fn set_window_size(&self, window: &Window) {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::dpi::PhysicalSize;

            let new_size = {
                let width = self.client_width().max(1) as u32;
                let height = self.client_height().max(1) as u32;
                PhysicalSize { width, height }
            };

            if new_size == window.inner_size() {
                return;
            }

            window.set_inner_size(new_size);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            _ = self;
            _ = window;
        }
    }
}

impl ops::Deref for Element {
    type Target = El;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
