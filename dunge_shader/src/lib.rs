mod access;
mod branch;
mod context;
mod convert;
mod define;
mod discard;
mod dyn_expr;
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
        branch::*, context::*, convert::*, define::*, discard::*, dyn_expr::*, eval::*, math::*,
        matrix::*, module::*, op::*, texture::*, vector::*, zero::*,
    };
}
