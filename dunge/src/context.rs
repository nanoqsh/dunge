use {
    crate::{
        buffer::{
            self, Buffer, Filter, Read, ReadFailed, TextureBuffer, TextureSampler, Write,
            WriteFailed,
        },
        layer::{Config, Layer},
        mesh::{self, Mesh},
        render,
        set::{self, Group, GroupHandler, Groups, Nth, UniqueSet},
        shader::{RenderShader, Shader},
        state::{Scheduler, State},
        store::{self, Row, Storage, StorageValue, Uniform},
        usage::u,
    },
    dunge_shade::{bytes::Bytes, irc::Value, link::Render},
    std::{error, fmt, sync::Arc},
};

/// Creates the [context](Context) instance.
///
/// This function returns an async builder, which must be `.await`ed to create
/// the context object. To do this, use the [`block_on`](crate::block_on) function
/// on desktop platform.
///
/// # On desktop example
///
/// ```
/// # fn f() -> Result<(), dunge::FailedMakeContext> {
/// let cx = dunge::block_on(dunge::context())?;
/// // Use the context
/// # Ok(())
/// # }
/// ```
///
/// If you're using the library in windowed mode via the
/// [`dunge-winit`](https://docs.rs/dunge-winit/latest/dunge-winit/index.html) crate, use
/// [`dunge_winit::block_on`](https://docs.rs/dunge_winit/latest/dunge_winit/fn.block_on.html) or
/// [`dunge_winit::try_block_on`](https://docs.rs/dunge_winit/latest/dunge_winit/fn.try_block_on.html)
/// instead.
///
/// # On wasm example
///
/// On wasm platform use the browser's runtime directly - no blocking
/// functions are needed in this case.
///
/// ```
/// # #[cfg(false)]
/// #[wasm_bindgen(start)]
/// async fn start() {
///     let cx = match dunge::context().await {
///         Ok(cx) => cx,
///         Err(e) => panic!("failed to create dunge context: {e}"),
///     };
///
///     // Use the context
/// }
/// ```
///
/// # Errors
///
/// The builder returns an error when the context could not be created.
/// See [`FailedMakeContext`] for details.
pub async fn context() -> Result<Context, FailedMakeContext> {
    let state = State::new().await?;
    Ok(Context(Arc::new(state)))
}

