mod access;
mod branch;
mod context;
mod convert;
mod define;
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

pub mod sl {
    //! Shader generator functions.

    pub use crate::{
        branch::*, context::*, convert::*, define::*, eval::*, math::*, matrix::*, module::*,
        op::*, texture::*, vector::*,
    };
}
