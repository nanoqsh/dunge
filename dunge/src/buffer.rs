use wgpu::{Buffer, BufferSlice};

pub(crate) struct BufferView<'a> {
    buf: &'a Buffer,
    len: u32,
}

impl<'a> BufferView<'a> {
    pub fn new<T>(buf: &'a Buffer) -> Self {
        use std::mem;

        Self {
            buf,
            len: (buf.size() / mem::size_of::<T>() as u64) as u32,
        }
    }

    pub fn slice(&self) -> BufferSlice<'a> {
        self.buf.slice(..)
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
