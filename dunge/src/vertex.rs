//! The vertex module.

use crate::{
    sl::{ReadVertex, Ret},
    types::{self, VectorType},
};

pub use dunge_shader::vertex::{Projection, verts_as_bytes};

/// Describes an input type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait InputProjection: f::F32Aligned {
    const TYPE: VectorType;
    type Field;
    fn input_projection(id: u32, index: u32) -> Self::Field;
}

impl f::F32Aligned for [f32; 2] {}

impl InputProjection for [f32; 2] {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadVertex, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for [f32; 3] {}

impl InputProjection for [f32; 3] {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadVertex, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for [f32; 4] {}

impl InputProjection for [f32; 4] {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadVertex, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for glam::Vec2 {}

impl InputProjection for glam::Vec2 {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadVertex, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for glam::Vec3 {}

impl InputProjection for glam::Vec3 {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadVertex, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

#[cfg(any())]
mod ignore {
    // glam::Vec4 is not f32 aligned
    impl !InputProjection for glam::Vec4 {}
}

pub const fn check_projection_type<P>()
where
    P: f::F32Aligned,
{
    assert!(
        align_of::<P>() == align_of::<f32>(),
        "the type must be f32 aligned",
    );
}

mod f {
    pub trait F32Aligned {}
}
