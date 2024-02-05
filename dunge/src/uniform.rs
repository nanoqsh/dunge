//! Uniform and value traits.

use {
    crate::{
        context::Context,
        state::State,
        types::{self, MatrixType, ScalarType, ValueType, VectorType},
    },
    std::{marker::PhantomData, mem, slice},
    wgpu::Buffer,
};

pub struct Uniform<U> {
    buf: Buffer,
    ty: PhantomData<U>,
}

impl<U> Uniform<U> {
    pub(crate) fn new(state: &State, contents: &[u8]) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buf = {
            let desc = BufferInitDescriptor {
                label: None,
                contents,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf,
            ty: PhantomData,
        }
    }

    pub fn update<V>(&self, cx: &Context, val: V)
    where
        V: IntoValue<Value = U>,
        U: Value,
    {
        let queue = cx.state().queue();
        let val = val.into_value();
        queue.write_buffer(&self.buf, 0, val.value().as_ref());
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

pub trait Value: private::Sealed {
    const TYPE: ValueType;
    type Type;
    type Data: AsRef<[u8]>;
    fn value(self) -> Self::Data;
}

pub struct Data<const N: usize = 4>([f32; N]);

impl<const N: usize> AsRef<[u8]> for Data<N> {
    fn as_ref(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }
}

pub(crate) fn values_as_bytes<U>(values: &[U]) -> &[u8]
where
    U: Value,
{
    // SAFETY:
    // * The `Value` invariant states converting a slice of values to bytes is safe
    unsafe { slice::from_raw_parts(values.as_ptr().cast(), mem::size_of_val(values)) }
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
    pub trait Sealed {}
}
