use crate::{
    sl::{ReadInput, Ret},
    types::{self, VectorType},
};

pub use dunge_shader::vertex::{verts_as_bytes, Projection};

/// Describes an input type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait InputProjection: private::Sealed {
    const TYPE: VectorType;
    type Field;
    fn input_projection(id: u32, index: u32) -> Self::Field;
}

impl private::Sealed for [f32; 2] {}

impl InputProjection for [f32; 2] {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadInput, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl private::Sealed for [f32; 3] {}

impl InputProjection for [f32; 3] {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadInput, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl private::Sealed for [f32; 4] {}

impl InputProjection for [f32; 4] {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadInput, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl private::Sealed for glam::Vec2 {}

impl InputProjection for glam::Vec2 {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadInput, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl private::Sealed for glam::Vec3 {}

impl InputProjection for glam::Vec3 {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadInput, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl private::Sealed for glam::Vec4 {}

impl InputProjection for glam::Vec4 {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadInput, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

mod private {
    pub trait Sealed {}
}
