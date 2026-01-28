use {
    crate::{
        Vertex,
        buffer::{
            self, Buffer, Filter, Read, ReadFailed, TextureBuffer, TextureSampler, Write,
            WriteFailed,
        },
        instance_old::{RowOld, RowValue},
        layer::{Config, Layer},
        mesh::{self, Mesh},
        render,
        set::{self, Data, GroupHandler, Groups, UniqueSet, Visit},
        shader::{RenderShader, RenderShaderOld, Shader},
        sl_old,
        state::{Scheduler, State},
        store::{self, Row, Storage, StorageValue, Uniform},
        store_old::{StorageOld, UniformOld},
        usage::u,
        value::{StorageValue as StorageValueOld, UniformValue},
    },
    dunge_shade::{bytes::Bytes, irc::Value, link::Render},
    dunge_shade_old::group::GroupLegacy,
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

    /// Creates a [shader](Shader) program from a function.
    ///
    /// The provided function defines the GPU computation, which is then compiled into a shader
    /// for the current backend.
    ///
    /// No actual computation is performed inside the function itself (aside from compile-time
    /// from the shader's perspective). Instead, the computation is described declaratively
    /// using functions from the [`sl`](crate::sl_old) module. For example, if you need to compute
    /// [`sin`](crate::sl_old::sin), use the corresponding function `let y = sl_old::sin(x);`. This creates
    /// a lazily evaluated sin expression, which will be compiled later during creation of a shader
    /// object. For more details, see the [`sl`](crate::sl_old) module.
    ///
    /// This function holds static type information of the shader:
    /// * Its input types - vertex and instance types, relevant for render shaders.
    /// * Its bind groups - relevant for all shader types.
    ///
    /// # Render shader
    ///
    /// Render shaders can accept the following input types:
    ///
    /// | Type                                      | Semantics in shader          | Must lead first |
    /// | :---------------------------------------- | :--------------------------- | :-------------- |
    /// | [`PassVertex`](crate::sl_old::PassVertex)     | Passes a vertex              | Yes             |
    /// | [`PassInstance`](crate::sl_old::PassInstance) | Passes an instance           | Yes             |
    /// | [`Pass`](crate::sl_old::Pass)                 | Passes a vertex and instance | Yes             |
    /// | [`Index`](crate::sl_old::Index)               | Passes a vertex index        | No              |
    /// | [`Groups`](crate::sl_old::Groups)             | Passes group data            | No              |
    ///
    /// The return type of a render shader must be the [`Render`](crate::sl_old::Render) struct.
    /// This struct requires two expressions to be set: the final vertex position in the `place` field
    /// and the final fragment (pixel) color in the `color` field.
    /// The vertex position is specified in
    /// [homogeneous coordinates](https://en.wikipedia.org/wiki/Homogeneous_coordinates), so the type
    /// of the `place` expression must be [`Vec4<f32>`](crate::types::Vec4). The fragment color is
    /// specified in RGBA format, so the type of the `color` expression must also be
    /// [`Vec4<f32>`](crate::types::Vec4).
    ///
    /// A render shader consists of two stages: the vertex stage and the fragment stage,  
    /// but both are described together as a single function. To pass output data from the
    /// vertex stage to the fragment stage, use the [`fragment`](crate::sl_old::fragment) function.
    ///
    /// # Examples
    ///
    /// ```
    /// use {
    ///     dunge::{
    ///         prelude::*,
    ///         sl_old::{Groups, PassVertex, Render},
    ///         store_old::UniformOld,
    ///     },
    ///     glam::Mat4,
    /// };
    ///
    /// type Vec4f = [f32; 4];
    ///
    /// // Describe a vertex type
    /// #[repr(C)]
    /// #[derive(Vertex)]
    /// struct Vert { pos: Vec4f, col: Vec4f }
    ///
    /// # async fn f() -> Result<(), dunge::FailedMakeContext> {
    /// // Pass the vertex and a bound 4x4 matrix in the shader
    /// let program = |PassVertex(v): PassVertex<Vert>, Groups(m): Groups<UniformOld<Mat4>>| Render {
    ///     // Multiply the matrix and the vertex `pos` field  
    ///     place: m.load() * v.pos,
    ///
    ///     // Pass `col` from the vertex to fragment stage and return as a final pixel color
    ///     color: sl_old::fragment(v.col),
    /// };
    ///
    /// let cx = dunge::context().await?;
    /// let shader = cx.make_shader_old(program);
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_shader_old<M, A, K>(&self, module: M) -> Shader<M::Input, M::Set>
    where
        M: sl_old::IntoModule<A, K>,
    {
        Shader::from_module(&self.0, module)
    }

    pub fn make_shader<I, S>(&self, render: Render<I, S>) -> Shader<I, S> {
        Shader::new(&self.0, render)
    }

    /// Creates a [set](UniqueSet) of data for the shader.
    ///
    /// A set is a collection of associated data that you can [bind](crate::set::Bind::bind) during
    /// [render](Scheduler::render) operations and access from within the shader.
    /// A set can be created from any value that implements the [`GroupLegacy`] trait, or from a tuple of such types.
    /// You can also derive an implementation of [`Group`](derive@crate::GroupLegacy) for your custom types.
    ///
    /// # Examples
    ///
    /// For example, here is a shader that fills each fragment with a color passed to it:
    ///
    /// ```
    /// use dunge::{
    ///     prelude::*,
    ///     color::Rgba,
    ///     sl_old::{Groups, PassVertex, Render},
    ///     store_old::UniformOld,
    /// };
    ///
    /// type Vec4f = [f32; 4];
    ///
    /// # async fn f(
    /// #     target: dunge::buffer::TextureBuffer<2>,
    /// #     opts: dunge::Options,
    /// #     layer: dunge::Layer<dunge::render::Input<Vec4f, (), (sl_old::Ret<sl_old::Global, dunge::types::Pointer<dunge::types::Vec4<f32>>>,)>>,
    /// #     mesh: dunge::mesh::Mesh<Vec4f>,
    /// # ) -> Result<(), dunge::FailedMakeContext> {
    /// // Pass the color value via a uniform
    /// let filler = |PassVertex(v): PassVertex<Vec4f>, Groups(color): Groups<UniformOld<Rgba>>| Render {
    ///     // Set vertex coordinates
    ///     place: v,
    ///     // Pass color from the vertex stage to the fragment stage
    ///     color: sl_old::fragment(color.load()),
    /// };
    ///
    /// // Create the context and shader
    /// let cx = dunge::context().await?;
    /// let shader = cx.make_shader_old(filler);
    ///
    /// // Create a color uniform in RGBA format - for example, red.
    /// let color_uniform = cx.make_uniform(&Rgba::from_bytes([!0, 0, 0, !0]));
    ///
    /// // Create the set value from the uniform
    /// let set = cx.make_set(&shader, color_uniform);
    ///
    /// // Now you can bind this set on a render operation
    /// # #[cfg(false)]
    /// # {
    /// let (target, opts, layer, mesh) = ..
    /// # ;
    /// # }
    /// cx.shed(|s| {
    ///     s.render(&target, opts).layer(&layer).set(&set).draw(&mesh);
    ///     //                                         ^^^ bind the set
    /// })
    /// .await;
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_set<I, S, D>(&self, shader: &Shader<I, S>, set: D) -> UniqueSet<S>
    where
        D: Data<Set = S>,
    {
        UniqueSet::from_data(&self.0, shader.data(), set)
    }

    pub fn make_set2<I, G, const N: usize>(
        &self,
        shader: &Shader<I, G::Inner>,
        set: G,
    ) -> UniqueSet<G::Inner>
    where
        G: Groups<N>,
    {
        UniqueSet::new(&self.0, shader.data(), set)
    }

    /// Creates a [uniform](UniformOld) from the given value.
    pub fn make_uniform<V>(&self, value: &V) -> UniformOld<V>
    where
        V: UniformValue,
    {
        UniformOld::new(self, value)
    }

    /// Creates a [uniform](Uniform) from the given value.
    pub fn make_uniform2<V>(&self, value: &V) -> Uniform<V>
    where
        V: Value + Bytes,
    {
        const { assert!(size_of::<V>() > 0, "value cannot be zero sized") }
        store::uniform(value, self)
    }

    /// Creates a [storage](StorageOld) from the given value.
    pub fn make_storage<V>(&self, value: &V) -> StorageOld<V>
    where
        V: StorageValueOld + ?Sized,
    {
        StorageOld::new(self, value)
    }

    /// Creates a [storage](Storage) from the given value.
    pub fn make_storage2<V>(&self, value: &V) -> Option<Storage<V>>
    where
        V: StorageValue + ?Sized,
    {
        store::storage(value, self)
    }

    /// Creates a [layer](Layer) for the given [render shader](RenderShaderOld).
    ///
    /// This method also accepts a [config](Config) which defines the layer's properties.
    pub fn make_layer<V, I, S, C>(
        &self,
        shader: &RenderShaderOld<S, V, I>,
        conf: C,
    ) -> Layer<render::Input<V, I, S>>
    where
        C: Into<Config>,
    {
        let conf = conf.into();
        Layer::new(&self.0, shader.data(), conf)
    }

    /// Creates a [layer](Layer) for the given [render shader](RenderShader).
    ///
    /// This method also accepts a [config](Config) which defines the layer's properties.
    pub fn make_layer2<V, I, S, C>(
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
        V: Vertex,
    {
        Mesh::from_vertex(&self.0, data)
    }

    /// Creates a [mesh](Mesh) with the given [data](mesh::MeshData).
    pub fn make_mesh2<V>(&self, data: &mesh::MeshData<'_, V>) -> Mesh<V>
    where
        V: Bytes,
    {
        Mesh::new(&self.0, data)
    }

    /// Creates a [row](RowOld) with the given data.
    pub fn make_row<V>(&self, data: &[V]) -> RowOld<V>
    where
        V: RowValue,
    {
        RowOld::new(&self.0, data)
    }

    /// Creates a [row](Row) with the given data.
    pub fn make_row2<V>(&self, value: &[V]) -> Option<Row<V>>
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

    pub fn update_group<S, G>(
        &self,
        set: &mut UniqueSet<S>,
        handler: &GroupHandler<S, G::Projection>,
        group: G,
    ) where
        G: Visit + GroupLegacy,
    {
        set::update(&self.0, set, handler, group);
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
