use dunge_shader::types::Atomic;

use crate::types::{self, MatrixType, ScalarType, ValueType, VectorType};

/// Uniform value.
pub trait Value: private::Sealed {
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

impl private::Sealed for Atomic<u32> {}

impl Value for Atomic<u32> {
    const TYPE: ValueType = ValueType::Atomic(ScalarType::Uint);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self.0 as f32, 0., 0., 0.])
    }
}

impl private::Sealed for u32 {}

impl Value for u32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Uint);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self as f32, 0., 0., 0.])
    }
}

impl private::Sealed for f32 {}

impl Value for f32 {
    const TYPE: ValueType = ValueType::Scalar(ScalarType::Float);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self, 0., 0., 0.])
    }
}

impl private::Sealed for [f32; 2] {}

impl Value for [f32; 2] {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y] = self;
        Data([x, y, 0., 0.])
    }
}

impl private::Sealed for [f32; 3] {}

impl Value for [f32; 3] {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y, z] = self;
        Data([x, y, z, 0.])
    }
}

impl private::Sealed for [f32; 4] {}

impl Value for [f32; 4] {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data(self)
    }
}

impl private::Sealed for [[f32; 2]; 2] {}

impl Value for [[f32; 2]; 2] {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat2);
    type Type = types::Mat2;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data(bytemuck::cast(self))
    }
}

impl private::Sealed for [[f32; 3]; 3] {}

impl Value for [[f32; 3]; 3] {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat3);
    type Type = types::Mat3;
    type Data = Data<9>;

    fn value(self) -> Self::Data {
        Data(bytemuck::cast(self))
    }
}

impl private::Sealed for [[f32; 4]; 4] {}

impl Value for [[f32; 4]; 4] {
    const TYPE: ValueType = ValueType::Matrix(MatrixType::Mat4);
    type Type = types::Mat4;
    type Data = Data<16>;

    fn value(self) -> Self::Data {
        Data(bytemuck::cast(self))
    }
}

/// Types that can be converted to a uniform value.
pub trait IntoValue {
    type Value: Value;
    fn into_value(self) -> Self::Value;
}

impl<U> IntoValue for U
where
    U: Value,
{
    type Value = Self;

    fn into_value(self) -> Self {
        self
    }
}

impl IntoValue for glam::Vec2 {
    type Value = [f32; 2];

    fn into_value(self) -> Self::Value {
        self.to_array()
    }
}

impl IntoValue for glam::Vec3 {
    type Value = [f32; 3];

    fn into_value(self) -> Self::Value {
        self.to_array()
    }
}

impl IntoValue for glam::Vec4 {
    type Value = [f32; 4];

    fn into_value(self) -> Self::Value {
        self.to_array()
    }
}

impl IntoValue for glam::Mat2 {
    type Value = [[f32; 2]; 2];

    fn into_value(self) -> Self::Value {
        self.to_cols_array_2d()
    }
}

impl IntoValue for glam::Mat3 {
    type Value = [[f32; 3]; 3];

    fn into_value(self) -> Self::Value {
        self.to_cols_array_2d()
    }
}

impl IntoValue for glam::Mat4 {
    type Value = [[f32; 4]; 4];

    fn into_value(self) -> Self::Value {
        self.to_cols_array_2d()
    }
}

mod private {
    pub trait Sealed: bytemuck::NoUninit {}
}
