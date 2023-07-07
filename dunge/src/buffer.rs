use {
    std::marker::PhantomData,
    wgpu::{Buffer, BufferSlice},
};

pub(crate) struct BufferView<'a, T> {
    buf: &'a Buffer,
    len: u32,
    ty: PhantomData<T>,
}

impl<'a, T> BufferView<'a, T> {
    pub fn new(buf: &'a Buffer) -> Self {
        use std::mem;

        Self {
            buf,
            len: (buf.size() / mem::size_of::<T>() as u64) as u32,
            ty: PhantomData,
        }
    }

    pub fn slice(&self) -> BufferSlice<'a> {
        self.buf.slice(..)
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
