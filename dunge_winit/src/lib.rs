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
        },
        dunge,
        dunge::prelude::*,
        winit,
    };
}

pub use {
    crate::{
        canvas::Canvas,
        reactor::{DurationTimerExt, InstantTimerExt, Timer},
        runtime::{Control, Error},
        window::{Redraw, Window, WindowBuilder},
    },
    dunge, winit,
};

#[cfg(target_family = "wasm")]
pub use crate::runtime::{run, try_run};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::{block_on, try_block_on};
