use {
    crate::Context,
    dunge_shade::{
        bytes::Bytes,
        irc::Value,
        store::{self, Data, Store},
    },
    std::{num::NonZeroU32, ops},
};

pub use dunge_shade::store::StorageValue;

#[derive(Clone)]
pub struct Dunge {
    buf: wgpu::Buffer,
    len: NonZeroU32,
}

impl Dunge {
    fn new(cx: &Context, bytes: &[u8], len: NonZeroU32, usage: wgpu::BufferUsages) -> Self {
        use wgpu::util::{self, DeviceExt};

        let buf = {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage,
            };

            cx.state().device().create_buffer_init(&desc)
        };

        Self { buf, len }
    }

    fn store(cx: &Context, bytes: &[u8], usage: wgpu::BufferUsages) -> Self {
        Self::new(cx, bytes, NonZeroU32::MIN, usage)
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }
}

impl Store for Dunge {
    type Context = Context;

    fn update(&self, cx: &Self::Context, bytes: &[u8]) {
        assert_eq!(
            bytes.len() as u64,
            self.buf.size(),
            "bytes length must be the same as the buffer size",
        );

        cx.state().queue().write_buffer(&self.buf, 0, bytes);
    }

    fn byte_size(&self) -> u64 {
        self.buf.size()
    }

    fn len_non_zero(&self) -> NonZeroU32 {
        self.len
    }
}

impl Data for Dunge {
    type Slice<'slice> = DungeSlice<'slice>;

    fn slice(&self, bounds: ops::Range<u64>, len: NonZeroU32) -> Self::Slice<'_> {
        let buf = self.buf.slice(bounds);
        DungeSlice { buf, len }
    }

    fn byte_offset(slice: &Self::Slice<'_>) -> u64 {
        slice.buf.offset()
    }
}

#[derive(Clone, Copy)]
pub struct DungeSlice<'slice> {
    buf: wgpu::BufferSlice<'slice>,
    len: NonZeroU32,
}

impl<'slice> DungeSlice<'slice> {
    pub(crate) fn slice(self) -> wgpu::BufferSlice<'slice> {
        self.buf
    }
}

impl Store for DungeSlice<'_> {
    type Context = Context;

    fn update(&self, cx: &Self::Context, bytes: &[u8]) {
        assert_eq!(
            bytes.len() as u64,
            self.buf.size().get(),
            "bytes length must be the same as the buffer size",
        );

        cx.state()
            .queue()
            .write_buffer(self.buf.buffer(), self.buf.offset(), bytes);
    }

    fn byte_size(&self) -> u64 {
        self.buf.size().get()
    }

    fn len_non_zero(&self) -> NonZeroU32 {
        self.len
    }
}

pub type Uniform<V> = store::Uniform<V, Dunge>;

pub(crate) fn uniform<V>(value: &V, cx: &Context) -> Uniform<V>
where
    V: Value + Bytes,
{
    store::internal::uniform(value, |bytes| {
        Dunge::store(
            cx,
            bytes,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    })
    .expect("non zero sized value")
}

pub type Storage<V> = store::Storage<V, Dunge>;

pub(crate) fn storage<V>(value: &V, cx: &Context) -> Option<Storage<V>>
where
    V: StorageValue + ?Sized,
{
    store::internal::storage(value, |bytes| {
        Dunge::store(
            cx,
            bytes,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        )
    })
}

pub type Row<V> = store::Row<V, Dunge>;
pub type RowSlice<'slice, V> = store::RowSlice<'slice, V, Dunge>;

pub(crate) fn row<V>(value: &[V], cx: &Context) -> Option<Row<V>>
where
    V: Value + Bytes,
{
    let len = value.len().try_into().ok().and_then(NonZeroU32::new)?;
    store::internal::row(value, |bytes| {
        Dunge::new(
            cx,
            bytes,
            len,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        )
    })
}
