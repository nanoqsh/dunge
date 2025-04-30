//! Storage type representing a typed array that can be read by a shader.
//!
//! Must be used with data that can be directly casted to the GPU buffer.

use {
    crate::{context::Context, state::State, types, value::Value},
    std::marker::PhantomData,
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

struct Data {
    buf: wgpu::Buffer,
    size: usize,
}

impl Data {
    fn new(state: &State, contents: &[u8], usage: wgpu::BufferUsages) -> Self {
        use wgpu::util::{self, DeviceExt};

        let buf = {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents,
                usage,
            };

            state.device().create_buffer_init(&desc)
        };

        let size = contents.len();
        Self { buf, size }
    }

    fn update(&self, state: &State, contents: &[u8]) {
        assert_eq!(
            contents.len(),
            self.size,
            "cannot update buffer of size {} with value of size {}",
            self.size,
            contents.len(),
        );

        state.queue().write_buffer(&self.buf, 0, contents);
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
    data: Data,
    ty: PhantomData<V>,
    mu: PhantomData<M>,
}

impl<V, M> Storage<V, M>
where
    V: ?Sized,
{
    #[inline]
    pub(crate) fn new(cx: &Context, val: &V) -> Self
    where
        V: StorageValue,
    {
        let data = Data::new(
            cx.state(),
            val.storage_value(),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            data,
            ty: PhantomData,
            mu: PhantomData,
        }
    }

    /// Updates the stored data.
    ///
    /// # Panics
    /// Panics if the buffer size is not equal to the size of the new value.
    #[inline]
    pub fn update(&self, cx: &Context, val: &V)
    where
        V: StorageValue,
    {
        self.data.update(cx.state(), val.storage_value());
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.data.size
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.data.buf
    }
}

impl<V> Storage<V>
where
    V: ?Sized,
{
    pub fn rw(self) -> RwStorage<V> {
        RwStorage {
            data: self.data,
            ty: PhantomData,
            mu: PhantomData,
        }
    }
}

/// Uniform shader data.
///
/// Can be created using the context's [`make_uniform`](crate::Context::make_uniform) function.
pub struct Uniform<V>
where
    V: ?Sized,
{
    data: Data,
    ty: PhantomData<V>,
}

impl<V> Uniform<V>
where
    V: ?Sized,
{
    #[inline]
    pub(crate) fn new(cx: &Context, val: &V) -> Self
    where
        V: StorageValue,
    {
        let data = Data::new(
            cx.state(),
            val.storage_value(),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            data,
            ty: PhantomData,
        }
    }

    /// Updates the uniform data.
    ///
    /// # Panics
    /// Panics if the buffer size is not equal to the size of the new value.
    #[inline]
    pub fn update(&self, cx: &Context, val: &V)
    where
        V: StorageValue,
    {
        self.data.update(cx.state(), val.storage_value());
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.data.size
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.data.buf
    }
}
