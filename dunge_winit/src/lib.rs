mod app;
mod canvas;
mod time;
mod window;

/// The dunge prelude.
pub mod prelude {
    pub use {dunge, dunge::prelude::*};
}

/// Extension of the dunge with a windowing system.
pub mod winit {
    pub use crate::{
        app::{Control, Error},
        canvas::Canvas,
        window::{Attributes, Redraw, Window},
    };

    #[cfg(target_family = "wasm")]
    pub use crate::app::{run, try_run};

    #[cfg(not(target_family = "wasm"))]
    pub use crate::app::{block_on, try_block_on};
}

pub use dunge::*;
