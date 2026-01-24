use {
    crate::Context,
    dunge_shade::{
        bytes::Bytes,
        irc::Value,
        store::{self, Data, StorageValue},
    },
    std::num::NonZeroU32,
};

pub struct Dunge {
    buf: wgpu::Buffer,
    len: NonZeroU32,
}

impl Dunge {
    fn new(cx: &Context, len: NonZeroU32, bytes: &[u8], usage: wgpu::BufferUsages) -> Self {
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

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }
}

impl Data for Dunge {
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

    fn len(&self) -> NonZeroU32 {
        self.len
    }
}

pub type Uniform<V> = store::Uniform<V, Dunge>;

pub(crate) fn uniform<V>(value: &V, cx: &Context) -> Uniform<V>
where
    V: Value + Bytes,
{
    store::internal::uniform(value, |bytes| {
        Dunge::new(
            cx,
            NonZeroU32::MIN, // unused
            bytes,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    })
}

pub type Storage<V> = store::Storage<V, Dunge>;

pub(crate) fn storage<V>(value: &V, cx: &Context) -> Storage<V>
where
    V: StorageValue + ?Sized,
{
    store::internal::storage(value, |bytes| {
        Dunge::new(
            cx,
            NonZeroU32::MIN, // unused
            bytes,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        )
    })
}
