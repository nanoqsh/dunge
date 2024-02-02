use {
    crate::{
        format::Format,
        state::{RenderView, State},
    },
    std::{error, fmt, future::IntoFuture, mem},
    wgpu::{
        Buffer, BufferAsyncError, BufferSlice, BufferView, CommandEncoder, FilterMode,
        TextureUsages, TextureView, WasmNotSend,
    },
};

#[derive(Clone, Copy)]
pub struct TextureData<'a> {
    data: &'a [u8],
    size: (u32, u32),
    format: Format,
}

impl<'a> TextureData<'a> {
    pub const fn empty(size: (u32, u32), format: Format) -> Result<Self, ZeroSized> {
        let (width, height) = size;
        if width == 0 || height == 0 {
            return Err(ZeroSized);
        }

        Ok(Self {
            data: &[],
            size,
            format,
        })
    }

    pub const fn new(data: &'a [u8], size: (u32, u32), format: Format) -> Result<Self, Error> {
        let Ok(empty) = Self::empty(size, format) else {
            return Err(Error::ZeroSized);
        };

        let len = {
            let (width, height) = size;
            width as usize * height as usize * format.bytes() as usize
        };

        if data.len() != len {
            return Err(Error::InvalidLen);
        }

        Ok(Self { data, ..empty })
    }

    /// Allow to use a texture in the shader.
    pub fn with_bind(self) -> Bind<Self> {
        Bind(self)
    }

    /// Allow to use a texture as render attachment.
    pub fn with_draw(self) -> Draw<Self> {
        Draw(self)
    }

    /// Allow to copy data from the texture.
    pub fn with_copy(self) -> Copy<Self> {
        Copy(self)
    }
}

/// The [texture data](crate::texture::TextureData) error.
#[derive(Debug)]
pub enum Error {
    /// The texture data is zero sized.
    ZeroSized,

    /// The texture data length doesn't match with size and format.
    InvalidLen,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ZeroSized => write!(f, "zero sized data"),
            Self::InvalidLen => write!(f, "invalid data length"),
        }
    }
}

impl error::Error for Error {}

/// The [texture data](crate::texture::TextureData) is zero sized.
#[derive(Debug)]
pub struct ZeroSized;

impl fmt::Display for ZeroSized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "zero sized data")
    }
}

impl error::Error for ZeroSized {}

pub struct Texture {
    inner: wgpu::Texture,
    view: TextureView,
}

impl Texture {
    fn new(state: &State, mut usage: TextureUsages, data: TextureData) -> Self {
        use wgpu::*;

        let (width, height) = data.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let copy_data = !data.data.is_empty();
        let inner = {
            usage.set(TextureUsages::COPY_DST, copy_data);
            let desc = TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: data.format.wgpu(),
                usage,
                view_formats: &[],
            };

            state.device().create_texture(&desc)
        };

        if copy_data {
            state.queue().write_texture(
                ImageCopyTexture {
                    texture: &inner,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                data.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * data.format.bytes()),
                    rows_per_image: Some(height),
                },
                size,
            );
        }

        let view = {
            let desc = TextureViewDescriptor::default();
            inner.create_view(&desc)
        };

        Self { inner, view }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    pub fn format(&self) -> Format {
        Format::from_wgpu(self.inner.format()).expect("supported format")
    }

    pub(crate) fn view(&self) -> &TextureView {
        &self.view
    }

    pub(crate) fn render_view(&self) -> RenderView {
        RenderView::new(&self.view, self.format())
    }
}

pub(crate) fn make<M>(state: &State, data: M) -> M::Out
where
    M: Make,
{
    data.make(Maker {
        state,
        usage: TextureUsages::empty(),
    })
}

#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    pub(crate) const fn wgpu(self) -> FilterMode {
        match self {
            Self::Nearest => FilterMode::Nearest,
            Self::Linear => FilterMode::Linear,
        }
    }
}

pub struct Sampler(wgpu::Sampler);

impl Sampler {
    pub(crate) fn new(state: &State, filter: Filter) -> Self {
        use wgpu::*;

        let inner = {
            let filter = filter.wgpu();
            let desc = SamplerDescriptor {
                mag_filter: filter,
                min_filter: filter,
                ..Default::default()
            };

            state.device().create_sampler(&desc)
        };

        Self(inner)
    }

    pub(crate) fn inner(&self) -> &wgpu::Sampler {
        &self.0
    }
}

pub struct CopyBuffer {
    buf: Buffer,
    size: (u32, u32),
    pixel_size: u32,
}

impl CopyBuffer {
    pub(crate) fn new(state: &State, (width, height): (u32, u32)) -> Self {
        use wgpu::*;

        const PIXEL_SIZE: u32 = mem::size_of::<u32>() as u32;
        const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / PIXEL_SIZE;

        let actual_width = util::align_to(width, ALIGNMENT);
        let buf = {
            let desc = BufferDescriptor {
                label: None,
                size: (actual_width * height * PIXEL_SIZE) as BufferAddress,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            state.device().create_buffer(&desc)
        };

        Self {
            buf,
            size: (actual_width, height),
            pixel_size: PIXEL_SIZE,
        }
    }

    pub(crate) fn copy_texture(&self, texture: &Texture, encoder: &mut CommandEncoder) {
        use wgpu::*;

        let texture = &texture.inner;
        let (width, height) = self.size;

        assert!(
            texture.width() <= width && texture.height() == height,
            "texture size doesn't match buffer size",
        );

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &self.buf,
                layout: ImageDataLayout {
                    bytes_per_row: Some(width * self.pixel_size),
                    rows_per_image: Some(height),
                    ..Default::default()
                },
            },
            texture.size(),
        );
    }

    pub fn view(&self) -> CopyBufferView {
        CopyBufferView(self.buf.slice(..))
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Drop for CopyBuffer {
    fn drop(&mut self) {
        self.buf.unmap();
        self.buf.destroy();
    }
}

pub type MapResult = Result<(), BufferAsyncError>;

#[derive(Clone, Copy)]
pub struct CopyBufferView<'a>(BufferSlice<'a>);

