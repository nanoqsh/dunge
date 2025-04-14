//! Uniform and value traits.

use {
    crate::{context::Context, state::State, value::Value},
    std::marker::PhantomData,
};

/// Uniform shader data.
///
/// Can be created using the context's [`make_uniform`](crate::Context::make_uniform) function.
pub struct Uniform<V> {
    buf: wgpu::Buffer,
    ty: PhantomData<V>,
}

impl<V> Uniform<V> {
    pub(crate) fn new(state: &State, contents: &[u8]) -> Self {
        use wgpu::util::{self, DeviceExt};

        let buf = {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            };

            state.device().create_buffer_init(&desc)
        };

        Self {
            buf,
            ty: PhantomData,
        }
    }

    /// Updates the uniform data.
    pub fn update(&self, cx: &Context, val: &V)
    where
        V: Value,
    {
        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, val.value());
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }
}
