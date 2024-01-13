use {
    crate::{
        context::Context,
        state::State,
        types::{self, MatrixType, MemberType, ScalarType, VectorType},
    },
    std::marker::PhantomData,
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

    pub fn update(&self, cx: &Context, val: U)
    where
        U: Value,
    {
        let queue = cx.state().queue();
        let data = val.value();
        queue.write_buffer(&self.buf, 0, data.as_ref());
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

pub trait Value: private::Sealed {
    const TYPE: MemberType;
    type Type;
    type Data: AsRef<[u8]>;
    fn value(self) -> Self::Data;
}

impl private::Sealed for f32 {}

impl Value for f32 {
    const TYPE: MemberType = MemberType::Scalar(ScalarType::Float);
    type Type = Self;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data([self, 0., 0., 0.])
    }
}

impl private::Sealed for [f32; 2] {}

impl Value for [f32; 2] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y] = self;
        Data([x, y, 0., 0.])
    }
}

impl private::Sealed for [f32; 3] {}

impl Value for [f32; 3] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        let [x, y, z] = self;
        Data([x, y, z, 0.])
    }
}

impl private::Sealed for [f32; 4] {}

impl Value for [f32; 4] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        Data(self)
    }
}

impl private::Sealed for glam::Vec2 {}

impl Value for glam::Vec2 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        self.to_array().value()
    }
}

impl private::Sealed for glam::Vec3 {}

impl Value for glam::Vec3 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        self.to_array().value()
    }
}

impl private::Sealed for glam::Vec4 {}

impl Value for glam::Vec4 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;
    type Data = Data;

    fn value(self) -> Self::Data {
        self.to_array().value()
    }
}

impl private::Sealed for glam::Mat2 {}

impl Value for glam::Mat2 {
    const TYPE: MemberType = MemberType::Matrix(MatrixType::Mat2);
    type Type = types::Mat2;
    type Data = Data;

    fn value(self) -> Self::Data {
        self.to_cols_array().value()
    }
}

impl private::Sealed for glam::Mat3 {}

impl Value for glam::Mat3 {
    const TYPE: MemberType = MemberType::Matrix(MatrixType::Mat3);
    type Type = types::Mat3;
    type Data = Data<9>;

    fn value(self) -> Self::Data {
        Data(self.to_cols_array())
    }
}

impl private::Sealed for glam::Mat4 {}

impl Value for glam::Mat4 {
    const TYPE: MemberType = MemberType::Matrix(MatrixType::Mat4);
    type Type = types::Mat4;
    type Data = Data<16>;

    fn value(self) -> Self::Data {
        Data(self.to_cols_array())
    }
}

pub struct Data<const N: usize = 4>([f32; N]);

impl<const N: usize> AsRef<[u8]> for Data<N> {
    fn as_ref(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }
}

mod private {
    pub trait Sealed {}
}
