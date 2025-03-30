use {
    crate::types::{self, ArrayType, MatrixType, ScalarType, ValueType, VectorType},
    std::num::NonZeroU32,
};

/// Uniform value.
pub trait Value {
    const TYPE: ValueType;
    type Type;
    fn value(&self) -> &[u8];
}

impl Value for u32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Uint);
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for f32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Float);
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec2 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec3 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec4 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat2 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat2);
    type Type = types::Mat2;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat3 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat3);
    type Type = types::Mat3;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat4 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat4);
    type Type = types::Mat4;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<V, const N: usize> Value for [V; N]
where
    V: Value + bytemuck::NoUninit,
{
    const TYPE: ValueType = ValueType::Array(ArrayType {
        base: &V::TYPE,
        size: NonZeroU32::new(N as u32).expect("array size cannot be zero"),
    });

    type Type = types::Array<V>;

    fn value(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}
