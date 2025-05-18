mod canvas;
mod reactor;
mod runtime;
mod window;

/// The dunge prelude.
pub mod prelude {
    pub use {
        crate::{
            reactor::{DurationTimerExt as _, InstantTimerExt as _},
            runtime::Control,
            window::Attributes,
        },
        dunge,
        dunge::prelude::*,
    };
}

/// Extension of the dunge with a windowing system.
pub mod winit {
    pub use crate::{
        canvas::Canvas,
        reactor::{DurationTimerExt, InstantTimerExt, Timer},
        runtime::{Control, Error},
        window::{Attributes, Redraw, Window},
    };

    #[cfg(target_family = "wasm")]
    pub use crate::runtime::{run, try_run};

    #[cfg(not(target_family = "wasm"))]
    pub use crate::runtime::{block_on, try_block_on};
}

pub use dunge::*;
