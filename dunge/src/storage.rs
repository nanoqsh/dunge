//! Storage type representing a typed array that can be read by a shader
//! Must be used with data that can be directly casted to the GPU buffer.

use {
    crate::{context::Context, state::State, types, value::Value},
    std::{error, fmt, marker::PhantomData},
    wgpu::Buffer,
};

pub trait StorageValue {
    fn storage_value(&self) -> &[u8];
}

impl<V> StorageValue for V
where
    V: Value,
{
    fn storage_value(&self) -> &[u8] {
        self.value()
    }
}

impl<V> StorageValue for [V]
where
    V: Value + bytemuck::Pod,
{
    fn storage_value(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

pub type RwStorage<V> = Storage<V, types::Mutable>;

/// Storage buffer data.
///
/// Can be created using the context's [`make_storage`](crate::Context::make_storage) function.
pub struct Storage<V, M = types::Immutable>
where
    V: ?Sized,
{
    buf: Buffer,
    size: usize,
    ty: PhantomData<V>,
    mu: PhantomData<M>,
}

impl<V, M> Storage<V, M>
where
    V: ?Sized,
{
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

        let size = contents.len();

        Self {
            buf,
            size,
            ty: PhantomData,
            mu: PhantomData,
        }
    }

    /// Updates the stored data.
    pub fn update(&self, cx: &Context, val: &V) -> Result<(), UpdateError>
    where
        V: StorageValue,
    {
        if size_of_val(val) != self.size {
            return Err(UpdateError);
        }

        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, val.storage_value());
        Ok(())
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

impl<V> Storage<V>
where
    V: ?Sized,
{
    pub fn rw(self) -> RwStorage<V> {
        RwStorage {
            buf: self.buf,
            size: self.size,
            ty: PhantomData,
            mu: PhantomData,
        }
    }
}

/// An error returned from the [update](crate::storage::Storage::update) function.
///
/// Returned when passed data size is invalid.
#[derive(Debug)]
pub struct UpdateError;

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "update error: the data size is invalid")
    }
}

impl error::Error for UpdateError {}
