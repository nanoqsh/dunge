use {
    crate::{
        sl::{ReadInstance, Ret},
        state::State,
        types::{self, VectorType},
        uniform::{self, Value},
        Instance,
    },
    std::marker::PhantomData,
    wgpu::{Buffer, RenderPass},
};

pub use dunge_shader::instance::Projection;

/// Describes a group member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: private::Sealed {
    const TYPE: VectorType;
    type Field;
    fn member_projection(id: u32) -> Self::Field;
}

impl private::Sealed for Row<[f32; 2]> {}

impl MemberProjection for Row<[f32; 2]> {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadInstance, types::Vec2<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

impl private::Sealed for Row<[f32; 3]> {}

impl MemberProjection for Row<[f32; 3]> {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadInstance, types::Vec3<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

impl private::Sealed for Row<[f32; 4]> {}

impl MemberProjection for Row<[f32; 4]> {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadInstance, types::Vec4<f32>>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

pub trait Set: Instance {
    fn set<'p>(&'p self, setter: &mut Setter<'_, 'p>);
}

pub struct Setter<'s, 'p> {
    len: Option<u32>,
    slot: u32,
    pass: &'s mut RenderPass<'p>,
}

impl<'s, 'p> Setter<'s, 'p> {
    pub(crate) fn new(slot: u32, pass: &'s mut RenderPass<'p>) -> Self {
        Self {
            len: None,
            slot,
            pass,
        }
    }

    pub(crate) fn len(&self) -> u32 {
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

pub trait SetMember<'p> {
    fn set_member(&'p self, setter: &mut Setter<'_, 'p>);
}

impl<'p, U> SetMember<'p> for Row<U> {
    fn set_member(&'p self, setter: &mut Setter<'_, 'p>) {
        setter.update_len(self.len);
        let slot = setter.next_slot();
        let slice = self.buf.slice(..);
        setter.pass.set_vertex_buffer(slot, slice);
    }
}

pub struct Row<U> {
    buf: Buffer,
    len: u32,
    ty: PhantomData<U>,
}

impl<U> Row<U> {
    pub(crate) fn new(state: &State, data: &[U]) -> Self
    where
        U: Value,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buf = {
            let desc = BufferInitDescriptor {
                label: None,
                contents: uniform::values_as_bytes(data),
                usage: BufferUsages::VERTEX,
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
}

mod private {
    pub trait Sealed {}
}
