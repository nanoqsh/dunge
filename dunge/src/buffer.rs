//! The texture module.

use {
    crate::{
        color::{ColorExt, Rgb, Rgba},
        group::BoundTexture,
        runtime::Ticket,
        state::State,
        usage::{BufferNoUsages, TextureNoUsages, Use, u},
    },
    std::{error, fmt, marker::PhantomData, num::NonZeroU32, ops, sync::Arc},
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
    pub const fn bytes(self) -> u32 {
        match self {
            Self::SrgbAlpha | Self::SbgrAlpha | Self::RgbAlpha | Self::BgrAlpha | Self::Depth => 4,
            Self::Byte => 1,
        }
    }

    #[inline]
    pub const fn is_standard(self) -> bool {
        matches!(self, Self::SrgbAlpha | Self::SbgrAlpha)
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

impl ColorExt for Format {
    #[inline]
    fn rgb_from_bytes(self, rgb: [u8; 3]) -> Rgb {
        if self.is_standard() {
            Rgb::from_standard_bytes(rgb)
        } else {
            Rgb::from_bytes(rgb)
        }
    }

    #[inline]
    fn rgba_from_bytes(self, rgba: [u8; 4]) -> Rgba {
        if self.is_standard() {
            Rgba::from_standard_bytes(rgba)
        } else {
            Rgba::from_bytes(rgba)
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

pub struct Texture<D, U> {
    inner: TextureInner,
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
            inner: TextureInner::new(state, new),
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
    pub fn bytes_per_row_aligned(&self) -> u32 {
        self.inner.bytes_per_row_aligned
    }

    #[inline]
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.inner.view
    }

    #[inline]
    pub fn copy_buffer_data<'data>(&self) -> BufferData<'data, BufferCopyTo> {
        let size = self.inner.texture.size();
        let size = self.inner.bytes_per_row_aligned * size.height * size.depth_or_array_layers;
        BufferData::empty(size).copy_to()
    }
}

type BufferCopyTo = <BufferNoUsages as Use<dyn u::CopyTo>>::Out;

pub type Texture2d<U> = Texture<D2, U>;

impl<U> Texture2d<U> {
    #[inline]
    pub fn bind(&self) -> BoundTexture<'_>
    where
        U: u::Bind,
    {
        BoundTexture(self.view())
    }
}

struct TextureInner {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bytes_per_row_aligned: u32,
}

impl TextureInner {
    fn new(state: &State, new: NewTexture<'_>) -> Self {
        use wgpu::util;

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

        let bytes_per_row = size.width * format.bytes();
        let bytes_per_row_aligned =
            util::align_to(bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

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
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = {
            let desc = wgpu::TextureViewDescriptor::default();
            texture.create_view(&desc)
        };

        Self {
            texture,
            view,
            bytes_per_row_aligned,
        }
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

pub struct BufferData<'data, U> {
    data: &'data [u8],
    size: u32,
    usage: PhantomData<U>,
}

impl<'data> BufferData<'data, BufferNoUsages> {
    #[inline]
    pub fn empty(size: u32) -> Self {
        Self {
            data: &[],
            size,
            usage: PhantomData,
        }
    }

    #[inline]
    pub fn new(data: &'data [u8]) -> Self {
        let Ok(size) = data.len().try_into() else {
            panic!("the buffer size doesn't fit into u32");
        };

        let empty = Self::empty(size);
        Self { data, ..empty }
    }
}

impl<'data, U> BufferData<'data, U> {
    #[inline]
    fn to<T>(self) -> BufferData<'data, U::Out>
    where
        T: ?Sized,
        U: Use<T>,
    {
        BufferData {
            data: self.data,
            size: self.size,
            usage: PhantomData,
        }
    }

    #[inline]
    pub fn read(self) -> BufferData<'data, U::Out>
    where
        U: Use<dyn u::Read>,
    {
        self.to()
    }

    #[inline]
    pub fn write(self) -> BufferData<'data, U::Out>
    where
        U: Use<dyn u::Write>,
    {
        self.to()
    }

    #[inline]
    pub fn copy_from(self) -> BufferData<'data, U::Out>
    where
        U: Use<dyn u::CopyFrom>,
    {
        self.to()
    }

    #[inline]
    pub fn copy_to(self) -> BufferData<'data, U::Out>
    where
        U: Use<dyn u::CopyTo>,
    {
        self.to()
    }
}

pub struct Buffer<U> {
    inner: BufferInner,
    ty: PhantomData<U>,
}

impl<U> Buffer<U> {
    #[inline]
    pub(crate) fn new(state: &State, data: BufferData<'_, U>) -> Self
    where
        U: u::BufferUsages,
    {
        let new = NewBuffer {
            data: data.data,
            size: data.size,
            usage: U::usages(),
        };

        Self {
            inner: BufferInner::new(state, new),
            ty: PhantomData,
        }
    }

    #[inline]
    pub(crate) async fn read(&mut self, state: &State) -> Result<Read<'_>, ReadFailed>
    where
        U: u::Read,
    {
        self.inner.read(state).await
    }

    #[inline]
    pub(crate) async fn write(&mut self, state: &State) -> Result<Write<'_>, WriteFailed>
    where
        U: u::Write,
    {
        self.inner.write(state).await
    }
}

struct BufferInner(wgpu::Buffer);

impl BufferInner {
    fn new(state: &State, new: NewBuffer<'_>) -> Self {
        use wgpu::util::{self, DeviceExt};

        let NewBuffer { size, data, usage } = new;

        let buf = if data.is_empty() {
            let desc = wgpu::BufferDescriptor {
                label: None,
                size: u64::from(size),
                usage,
                mapped_at_creation: false,
            };

            state.device().create_buffer(&desc)
        } else {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents: data,
                usage,
            };

            state.device().create_buffer_init(&desc)
        };

        Self(buf)
    }

    async fn read(&mut self, state: &State) -> Result<Read<'_>, ReadFailed> {
        let ticket = Arc::new(const { Ticket::new() });

        self.0.map_async(wgpu::MapMode::Read, .., {
            let ticket = ticket.clone();
            move |res| {
                if res.is_ok() {
                    ticket.done();
                } else {
                    ticket.fail();
                }
            }
        });

        state.work();
        if ticket.wait().await {
            let view = self.0.get_mapped_range(..);

            Ok(Read {
                view,
                _unmap: Unmap(&self.0),
            })
        } else {
            Err(ReadFailed)
        }
    }

    async fn write(&mut self, state: &State) -> Result<Write<'_>, WriteFailed> {
        let ticket = Arc::new(const { Ticket::new() });

        self.0.map_async(wgpu::MapMode::Write, .., {
            let ticket = ticket.clone();
            move |res| {
                if res.is_ok() {
                    ticket.done();
                } else {
                    ticket.fail();
                }
            }
        });

        state.work();
        if ticket.wait().await {
            let view = self.0.get_mapped_range_mut(..);

            Ok(Write {
                view,
                _unmap: Unmap(&self.0),
            })
        } else {
            Err(WriteFailed)
        }
    }
}

struct NewBuffer<'data> {
    data: &'data [u8],
    size: u32,
    usage: wgpu::BufferUsages,
}

struct Unmap<'buf>(&'buf wgpu::Buffer);

impl Drop for Unmap<'_> {
    fn drop(&mut self) {
        self.0.unmap();
    }
}

pub struct Read<'buf> {
    view: wgpu::BufferView<'buf>,

    // drop order is important here!
    // `Unmap` unmaps the view in drop
    // while it should be already dropped
    _unmap: Unmap<'buf>,
}

