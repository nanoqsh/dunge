#![cfg(target_family = "wasm")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn start() {
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let run;

    #[cfg(feature = "cube")]
    {
        run = cube::run;
    }

    #[cfg(feature = "ssaa")]
    {
        run = ssaa::run;
    }

    #[cfg(feature = "triangle")]
    {
        run = triangle::run;
    }

    let window = dunge::from_element("root").await;
    if let Err(err) = window.map_err(Box::from).and_then(run) {
        panic!("error: {err}");
    }
}
