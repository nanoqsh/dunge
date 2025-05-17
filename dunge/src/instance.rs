//! Shader instance types and traits.

use {
    crate::{
        Instance,
        context::Context,
        render::VertexSetter,
        sl::{ReadInstance, Ret},
        state::State,
        types::{self, ValueType, VectorType},
        value::UnsizedValue,
    },
    std::marker::PhantomData,
};

pub use dunge_shader::instance::Projection;

/// Describes an instance member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: s::Sealed {
    const TYPE: ValueType;
    type Field;
    fn member_projection(id: u32) -> Self::Field;
}

impl s::Sealed for Row<glam::Vec2> {}

impl MemberProjection for Row<glam::Vec2> {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec2f);
    type Field = Ret<ReadInstance, types::Vec2<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

impl s::Sealed for Row<glam::Vec3> {}

impl MemberProjection for Row<glam::Vec3> {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec3f);
    type Field = Ret<ReadInstance, types::Vec3<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

impl s::Sealed for Row<glam::Vec4> {}

impl MemberProjection for Row<glam::Vec4> {
    const TYPE: ValueType = ValueType::Vector(VectorType::Vec4f);
    type Field = Ret<ReadInstance, types::Vec4<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

pub trait Set: Instance {
    fn set(&self, setter: &mut Setter<'_, '_>);
}

pub(crate) fn set<I>(vs: VertexSetter<'_, '_>, slot: u32, instance: &I) -> u32
where
    I: Set,
{
    let len = None;
    let mut setter = Setter { len, slot, vs };
    instance.set(&mut setter);
    setter.len()
}

pub struct Setter<'ren, 'layer> {
    len: Option<u32>,
    slot: u32,
    vs: VertexSetter<'ren, 'layer>,
}

impl Setter<'_, '_> {
    fn len(&self) -> u32 {
        self.len.unwrap_or_default()
    }

    fn next_slot(&mut self) -> u32 {
        let next = self.slot;
        self.slot += 1;
        next
    }

    fn update_len(&mut self, len: u32) {
        let current = self.len.get_or_insert(len);
        *current = u32::min(*current, len);
    }
}

pub trait SetMember {
    fn set_member(&self, setter: &mut Setter<'_, '_>);
}

impl<U> SetMember for Row<U> {
    fn set_member(&self, setter: &mut Setter<'_, '_>) {
        setter.update_len(self.len);
        let slot = setter.next_slot();
        setter.vs.set(&self.buf, slot);
    }
}

pub struct Row<U> {
    buf: wgpu::Buffer,
    len: u32,
    ty: PhantomData<U>,
}

impl<U> Row<U> {
    pub(crate) fn new(state: &State, data: &[U]) -> Self
    where
        [U]: UnsizedValue,
    {
        use wgpu::util::{self, DeviceExt};

        let buf = {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents: data.unsized_value(),
                usage: wgpu::BufferUsages::VERTEX,
            };

            state.device().create_buffer_init(&desc)
        };

        let len = data.len() as u32;

        Self {
            buf,
            len,
            ty: PhantomData,
        }
    }

    /// Updates the row data.
    ///
    /// # Panics
    /// Panics if the row length is not equal to the length of the new value.
    pub fn update(&self, cx: &Context, data: &[U])
    where
        [U]: UnsizedValue,
    {
        assert_eq!(
            data.len(),
            self.len(),
            "cannot update row of length {} with value of length {}",
            self.len(),
            data.len(),
        );

        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, data.unsized_value());
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

mod s {
    pub trait Sealed {}
}
