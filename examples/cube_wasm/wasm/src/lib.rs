use {cube::App, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn run() {
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    dunge::from_element("root").run(App::new).await;
}
