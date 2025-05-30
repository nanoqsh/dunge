mod access;
mod branch;
mod context;
mod convert;
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

/// Shader generator functions.
pub mod sl {
    pub use crate::{
        branch::*, context::*, convert::*, define::*, discard::*, eval::*, math::*, matrix::*,
        module::*, op::*, texture::*, vector::*, zero::*,
    };
}
