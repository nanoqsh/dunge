use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use web_sys::Element as El;

#[cfg(not(target_arch = "wasm32"))]
type El = ();

pub(crate) struct Element(pub El);

impl Element {
    pub fn set_canvas(&self, window: &Window) {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas().expect("get canvas");
            canvas.remove_attribute("style").expect("remove attribute");
            self.0.append_child(&canvas).expect("append child");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            _ = window;
        }
    }

    pub fn set_window_size(&self, window: &Window) {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::dpi::PhysicalSize;

            let new_size = {
                let width = self.0.client_width().max(1) as u32;
                let height = self.0.client_height().max(1) as u32;
                PhysicalSize { width, height }
            };

            if new_size == window.inner_size() {
                return;
            }

            window.set_max_inner_size(Some(new_size));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            _ = window;
        }
    }
}
