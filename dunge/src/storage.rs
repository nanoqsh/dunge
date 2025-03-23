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
pub struct Storage<U> {
    buf: Buffer,
    ty: PhantomData<U>,
    len: usize,
}

impl<U> Storage<U> {
    pub(crate) fn new(state: &State, contents: &[U]) -> Self
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
                contents: bytemuck::cast_slice(contents),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf,
            ty: PhantomData,
            len: contents.len(),
        }
    }

    /// Updates the stored data.
    pub fn update(&self, cx: &Context, contents: &[U])
    where
        U: Value,
    {
        assert_eq!(
            contents.len(),
            self.len(),
            "attempted to update storage buffer with an array of the wrong size",
        );

        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(contents));
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<U> Storage<U> {
    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}
