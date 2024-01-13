use {
    crate::{
        state::State,
        types::ValueType,
        uniform::{self, Value},
        Instance,
    },
    std::{iter, marker::PhantomData},
    wgpu::{Buffer, RenderPass},
};

#[derive(Clone, Copy)]
pub struct Row<'a> {
    data: &'a [u8],
    size: usize,
    ty: ValueType,
}

impl<'a> Row<'a> {
    pub fn new<U>(data: &'a [U]) -> Self
    where
        U: Value,
    {
        Self {
            data: uniform::values_as_bytes(data),
            size: data.len(),
            ty: U::TYPE,
        }
    }
}

pub struct Data<'a, I> {
    rows: &'a [Row<'a>],
    ty: PhantomData<I>,
}

impl<'a, I> Data<'a, I> {
    pub fn new(rows: &'a [Row<'a>]) -> Result<Self, Error>
    where
        I: Instance,
    {
        let Some(r) = rows.first() else {
            todo!("error");
        };

        for (row, ty) in iter::zip(rows, I::DEF) {
            if r.size != row.size {
                todo!("error");
            }

            if row.ty != ValueType::Vector(ty) {
                todo!("error");
            }
        }

        Ok(Self {
            rows,
            ty: PhantomData,
        })
    }
}

impl<I> Clone for Data<'_, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I> Copy for Data<'_, I> {}

/// An error returned from the [table data](crate::table::Data) constructors.
#[derive(Debug)]
pub enum Error {
    // todo
}

pub struct Table<I> {
    count: u32,
    bs: Box<[Buffer]>,
    ty: PhantomData<I>,
}

impl<I> Table<I> {
    pub(crate) fn new(state: &State, data: Data<I>) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let device = state.device();
        let buf = |row: &Row| {
            let desc = BufferInitDescriptor {
                label: None,
                contents: row.data,
                usage: BufferUsages::VERTEX,
            };

            device.create_buffer_init(&desc)
        };

        Self {
            count: data.rows[0].size as u32,
            bs: data.rows.iter().map(buf).collect(),
            ty: PhantomData,
        }
    }

    pub(crate) fn count(&self) -> u32 {
        self.count
    }

    pub(crate) fn set<'a>(&'a self, pass: &mut RenderPass<'a>, start_slot: u32) {
        for (slot, buf) in iter::zip(start_slot.., &self.bs[..]) {
            pass.set_vertex_buffer(slot, buf.slice(..));
        }
    }
}
