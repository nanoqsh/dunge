use wgpu::{Buffer, BufferAddress, BufferSlice};

#[derive(Clone, Copy)]
pub(crate) struct BufferView<'a> {
    buf: &'a Buffer,
    len: u32,
}

impl<'a> BufferView<'a> {
    pub fn new<T>(buf: &'a Buffer, limit: Option<u32>) -> Self {
        use std::mem;

        let size = mem::size_of::<T>() as BufferAddress;
        assert!(size > 0, "buffer element size cannot be zero");

        let len = (buf.size() / size) as u32;
        Self {
            buf,
            len: limit.map_or(len, |n| len.min(n)),
        }
    }

    pub fn slice(&self) -> BufferSlice<'a> {
        self.buf.slice(..)
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
