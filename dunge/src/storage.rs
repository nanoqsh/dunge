//! Storage type representing a typed array that can be read by a shader
//! Currently only supports read-only arrays. Must be used with
//! data that can be directly casted to the GPU buffer.

use {
    crate::{context::Context, state::State, value::Value},
    std::marker::PhantomData,
    wgpu::Buffer,
};

/// Storage buffer data.
///
/// Can be created using the context's [`make_storage`](crate::Context::make_storage) function.
pub struct Storage<V> {
    buf: Buffer,
    ty: PhantomData<V>,
}

impl<V> Storage<V> {
    pub(crate) fn new(state: &State, contents: &[u8]) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buf = {
            let desc = BufferInitDescriptor {
                label: None,
                contents,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf,
            ty: PhantomData,
        }
    }

    /// Updates the stored data.
    pub fn update(&self, cx: &Context, val: &V)
    where
        V: Value,
    {
        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, val.value());
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}
