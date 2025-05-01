mod app;
mod canvas;
mod time;
mod window;

pub mod prelude {
    pub use {dunge, dunge::prelude::*};
}

pub mod winit {
    pub use crate::{
        app::{Control, Error},
        canvas::Canvas,
        window::{Attributes, Window},
    };

    #[cfg(target_family = "wasm")]
    pub use crate::app::{run, try_run};

    #[cfg(not(target_family = "wasm"))]
    pub use crate::app::{block_on, try_block_on};
}

pub use dunge::*;
