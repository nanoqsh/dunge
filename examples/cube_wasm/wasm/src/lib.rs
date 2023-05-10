use {cube::App, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn run() {
    use {dunge::CanvasConfig, std::panic};

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    dunge::from_element("root")
        .run(CanvasConfig::default(), App::new)
        .await
        .into_panic();
}
