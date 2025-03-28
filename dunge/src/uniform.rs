//! Uniform and value traits.

use {
    crate::{
        context::Context,
        state::State,
        value::{IntoValue, Value},
    },
    std::marker::PhantomData,
    wgpu::Buffer,
};

/// Uniform shader data.
///
/// Can be created using the context's [`make_uniform`](crate::Context::make_uniform) function.
pub struct Uniform<U> {
    buf: Buffer,
    ty: PhantomData<U>,
}

impl<U> Uniform<U> {
    pub(crate) fn new(state: &State, contents: &[u8]) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buf = {
            let desc = BufferInitDescriptor {
                label: None,
                contents,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf,
            ty: PhantomData,
        }
    }

    /// Updates the uniform data.
    pub fn update<V>(&self, cx: &Context, val: V)
    where
        V: IntoValue<Value = U>,
        U: Value,
    {
        let queue = cx.state().queue();
        let val = val.into_value();
        queue.write_buffer(&self.buf, 0, val.value().as_ref());
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}
