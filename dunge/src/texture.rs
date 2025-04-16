//! The texture module.

use {
    crate::state::State,
    std::{error, fmt, num::NonZeroU32},
};

/// The texture format type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    SrgbAlpha,
    SbgrAlpha,
    RgbAlpha,
    BgrAlpha,
    Depth,
    Byte,
}

impl Format {
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
    }

    pub(crate) const fn wgpu(self) -> wgpu::TextureFormat {
        match self {
            Self::SrgbAlpha => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::SbgrAlpha => wgpu::TextureFormat::Bgra8UnormSrgb,
            Self::RgbAlpha => wgpu::TextureFormat::Rgba8Unorm,
            Self::BgrAlpha => wgpu::TextureFormat::Bgra8Unorm,
            Self::Depth => wgpu::TextureFormat::Depth32Float,
            Self::Byte => wgpu::TextureFormat::R8Uint,
        }
    }

    pub(crate) const fn from_wgpu(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => Self::SrgbAlpha,
            wgpu::TextureFormat::Bgra8UnormSrgb => Self::SbgrAlpha,
            wgpu::TextureFormat::Rgba8Unorm => Self::RgbAlpha,
            wgpu::TextureFormat::Bgra8Unorm => Self::BgrAlpha,
            wgpu::TextureFormat::Depth32Float => Self::Depth,
            wgpu::TextureFormat::R8Uint => Self::Byte,
            _ => panic!("unsupported format"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub depth: NonZeroU32,
}

impl Size {
    fn volume(self) -> usize {
        let width = self.width.get() as usize;
        let height = self.height.get() as usize;
        let depth = self.depth.get() as usize;
        width * height * depth
    }

    fn wgpu(self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width.get(),
            height: self.height.get(),
            depth_or_array_layers: self.depth.get(),
        }
    }
}

impl TryFrom<u32> for Size {
    type Error = ZeroSized;

    fn try_from(width: u32) -> Result<Self, Self::Error> {
        let width = NonZeroU32::new(width).ok_or(ZeroSized)?;
        Ok(Self::from(width))
    }
}

impl From<NonZeroU32> for Size {
    fn from(width: NonZeroU32) -> Self {
        Self {
            width,
            height: NonZeroU32::MIN,
            depth: NonZeroU32::MIN,
        }
    }
}

impl TryFrom<(u32, u32)> for Size {
    type Error = ZeroSized;

    fn try_from((width, height): (u32, u32)) -> Result<Self, Self::Error> {
        let width = NonZeroU32::new(width).ok_or(ZeroSized)?;
        let height = NonZeroU32::new(height).ok_or(ZeroSized)?;
        Ok(Self::from((width, height)))
    }
}

impl From<(NonZeroU32, NonZeroU32)> for Size {
    fn from((width, height): (NonZeroU32, NonZeroU32)) -> Self {
        Self {
            width,
            height,
            depth: NonZeroU32::MIN,
        }
    }
}

#[derive(Debug)]
pub struct ZeroSized;

impl fmt::Display for ZeroSized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "zero sized data")
    }
}

impl error::Error for ZeroSized {}

#[derive(Clone, Copy)]
pub struct TextureData<'data> {
    size: Size,
    format: Format,
    data: &'data [u8],
}

impl<'data> TextureData<'data> {
    pub fn empty<S>(size: S, format: Format) -> Self
    where
        S: Into<Size>,
    {
        let size = size.into();
        let data = &[];
        Self { size, format, data }
    }

    pub fn new<S>(size: S, format: Format, data: &'data [u8]) -> Result<Self, InvalidLen>
    where
        S: Into<Size>,
    {
        let empty = Self::empty(size, format);
        let len = empty.size.volume() * format.bytes() as usize;
        if data.len() == len {
            Ok(Self { data, ..empty })
        } else {
            Err(InvalidLen)
        }
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
    pub fn with_copy(self) -> _Copy<Self> {
        _Copy(self)
    }
}

/// The texture data length doesn't match with size and format.
#[derive(Debug)]
pub struct InvalidLen;

impl fmt::Display for InvalidLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid data length")
    }
}

impl error::Error for InvalidLen {}

