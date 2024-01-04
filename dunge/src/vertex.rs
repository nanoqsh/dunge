use crate::{
    sl::{ReadInput, Ret},
    types::{self, VectorType},
};

pub use dunge_shader::vertex::*;

pub trait InputProjection {
    const TYPE: VectorType;
    type Field;
    fn input_projection(id: u32, index: u32) -> Self::Field;
}

impl InputProjection for [f32; 2] {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadInput, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl InputProjection for [f32; 3] {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadInput, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl InputProjection for [f32; 4] {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadInput, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl InputProjection for glam::Vec2 {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadInput, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl InputProjection for glam::Vec3 {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadInput, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}

impl InputProjection for glam::Vec4 {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadInput, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadInput::new(id, index)
    }
}