/// The main dunge context.
#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    pub fn make_shader<I, S>(&self, render: Render<I, S>) -> Shader<I, S> {
        Shader::new(&self.0, render)
    }

    pub fn make_set<I, G, const N: usize>(
        &self,
        shader: &Shader<I, G::Inner>,
        set: G,
    ) -> UniqueSet<G::Inner>
    where
        G: Groups<N>,
    {
        UniqueSet::new(&self.0, shader.data(), set)
    }

    /// Creates a [uniform](Uniform) from the given value.
    pub fn make_uniform<V>(&self, value: &V) -> Uniform<V>
    where
        V: Value + Bytes,
    {
        const { assert!(size_of::<V>() > 0, "value cannot be zero sized") }
        store::uniform(value, self)
    }

    /// Creates a [storage](Storage) from the given value.
    pub fn make_storage<V>(&self, value: &V) -> Option<Storage<V>>
    where
        V: StorageValue + ?Sized,
    {
        store::storage(value, self)
    }

    /// Creates a [layer](Layer) for the given [render shader](RenderShader).
    ///
    /// This method also accepts a [config](Config) which defines the layer's properties.
    pub fn make_layer<V, I, S, C>(
        &self,
        shader: &RenderShader<S, V, I>,
        conf: C,
    ) -> Layer<render::Input<V, I, S>>
    where
        C: Into<Config>,
    {
        let conf = conf.into();
        Layer::new(&self.0, shader.data(), conf)
    }

    /// Creates a [mesh](Mesh) with the given [data](mesh::MeshData).
    pub fn make_mesh<V>(&self, data: &mesh::MeshData<'_, V>) -> Mesh<V>
    where
        V: Bytes,
    {
        Mesh::new(&self.0, data)
    }

    /// Creates a [row](Row) with the given data.
    pub fn make_row<V>(&self, value: &[V]) -> Option<Row<V>>
    where
        V: Value + Bytes,
    {
        store::row(value, self)
    }

    /// Creates a [2D texture](TextureBuffer) with the given [data](buffer::TextureData).
    pub fn make_texture<U>(&self, data: buffer::TextureData<'_, U>) -> TextureBuffer<2, U>
    where
        U: u::TextureUsages,
    {
        TextureBuffer::new(&self.0, data)
    }

    /// Creates a [sampler](TextureSampler) with the [filter](Filter) value.
    pub fn make_sampler(&self, filter: Filter) -> TextureSampler {
        TextureSampler::new(&self.0, filter)
    }

    /// Creates a [buffer](Buffer) with the given [data](buffer::BufferData).
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn f() -> Result<(), dunge::FailedMakeContext> {
    /// use dunge::buffer::BufferData;
    ///
    /// let cx = dunge::context().await?;
    ///
    /// // Create a buffer filled with four `i32` numbers
    /// let data = BufferData::new(&[1, 2, 3, 4])
    ///     .read()     // set a usage to read from the buffer
    ///     .copy_to(); // set a usage to copy to the buffer
    ///
    /// let buffer = cx.make_buffer(data);
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_buffer<U>(&self, data: buffer::BufferData<'_, U>) -> Buffer<U>
    where
        U: u::BufferUsages,
    {
        Buffer::new(&self.0, data)
    }

    /// Reads from a buffer.
    #[inline]
    pub async fn read<'buf, U>(&self, buf: &'buf mut Buffer<U>) -> Result<Read<'buf>, ReadFailed>
    where
        U: u::Read,
    {
        buf.read(&self.0).await
    }

    /// Writes to a buffer.
    #[inline]
    pub async fn write<'buf, U>(&self, buf: &'buf mut Buffer<U>) -> Result<Write<'buf>, WriteFailed>
    where
        U: u::Write,
    {
        buf.write(&self.0).await
    }

    /// Runs a closure that schedules GPU work.
    ///
    /// The closure receives a [scheduler](Scheduler) object capable of scheduling various GPU operations,
    /// such as rendering, compute, or data copying. All scheduled operations will begin executing
    /// as soon as possible. This function is asynchronous and awaiting it will wait until
    /// all scheduled operations have completed.
    ///
    /// # Examples
    ///
    /// Typical window render loop:
    ///
    /// ```
    /// # struct Window;
    /// # impl Window {
    /// #     async fn redraw(&self) -> Redraw { Redraw }
    /// # }
    /// #
    /// # struct Redraw;
    /// # impl Redraw {
    /// #     fn present(&self) {}
    /// # }
    /// # impl dunge::AsTarget for Redraw {
    /// #     fn as_target(&self) -> dunge::Target<'_> { unreachable!() }
    /// # }
    /// #
    /// # async fn f<V>(
    /// #    window: Window,
    /// #    layer: dunge::Layer<dunge::render::Input<V, (), ()>>,
    /// #    mesh: dunge::mesh::Mesh<V>,
    /// # ) -> Result<(), dunge::FailedMakeContext> {
    /// use dunge::Options;
    ///
    /// let cx = dunge::context().await?;
    /// # #[cfg(false)]
    /// # {
    /// let (window, layer, mesh) = ..
    /// # ;
    /// # }
    ///
    /// loop {
    ///     let redraw = window.redraw().await;
    ///
    ///     cx.shed(|s| {
    ///         let opts = Options::default();
    ///         s.render(&redraw, opts).layer(&layer).draw(&mesh);
    ///     })
    ///     .await;
    ///
    ///     redraw.present();
    /// }
    /// # }
    /// ```
    ///
    /// See the [`render`](Scheduler::render) function for more details.
    #[inline]
    pub async fn shed<F>(&self, f: F)
    where
        F: FnOnce(&mut Scheduler),
    {
        self.0.run(f).await;
    }

    pub fn update_group<S, G, const N: usize>(
        &self,
        set: &mut UniqueSet<S>,
        handler: &GroupHandler<G::Inner, N>,
        group: G,
    ) where
        S: Nth<N, Output = G::Inner>,
        G: Group,
    {
        set::update(&self.0, set, handler, group);
    }

    pub fn wgpu_info(&self) -> wgpu::AdapterInfo {
        self.0.wgpu_info()
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Context").field(&"..").finish()
    }
}

/// An error returned from the [context](Context) constructor.
#[derive(Debug)]
pub enum FailedMakeContext {
    BackendSelection(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for FailedMakeContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BackendSelection(e) => write!(f, "failed to select backend: {e}"),
            Self::RequestDevice(e) => write!(f, "failed to get device: {e}"),
        }
    }
}

impl error::Error for FailedMakeContext {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BackendSelection(e) => Some(e),
            Self::RequestDevice(e) => Some(e),
        }
    }
}