pub struct Texture2d {
    inner: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture2d {
    fn new(state: &State, mut usage: wgpu::TextureUsages, data: TextureData<'_>) -> Self {
        let size = data.size.wgpu();
        let copy_data = !data.data.is_empty();

        if copy_data {
            usage |= wgpu::TextureUsages::COPY_DST;
        }

        let inner = {
            let desc = wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: data.format.wgpu(),
                usage,
                view_formats: &[],
            };

            state.device().create_texture(&desc)
        };

        if copy_data {
            state.queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &inner,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * data.format.bytes()),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = {
            let desc = wgpu::TextureViewDescriptor::default();
            inner.create_view(&desc)
        };

        Self { inner, view }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    pub fn format(&self) -> Format {
        Format::from_wgpu(self.inner.format())
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

pub(crate) fn make<M>(state: &State, data: M) -> M::Out
where
    M: Make,
{
    data.make(Maker {
        state,
        usage: wgpu::TextureUsages::empty(),
    })
}

#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    pub(crate) const fn wgpu(self) -> wgpu::FilterMode {
        match self {
            Self::Nearest => wgpu::FilterMode::Nearest,
            Self::Linear => wgpu::FilterMode::Linear,
        }
    }
}

pub struct Sampler(wgpu::Sampler);

impl Sampler {
    pub(crate) fn new(state: &State, filter: Filter) -> Self {
        let inner = {
            let filter = filter.wgpu();
            let desc = wgpu::SamplerDescriptor {
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
    buf: wgpu::Buffer,
    size: (u32, u32),
    pixel_size: u32,
}

impl CopyBuffer {
    pub(crate) fn new(state: &State, (width, height): (u32, u32)) -> Self {
        use wgpu::util;

        let (pixel_size, alignment) = const {
            let pixel_size = size_of::<u32>() as u32;
            let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / pixel_size;
            (pixel_size, alignment)
        };

        let actual_width = util::align_to(width, alignment);
        let buf = {
            let desc = wgpu::BufferDescriptor {
                label: None,
                size: wgpu::BufferAddress::from(actual_width * height * pixel_size),
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            state.device().create_buffer(&desc)
        };

        Self {
            buf,
            size: (actual_width, height),
            pixel_size,
        }
    }

    pub(crate) fn copy_texture(&self, texture: &Texture2d, encoder: &mut wgpu::CommandEncoder) {
        let texture = &texture.inner;
        let (width, height) = self.size;

        assert!(
            texture.width() <= width && texture.height() == height,
            "texture size doesn't match buffer size",
        );

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.buf,
                layout: wgpu::TexelCopyBufferLayout {
                    bytes_per_row: Some(width * self.pixel_size),
                    rows_per_image: Some(height),
                    ..Default::default()
                },
            },
            texture.size(),
        );
    }

    pub fn view(&self) -> CopyBufferView<'_> {
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

pub type MapResult = Result<(), wgpu::BufferAsyncError>;

#[derive(Clone, Copy)]
pub struct CopyBufferView<'slice>(wgpu::BufferSlice<'slice>);

impl<'slice> CopyBufferView<'slice> {
    pub(crate) async fn map<S, R>(self, state: &State, tx: S, rx: R) -> Mapped<'slice>
    where
        S: FnOnce(MapResult) + wgpu::WasmNotSend + 'static,
        R: IntoFuture<Output = MapResult>,
    {
        use wgpu::*;

        self.0.map_async(MapMode::Read, tx);
        _ = state.device().poll(PollType::Wait);
        if let Err(err) = rx.await {
            panic!("failed to copy texture: {err}");
        }

        Mapped(self.0.get_mapped_range())
    }
}

pub struct Mapped<'slice>(wgpu::BufferView<'slice>);

impl Mapped<'_> {
    pub fn data(&self) -> &[[u8; 4]] {
        bytemuck::cast_slice(&self.0)
    }
}

trait Get {
    fn get(&self) -> &Texture2d;
}

impl Get for Texture2d {
    fn get(&self) -> &Texture2d {
        self
    }
}

pub trait BindTexture: private::Sealed {
    fn bind_texture(&self) -> &Texture2d;
}

impl<M> BindTexture for Bind<M>
where
    M: Get,
{
    fn bind_texture(&self) -> &Texture2d {
        self.0.get()
    }
}

impl<M> BindTexture for Draw<M>
where
    M: BindTexture,
{
    fn bind_texture(&self) -> &Texture2d {
        self.0.bind_texture()
    }
}

impl<M> BindTexture for _Copy<M>
where
    M: BindTexture,
{
    fn bind_texture(&self) -> &Texture2d {
        self.0.bind_texture()
    }
}

pub trait DrawTexture: private::Sealed {
    fn draw_texture(&self) -> &Texture2d;
}

impl<D> private::Sealed for &D {}

impl<D> DrawTexture for &D
where
    D: DrawTexture,
{
    fn draw_texture(&self) -> &Texture2d {
        (**self).draw_texture()
    }
}

impl<M> DrawTexture for Bind<M>
where
    M: DrawTexture,
{
    fn draw_texture(&self) -> &Texture2d {
        self.0.draw_texture()
    }
}

impl<M> DrawTexture for Draw<M>
where
    M: Get,
{
    fn draw_texture(&self) -> &Texture2d {
        self.0.get()
    }
}

impl<M> DrawTexture for _Copy<M>
where
    M: DrawTexture,
{
    fn draw_texture(&self) -> &Texture2d {
        self.0.draw_texture()
    }
}

pub trait CopyTexture: private::Sealed {
    fn copy_texture(&self) -> &Texture2d;
}

impl<M> CopyTexture for Bind<M>
where
    M: CopyTexture,
{
    fn copy_texture(&self) -> &Texture2d {
        self.0.copy_texture()
    }
}

impl<M> CopyTexture for Draw<M>
where
    M: CopyTexture,
{
    fn copy_texture(&self) -> &Texture2d {
        self.0.copy_texture()
    }
}

impl<M> CopyTexture for _Copy<M>
where
    M: Get,
{
    fn copy_texture(&self) -> &Texture2d {
        self.0.get()
    }
}

pub struct Maker<'state> {
    state: &'state State,
    usage: wgpu::TextureUsages,
}

