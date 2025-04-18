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

    let ws = dunge::from_element("root");
    if let Err(e) = run(ws).await {
        panic!("error: {e}");
    }
}
