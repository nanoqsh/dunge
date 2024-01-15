use {
    crate::{
        state::State,
        uniform::{self, Value},
        Instance,
    },
    std::marker::PhantomData,
    wgpu::{Buffer, RenderPass},
};

pub use dunge_shader::instance::Projection;

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
