#![cfg(target_family = "wasm")]

use {std::panic, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
async fn start() {
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

    if let Err(e) = dunge_winit::winit::try_run(run).await {
        panic!("error: {e}");
    }
}