impl<'a> CopyBufferView<'a> {
    pub(crate) async fn map<S, R, F>(self, state: &State, tx: S, rx: R) -> Mapped<'a>
    where
        S: FnOnce(MapResult) + WasmNotSend + 'static,
        R: FnOnce() -> F,
        F: IntoFuture<Output = MapResult>,
    {
        use wgpu::*;

        self.0.map_async(MapMode::Read, tx);
        state.device().poll(Maintain::Wait);
        if let Err(err) = rx().await {
            panic!("failed to copy texture: {err}");
        }

        Mapped(self.0.get_mapped_range())
    }
}

pub struct Mapped<'a>(BufferView<'a>);

impl Mapped<'_> {
    pub fn data(&self) -> &[[u8; 4]] {
        bytemuck::cast_slice(&self.0)
    }
}

trait Get {
    fn get(&self) -> &Texture;
}

impl Get for Texture {
    fn get(&self) -> &Texture {
        self
    }
}

pub trait BindTexture: private::Sealed {
    fn bind_texture(&self) -> &Texture;
}

impl<M> BindTexture for Bind<M>
where
    M: Get,
{
    fn bind_texture(&self) -> &Texture {
        self.0.get()
    }
}

impl<M> BindTexture for Draw<M>
where
    M: BindTexture,
{
    fn bind_texture(&self) -> &Texture {
        self.0.bind_texture()
    }
}

impl<M> BindTexture for Copy<M>
where
    M: BindTexture,
{
    fn bind_texture(&self) -> &Texture {
        self.0.bind_texture()
    }
}

pub trait DrawTexture: private::Sealed {
    fn draw_texture(&self) -> &Texture;
}

impl<M> DrawTexture for Bind<M>
where
    M: DrawTexture,
{
    fn draw_texture(&self) -> &Texture {
        self.0.draw_texture()
    }
}

impl<M> DrawTexture for Draw<M>
where
    M: Get,
{
    fn draw_texture(&self) -> &Texture {
        self.0.get()
    }
}

impl<M> DrawTexture for Copy<M>
where
    M: DrawTexture,
{
    fn draw_texture(&self) -> &Texture {
        self.0.draw_texture()
    }
}

pub trait CopyTexture: private::Sealed {
    fn copy_texture(&self) -> &Texture;
}

impl<M> CopyTexture for Bind<M>
where
    M: CopyTexture,
{
    fn copy_texture(&self) -> &Texture {
        self.0.copy_texture()
    }
}

impl<M> CopyTexture for Draw<M>
where
    M: CopyTexture,
{
    fn copy_texture(&self) -> &Texture {
        self.0.copy_texture()
    }
}

impl<M> CopyTexture for Copy<M>
where
    M: Get,
{
    fn copy_texture(&self) -> &Texture {
        self.0.get()
    }
}

pub struct Maker<'a> {
    state: &'a State,
    usage: TextureUsages,
}

pub trait Make: private::Sealed {
    type Out;
    fn make(self, maker: Maker) -> Self::Out;
}

impl private::Sealed for TextureData<'_> {}

impl Make for TextureData<'_> {
    type Out = Texture;

    fn make(self, Maker { state, usage }: Maker) -> Self::Out {
        Texture::new(state, usage, self)
    }
}

pub struct Bind<M>(M);

impl<M> Bind<M> {
    pub fn with_draw(self) -> Draw<Self> {
        Draw(self)
    }

    pub fn with_copy(self) -> Copy<Self> {
        Copy(self)
    }
}

impl<M> Get for Bind<M>
where
    M: Get,
{
    fn get(&self) -> &Texture {
        self.0.get()
    }
}

impl<M> private::Sealed for Bind<M> {}

impl<M> Make for Bind<M>
where
    M: Make,
{
    type Out = Bind<M::Out>;

    fn make(self, mut maker: Maker) -> Self::Out {
        maker.usage |= TextureUsages::TEXTURE_BINDING;
        Bind(self.0.make(maker))
    }
}

pub struct Draw<M>(M);

impl<M> Draw<M> {
    pub fn with_bind(self) -> Bind<Self> {
        Bind(self)
    }

    pub fn with_copy(self) -> Copy<Self> {
        Copy(self)
    }
}

impl<M> Get for Draw<M>
where
    M: Get,
{
    fn get(&self) -> &Texture {
        self.0.get()
    }
}

impl<M> private::Sealed for Draw<M> {}

impl<M> Make for Draw<M>
where
    M: Make,
{
    type Out = Draw<M::Out>;

    fn make(self, mut maker: Maker) -> Self::Out {
        maker.usage |= TextureUsages::RENDER_ATTACHMENT;
        Draw(self.0.make(maker))
    }
}

pub struct Copy<M>(M);

impl<M> Copy<M> {
    pub fn with_bind(self) -> Bind<Self> {
        Bind(self)
    }

    pub fn with_draw(self) -> Draw<Self> {
        Draw(self)
    }
}

impl<M> Get for Copy<M>
where
    M: Get,
{
    fn get(&self) -> &Texture {
        self.0.get()
    }
}

impl<M> private::Sealed for Copy<M> {}

impl<M> Make for Copy<M>
where
    M: Make,
{
    type Out = Copy<M::Out>;

    fn make(self, mut maker: Maker) -> Self::Out {
        maker.usage |= TextureUsages::COPY_SRC;
        Copy(self.0.make(maker))
    }
}

mod private {
    pub trait Sealed {}
}
