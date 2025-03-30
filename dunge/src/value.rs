use crate::types::{self, MatrixType, ScalarType, ValueType, VectorType};

/// Uniform value.
pub trait Value {
    const TYPE: ValueType;
    type Type;
    type Data: AsRef<[u8]>;
    fn value(self) -> Self::Data;
}

/// Uniform binary data.
pub struct Data<const N: usize = 4>([f32; N]);

impl<const N: usize> AsRef<[u8]> for Data<N> {
    fn as_ref(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }
}

impl Value for u32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Uint);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self as f32, 0., 0., 0.])
    }
}

impl Value for f32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Float);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self, 0., 0., 0.])
    }
}

impl Value for glam::Vec2 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y] = self.to_array();
        Data([x, y, 0., 0.])
    }
}

impl Value for glam::Vec3 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y, z] = self.to_array();
        Data([x, y, z, 0.])
    }
}

impl Value for glam::Vec4 {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data(self.to_array())
    }
}

impl Value for glam::Mat2 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat2);
    type Type = types::Mat2;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data(self.to_cols_array())
    }
}

impl Value for glam::Mat3 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat3);
    type Type = types::Mat3;
    type Data = Data<9>;

    fn value(self) -> Self::Data {
        Data(self.to_cols_array())
    }
}

impl Value for glam::Mat4 {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat4);
    type Type = types::Mat4;
    type Data = Data<16>;

    fn value(self) -> Self::Data {
        Data(self.to_cols_array())
    }
}