impl ops::Deref for Read<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl AsRef<[u8]> for Read<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.view
    }
}

#[derive(Debug)]
pub struct ReadFailed;

impl fmt::Display for ReadFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to read buffer")
    }
}

impl error::Error for ReadFailed {}

pub struct Write<'buf> {
    view: wgpu::BufferViewMut<'buf>,

    // drop order is important here!
    // `Unmap` unmaps the view in drop
    // while it should be already dropped
    _unmap: Unmap<'buf>,
}

impl ops::Deref for Write<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl ops::DerefMut for Write<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}

impl AsRef<[u8]> for Write<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.view
    }
}

impl AsMut<[u8]> for Write<'_> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.view
    }
}

#[derive(Debug)]
pub struct WriteFailed;

impl fmt::Display for WriteFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to write buffer")
    }
}

impl error::Error for WriteFailed {}

enum Inner<'inner> {
    Texture(&'inner TextureInner),
    Buffer(&'inner BufferInner),
}

mod i {
    pub struct Wrap<'inner>(pub(super) super::Inner<'inner>);

    pub trait AsInner {
        fn as_inner(&self) -> Wrap<'_>;
    }
}

impl<I> i::AsInner for &I
where
    I: i::AsInner,
{
    fn as_inner(&self) -> i::Wrap<'_> {
        (**self).as_inner()
    }
}