pub trait Make: private::Sealed {
    type Out;
    fn make(self, maker: Maker<'_>) -> Self::Out;
}

impl private::Sealed for TextureData<'_> {}

impl Make for TextureData<'_> {
    type Out = Texture2d;

    fn make(self, Maker { state, usage }: Maker<'_>) -> Self::Out {
        Texture2d::new(state, usage, self)
    }
}

pub struct Bind<M>(M);

impl<M> Bind<M> {
    pub fn with_draw(self) -> Draw<Self> {
        Draw(self)
    }

    pub fn with_copy(self) -> _Copy<Self> {
        _Copy(self)
    }
}

impl<M> Get for Bind<M>
where
    M: Get,
{
    fn get(&self) -> &Texture2d {
        self.0.get()
    }
}

impl<M> private::Sealed for Bind<M> {}

impl<M> Make for Bind<M>
where
    M: Make,
{
    type Out = Bind<M::Out>;

    fn make(self, mut maker: Maker<'_>) -> Self::Out {
        maker.usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        Bind(self.0.make(maker))
    }
}

pub struct Draw<M>(M);

impl<M> Draw<M> {
    pub fn with_bind(self) -> Bind<Self> {
        Bind(self)
    }

    pub fn with_copy(self) -> _Copy<Self> {
        _Copy(self)
    }
}

impl<M> Get for Draw<M>
where
    M: Get,
{
    fn get(&self) -> &Texture2d {
        self.0.get()
    }
}

impl<M> private::Sealed for Draw<M> {}

impl<M> Make for Draw<M>
where
    M: Make,
{
    type Out = Draw<M::Out>;

    fn make(self, mut maker: Maker<'_>) -> Self::Out {
        maker.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        Draw(self.0.make(maker))
    }
}

pub struct _Copy<M>(M);

impl<M> _Copy<M> {
    pub fn with_bind(self) -> Bind<Self> {
        Bind(self)
    }

    pub fn with_draw(self) -> Draw<Self> {
        Draw(self)
    }
}

impl<M> Get for _Copy<M>
where
    M: Get,
{
    fn get(&self) -> &Texture2d {
        self.0.get()
    }
}

impl<M> private::Sealed for _Copy<M> {}

impl<M> Make for _Copy<M>
where
    M: Make,
{
    type Out = _Copy<M::Out>;

    fn make(self, mut maker: Maker<'_>) -> Self::Out {
        maker.usage |= wgpu::TextureUsages::COPY_SRC;
        _Copy(self.0.make(maker))
    }
}

mod private {
    pub trait Sealed {}
}
