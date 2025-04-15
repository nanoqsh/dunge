pub enum MapRead<
    // Read
    const T: bool, // CopyTo
> {}

pub enum MapWrite<
    // Write
    const F: bool, // CopyFrom
> {}

pub enum Copy<
    const F: bool, // CopyFrom
    const T: bool, // CopyTo
> {}

pub type EmptyBuffer = Copy<false, false>;

#[expect(dead_code)]
pub(crate) trait BufferUsages {
    fn usages() -> wgpu::BufferUsages;
}

impl<const T: bool> BufferUsages for MapRead<T> {
    fn usages() -> wgpu::BufferUsages {
        let mut u = wgpu::BufferUsages::MAP_READ;
        u.set(wgpu::BufferUsages::COPY_DST, T);
        u
    }
}

impl<const F: bool> BufferUsages for MapWrite<F> {
    fn usages() -> wgpu::BufferUsages {
        let mut u = wgpu::BufferUsages::MAP_WRITE;
        u.set(wgpu::BufferUsages::COPY_SRC, F);
        u
    }
}

impl<const F: bool, const T: bool> BufferUsages for Copy<F, T> {
    fn usages() -> wgpu::BufferUsages {
        let mut u = wgpu::BufferUsages::empty();
        u.set(wgpu::BufferUsages::COPY_SRC, F);
        u.set(wgpu::BufferUsages::COPY_DST, T);
        u
    }
}

pub enum Texture<
    const F: bool, // CopyFrom
    const T: bool, // CopyTo
    const B: bool, // Bind
    const R: bool, // Render
> {}

pub type EmptyTexture = Texture<false, false, false, false>;

#[expect(dead_code)]
pub(crate) trait TextureUsages {
    fn usages() -> wgpu::TextureUsages;
}

impl<const F: bool, const T: bool, const B: bool, const R: bool> TextureUsages
    for Texture<F, T, B, R>
{
    fn usages() -> wgpu::TextureUsages {
        let mut u = wgpu::TextureUsages::empty();
        u.set(wgpu::TextureUsages::COPY_SRC, F);
        u.set(wgpu::TextureUsages::COPY_DST, T);
        u.set(wgpu::TextureUsages::TEXTURE_BINDING, B);
        u.set(wgpu::TextureUsages::RENDER_ATTACHMENT, R);
        u
    }
}

pub trait Use<T>
where
    T: ?Sized,
{
    type Out;
}

pub(crate) trait Read {}

impl<const T: bool> Read for MapRead<T> {}
impl<const T: bool> Use<dyn Read> for Copy<false, T> {
    type Out = MapRead<T>;
}

pub(crate) trait Write {}

impl<const F: bool> Write for MapWrite<F> {}
impl<const F: bool> Use<dyn Write> for Copy<F, false> {
    type Out = MapWrite<F>;
}

pub(crate) trait CopyFrom {}

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

pub(crate) trait CopyTo {}

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

pub(crate) trait Bind {}

impl<const F: bool, const T: bool, const R: bool> Bind for Texture<F, T, true, R> {}
impl<const F: bool, const T: bool, const R: bool> Use<dyn Bind> for Texture<F, T, false, R> {
    type Out = Texture<F, T, true, R>;
}

pub(crate) trait Render {}

impl<const F: bool, const T: bool, const B: bool> Render for Texture<F, T, B, true> {}
impl<const F: bool, const T: bool, const B: bool> Use<dyn Render> for Texture<F, T, B, false> {
    type Out = Texture<F, T, B, true>;
}
