#[derive(Default)]
pub struct MapRead<
    // Read
    const T: bool, // CopyTo
> {}

#[derive(Default)]
pub struct MapWrite<
    // Write
    const F: bool, // CopyFrom
> {}

#[derive(Default)]
pub struct Copy<
    const F: bool, // CopyFrom
    const T: bool, // CopyTo
> {}

pub type BufferNoUsages = Copy<false, false>;

pub struct DynamicBufferUsages(pub(crate) wgpu::BufferUsages);

#[derive(Default)]
pub struct Texture<
    const F: bool, // CopyFrom
    const T: bool, // CopyTo
    const B: bool, // Bind
    const R: bool, // Render
> {}

pub type TextureNoUsages = Texture<false, false, false, false>;

pub struct DynamicTextureUsages(pub(crate) wgpu::TextureUsages);

pub trait Use<T>
where
    T: ?Sized,
{
    type Out: Default;
}

pub(crate) mod u {
    use super::*;

    pub trait BufferUsages {
        fn usages(&self) -> wgpu::BufferUsages;
    }

    impl<const T: bool> BufferUsages for MapRead<T> {
        #[inline]
        fn usages(&self) -> wgpu::BufferUsages {
            let mut u = wgpu::BufferUsages::MAP_READ;
            u.set(wgpu::BufferUsages::COPY_DST, T);
            u
        }
    }

    impl<const F: bool> BufferUsages for MapWrite<F> {
        #[inline]
        fn usages(&self) -> wgpu::BufferUsages {
            let mut u = wgpu::BufferUsages::MAP_WRITE;
            u.set(wgpu::BufferUsages::COPY_SRC, F);
            u
        }
    }

    impl<const F: bool, const T: bool> BufferUsages for Copy<F, T> {
        #[inline]
        fn usages(&self) -> wgpu::BufferUsages {
            let mut u = wgpu::BufferUsages::empty();
            u.set(wgpu::BufferUsages::COPY_SRC, F);
            u.set(wgpu::BufferUsages::COPY_DST, T);
            u
        }
    }

    impl BufferUsages for DynamicBufferUsages {
        #[inline]
        fn usages(&self) -> wgpu::BufferUsages {
            self.0
        }
    }

    pub trait TextureUsages {
        fn usages(&self) -> wgpu::TextureUsages;
    }

    impl<const F: bool, const T: bool, const B: bool, const R: bool> TextureUsages
        for Texture<F, T, B, R>
    {
        #[inline]
        fn usages(&self) -> wgpu::TextureUsages {
            let mut u = wgpu::TextureUsages::empty();
            u.set(wgpu::TextureUsages::COPY_SRC, F);
            u.set(wgpu::TextureUsages::COPY_DST, T);
            u.set(wgpu::TextureUsages::TEXTURE_BINDING, B);
            u.set(wgpu::TextureUsages::RENDER_ATTACHMENT, R);
            u
        }
    }

    impl TextureUsages for DynamicTextureUsages {
        #[inline]
        fn usages(&self) -> wgpu::TextureUsages {
            self.0
        }
    }

    pub trait Read {
        #[doc(hidden)]
        #[inline]
        fn read(&self) {}
    }

    impl<const T: bool> Read for MapRead<T> {}
    impl<const T: bool> Use<dyn Read> for Copy<false, T> {
        type Out = MapRead<T>;
    }

    impl Read for DynamicBufferUsages {
        #[inline]
        fn read(&self) {
            assert!(
                self.0.contains(wgpu::BufferUsages::MAP_READ),
                "the buffer usages has no read unsage",
            )
        }
    }

    pub trait Write {
        #[doc(hidden)]
        #[inline]
        fn write(&self) {}
    }

    impl<const F: bool> Write for MapWrite<F> {}
    impl<const F: bool> Use<dyn Write> for Copy<F, false> {
        type Out = MapWrite<F>;
    }

