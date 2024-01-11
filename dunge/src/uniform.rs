use {
    crate::{
        context::Context,
        state::State,
        types::{self, MemberType, ScalarType, VectorType},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::Buffer,
};

#[derive(Clone)]
pub struct Uniform<V> {
    buf: Arc<Buffer>,
    ty: PhantomData<V>,
}

impl<V> Uniform<V> {
    pub(crate) fn new(state: &State, data: &Data) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buf = {
            let desc = BufferInitDescriptor {
                label: None,
                contents: data.as_slice(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf: Arc::new(buf),
            ty: PhantomData,
        }
    }

    pub fn update(&self, cx: &Context, val: V)
    where
        V: Value,
    {
        let queue = cx.state().queue();
        let data = val.value();
        queue.write_buffer(&self.buf, 0, data.as_slice());
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

pub trait Value: private::Sealed {
    const TYPE: MemberType;
    type Type;
    fn value(self) -> Data;
}

impl private::Sealed for f32 {}

impl Value for f32 {
    const TYPE: MemberType = MemberType::Scalar(ScalarType::Float);
    type Type = Self;

    fn value(self) -> Data {
        Data([self, 0., 0., 0.])
    }
}

impl private::Sealed for [f32; 2] {}

impl Value for [f32; 2] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;

    fn value(self) -> Data {
        let [x, y] = self;
        Data([x, y, 0., 0.])
    }
}

impl private::Sealed for [f32; 3] {}

impl Value for [f32; 3] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;

    fn value(self) -> Data {
        let [x, y, z] = self;
        Data([x, y, z, 0.])
    }
}

impl private::Sealed for [f32; 4] {}

impl Value for [f32; 4] {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;

    fn value(self) -> Data {
        let [x, y, z, w] = self;
        Data([x, y, z, w])
    }
}

impl private::Sealed for glam::Vec2 {}

impl Value for glam::Vec2 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec2f);
    type Type = types::Vec2<f32>;

    fn value(self) -> Data {
        self.to_array().value()
    }
}

impl private::Sealed for glam::Vec3 {}

impl Value for glam::Vec3 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec3f);
    type Type = types::Vec3<f32>;

    fn value(self) -> Data {
        self.to_array().value()
    }
}

impl private::Sealed for glam::Vec4 {}

impl Value for glam::Vec4 {
    const TYPE: MemberType = MemberType::Vector(VectorType::Vec4f);
    type Type = types::Vec4<f32>;

    fn value(self) -> Data {
        self.to_array().value()
    }
}

pub struct Data([f32; 4]);

impl Data {
    fn as_slice(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }
}

mod private {
    pub trait Sealed {}
}
