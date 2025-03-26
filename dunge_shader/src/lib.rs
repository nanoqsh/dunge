mod access;
mod branch;
mod context;
mod convert;
mod cs_module;
mod define;
mod discard;
mod eval;
pub mod group;
pub mod instance;
mod math;
mod matrix;
mod module;
mod op;
mod texture;
pub mod types;
mod vector;
pub mod vertex;
mod zero;

pub mod sl {
    //! Shader generator functions.

    pub use crate::{
        branch::*, context::*, convert::*, cs_module::*, define::*, discard::*, eval::*, math::*,
        matrix::*, module::*, op::*, texture::*, vector::*, zero::*,
    };
}
