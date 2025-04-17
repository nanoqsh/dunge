//! The texture module.

use {
    crate::{
        state::State,
        usage::{TextureNoUsages, Use, u},
    },
    std::{error, fmt, marker::PhantomData, num::NonZeroU32},
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
    #[inline]
    pub(crate) const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
    }

    #[inline]
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

    #[inline]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
    pub depth: NonZeroU32,
}

impl Size {
    #[inline]
    fn volume(self) -> usize {
        let width = self.width.get() as usize;
        let height = self.height.get() as usize;
        let depth = self.depth.get() as usize;
        width * height * depth
    }

    #[inline]
    fn wgpu(self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width.get(),
            height: self.height.get(),
            depth_or_array_layers: self.depth.get(),
        }
    }

    #[inline]
    fn from_wgpu(ex: wgpu::Extent3d) -> Self {
        let make = || {
            Some(Self {
                width: NonZeroU32::new(ex.width)?,
                height: NonZeroU32::new(ex.height)?,
                depth: NonZeroU32::new(ex.depth_or_array_layers)?,
            })
        };

        make().expect("non zero sized")
    }
}

impl TryFrom<u32> for Size {
    type Error = ZeroSized;

    #[inline]
    fn try_from(width: u32) -> Result<Self, Self::Error> {
        let width = NonZeroU32::new(width).ok_or(ZeroSized)?;
        Ok(Self::from(width))
    }
}

impl From<NonZeroU32> for Size {
    #[inline]
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

    #[inline]
    fn try_from((width, height): (u32, u32)) -> Result<Self, Self::Error> {
        let width = NonZeroU32::new(width).ok_or(ZeroSized)?;
        let height = NonZeroU32::new(height).ok_or(ZeroSized)?;
        Ok(Self::from((width, height)))
    }
}

impl From<(NonZeroU32, NonZeroU32)> for Size {
    #[inline]
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
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "zero sized data")
    }
}

impl error::Error for ZeroSized {}

#[derive(Clone, Copy)]
pub struct TextureData<'data, U> {
    data: &'data [u8],
    size: Size,
    format: Format,
    usage: PhantomData<U>,
}

impl<'data> TextureData<'data, TextureNoUsages> {
    #[inline]
    pub fn empty<S>(size: S, format: Format) -> Self
    where
        S: Into<Size>,
    {
        Self {
            data: &[],
            size: size.into(),
            format,
            usage: PhantomData,
        }
    }

    #[inline]
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
}

impl<'data, U> TextureData<'data, U> {
    #[inline]
    fn to<T>(self) -> TextureData<'data, U::Out>
    where
        T: ?Sized,
        U: Use<T>,
    {
        TextureData {
            data: self.data,
            size: self.size,
            format: self.format,
            usage: PhantomData,
        }
    }

    #[inline]
    pub fn read(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::Read>,
    {
        self.to()
    }

    #[inline]
    pub fn write(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::Write>,
    {
        self.to()
    }

    #[inline]
    pub fn copy_from(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::CopyFrom>,
    {
        self.to()
    }

    #[inline]
    pub fn copy_to(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::CopyTo>,
    {
        self.to()
    }

    #[inline]
    pub fn bind(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::Bind>,
    {
        self.to()
    }

    #[inline]
    pub fn render(self) -> TextureData<'data, U::Out>
    where
        U: Use<dyn u::Render>,
    {
        self.to()
    }
}

/// The texture data length doesn't match with size and format.
#[derive(Debug)]
pub struct InvalidLen;

impl fmt::Display for InvalidLen {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid data length")
    }
}

impl error::Error for InvalidLen {}

pub enum DimensionsNumber {
    D1,
    D2,
    D3,
}

impl DimensionsNumber {
    #[inline]
    fn wgpu(self) -> wgpu::TextureDimension {
        match self {
            Self::D1 => wgpu::TextureDimension::D1,
            Self::D2 => wgpu::TextureDimension::D2,
            Self::D3 => wgpu::TextureDimension::D3,
        }
    }
}

pub trait Dimension {
    const N: DimensionsNumber;
}

pub enum D2 {}

impl Dimension for D2 {
    const N: DimensionsNumber = DimensionsNumber::D2;
}

pub type Texture2d<U> = Texture<D2, U>;

pub struct Texture<D, U> {
    inner: Inner,
    ty: PhantomData<(D, U)>,
}

impl<D, U> Texture<D, U> {
    #[inline]
    pub(crate) fn new(state: &State, data: TextureData<'_, U>) -> Self
    where
        D: Dimension,
        U: u::TextureUsages,
    {
        let new = NewTexture {
            data: data.data,
            size: data.size,
            format: data.format,
            dimension: D::N,
            usage: U::usages(),
        };

        Self {
            inner: Inner::new(state, new),
            ty: PhantomData,
        }
    }

    #[inline]
    pub fn size(&self) -> Size {
        Size::from_wgpu(self.inner.texture.size())
    }

    #[inline]
    pub fn format(&self) -> Format {
        Format::from_wgpu(self.inner.texture.format())
    }

    #[inline]
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.inner.view
    }
}

struct Inner {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Inner {
    fn new(state: &State, new: NewTexture<'_>) -> Self {
        let NewTexture {
            size,
            data,
            format,
            dimension,
            mut usage,
        } = new;

        let size = size.wgpu();
        let copy_data = !data.is_empty();

        let texture = {
            if copy_data {
                usage |= wgpu::TextureUsages::COPY_DST;
            }

            let desc = wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: dimension.wgpu(),
                format: format.wgpu(),
                usage,
                view_formats: &[],
            };

            state.device().create_texture(&desc)
        };

        if copy_data {
            state.queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * format.bytes()),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = {
            let desc = wgpu::TextureViewDescriptor::default();
            texture.create_view(&desc)
        };

        Self { texture, view }
    }
}

struct NewTexture<'data> {
    data: &'data [u8],
    size: Size,
    format: Format,
    dimension: DimensionsNumber,
    usage: wgpu::TextureUsages,
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

    pub(crate) fn copy_texture<U>(
        &self,
        texture: &Texture2d<U>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let texture = &texture.inner.texture;
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