impl<D, U> i::AsInner for Texture<D, U> {
    fn as_inner(&self) -> i::Wrap<'_> {
        i::Wrap(Inner::Texture(&self.inner))
    }
}

impl<U> i::AsInner for Buffer<U> {
    fn as_inner(&self) -> i::Wrap<'_> {
        i::Wrap(Inner::Buffer(&self.inner))
    }
}

pub trait Source: i::AsInner {}

impl<S> Source for &S where S: Source {}
impl<D, U> Source for Texture<D, U> where U: u::CopyFrom {}
impl<U> Source for Buffer<U> where U: u::CopyFrom {}

pub trait Destination: i::AsInner {}

impl<D> Destination for &D where D: Destination {}
impl<D, U> Destination for Texture<D, U> where U: u::CopyTo {}
impl<U> Destination for Buffer<U> where U: u::CopyTo {}

#[inline]
pub(crate) fn try_copy<S, D>(from: S, to: D, en: &mut wgpu::CommandEncoder) -> Result<(), SizeError>
where
    S: Source,
    D: Destination,
{
    let i::Wrap(from) = from.as_inner();
    let i::Wrap(to) = to.as_inner();

    match (from, to) {
        (Inner::Texture(_from), Inner::Texture(_to)) => todo!(),
        (Inner::Texture(from), Inner::Buffer(to)) => copy_texture_to_buffer(from, to, en),
        (Inner::Buffer(_from), Inner::Texture(_to)) => todo!(),
        (Inner::Buffer(from), Inner::Buffer(to)) => copy_buffer_to_buffer(from, to, en),
    }
}

#[derive(Debug)]
pub struct SizeError {
    pub from_size: u64,
    pub to_size: u64,
}

impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { from_size, to_size } = self;
        write!(f, "can't copy from size {from_size} to size {to_size}")
    }
}

impl error::Error for SizeError {}

fn copy_texture_to_buffer(
    from: &TextureInner,
    to: &BufferInner,
    en: &mut wgpu::CommandEncoder,
) -> Result<(), SizeError> {
    let size = from.texture.size();
    let from_size = u64::from(from.bytes_per_row_aligned)
        * u64::from(size.height)
        * u64::from(size.depth_or_array_layers);

    let to_size = to.0.size();
    if from_size != to_size {
        return Err(SizeError { from_size, to_size });
    }

    en.copy_texture_to_buffer(
        from.texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &to.0,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(from.bytes_per_row_aligned),
                rows_per_image: Some(size.height),
            },
        },
        size,
    );

    Ok(())
}

fn copy_buffer_to_buffer(
    from: &BufferInner,
    to: &BufferInner,
    en: &mut wgpu::CommandEncoder,
) -> Result<(), SizeError> {
    let from_size = from.0.size();
    let to_size = to.0.size();
    if from_size != to_size {
        return Err(SizeError { from_size, to_size });
    }

    en.copy_buffer_to_buffer(&from.0, 0, &to.0, 0, from.0.size());
    Ok(())
}