    impl Write for DynamicBufferUsages {
        #[inline]
        fn write(&self) {
            assert!(
                self.0.contains(wgpu::BufferUsages::MAP_WRITE),
                "the buffer usages has no write unsage",
            )
        }
    }

    pub trait CopyFrom {
        #[doc(hidden)]
        #[inline]
        fn copy_from(&self) {}
    }

    impl CopyFrom for MapWrite<true> {}
    impl Use<dyn CopyFrom> for MapWrite<false> {
        type Out = MapWrite<true>;
    }

    impl<const T: bool> CopyFrom for Copy<true, T> {}
    impl<const T: bool> Use<dyn CopyFrom> for Copy<false, T> {
        type Out = Copy<true, T>;
    }

    impl<const T: bool, const B: bool, const R: bool> CopyFrom for Texture<true, T, B, R> {}
    impl<const T: bool, const B: bool, const R: bool> Use<dyn CopyFrom> for Texture<false, T, B, R> {
        type Out = Texture<true, T, B, R>;
    }

    impl CopyFrom for DynamicBufferUsages {
        #[inline]
        fn copy_from(&self) {
            assert!(
                self.0.contains(wgpu::BufferUsages::COPY_SRC),
                "the buffer usages has no copy from unsage",
            )
        }
    }

    impl CopyFrom for DynamicTextureUsages {
        #[inline]
        fn copy_from(&self) {
            assert!(
                self.0.contains(wgpu::TextureUsages::COPY_SRC),
                "the texture usages has no copy from unsage",
            )
        }
    }

    pub trait CopyTo {
        #[doc(hidden)]
        #[inline]
        fn copy_to(&self) {}
    }

    impl CopyTo for MapRead<true> {}
    impl Use<dyn CopyTo> for MapRead<false> {
        type Out = MapRead<true>;
    }

    impl<const S: bool> CopyTo for Copy<S, true> {}
    impl<const F: bool> Use<dyn CopyTo> for Copy<F, false> {
        type Out = Copy<F, true>;
    }

    impl<const F: bool, const B: bool, const R: bool> CopyTo for Texture<F, true, B, R> {}
    impl<const F: bool, const B: bool, const R: bool> Use<dyn CopyTo> for Texture<F, false, B, R> {
        type Out = Texture<F, true, B, R>;
    }

    impl CopyTo for DynamicBufferUsages {
        #[inline]
        fn copy_to(&self) {
            assert!(
                self.0.contains(wgpu::BufferUsages::COPY_DST),
                "the buffer usages has no copy to unsage",
            )
        }
    }

    impl CopyTo for DynamicTextureUsages {
        #[inline]
        fn copy_to(&self) {
            assert!(
                self.0.contains(wgpu::TextureUsages::COPY_DST),
                "the texture usages has no copy to unsage",
            )
        }
    }

    pub trait Bind {
        #[doc(hidden)]
        #[inline]
        fn bind(&self) {}
    }

    impl<const F: bool, const T: bool, const R: bool> Bind for Texture<F, T, true, R> {}
    impl<const F: bool, const T: bool, const R: bool> Use<dyn Bind> for Texture<F, T, false, R> {
        type Out = Texture<F, T, true, R>;
    }

    impl Bind for DynamicTextureUsages {
        #[inline]
        fn bind(&self) {
            assert!(
                self.0.contains(wgpu::TextureUsages::TEXTURE_BINDING),
                "the texture usages has no bind unsage",
            )
        }
    }

    pub trait Render {
        #[doc(hidden)]
        #[inline]
        fn render(&self) {}
    }

    impl<const F: bool, const T: bool, const B: bool> Render for Texture<F, T, B, true> {}
    impl<const F: bool, const T: bool, const B: bool> Use<dyn Render> for Texture<F, T, B, false> {
        type Out = Texture<F, T, B, true>;
    }

    impl Render for DynamicTextureUsages {
        #[inline]
        fn render(&self) {
            assert!(
                self.0.contains(wgpu::TextureUsages::RENDER_ATTACHMENT),
                "the texture usages has no render unsage",
            )
        }
    }
}
