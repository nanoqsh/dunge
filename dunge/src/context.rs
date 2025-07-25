use {
    crate::{
        Vertex,
        buffer::{
            self, Buffer, Filter, Read, ReadFailed, Sampler, Texture, Texture2d, Write, WriteFailed,
        },
        compute,
        instance::Row,
        layer::{Config, Layer},
        mesh::{self, Mesh},
        render,
        set::{self, Data, GroupHandler, UniqueSet, Visit},
        shader::{ComputeShader, RenderShader, Shader},
        sl,
        state::{Scheduler, State},
        storage::{Storage, Uniform},
        usage::u,
        value::UnsizedValue,
        workload::Workload,
    },
    dunge_shader::group::Group,
    std::{error, fmt, pin::Pin, sync::Arc},
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
/// /* use the context */
/// # Ok(())
/// # }
/// ```
///
/// If you're using the library in windowed mode via the
/// [`dunge_winit`](https://docs.rs/dunge_winit/latest/dunge_winit/index.html) crate, use
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
///     /* use the context */
/// }
/// ```
///
/// # Errors
///
/// The builder returns an error when the context could not be created.
/// See [`FailedMakeContext`] for details.
pub fn context() -> Builder {
    Builder(wgpu::Features::empty())
}

/// The [context](Context) builder.
pub struct Builder(wgpu::Features);

impl Builder {
    /// Enables line polygon mode for current backend.
    #[cfg(not(target_family = "wasm"))]
    pub fn enable_polygon_mode_line(self) -> Self {
        Self(self.0 | wgpu::Features::POLYGON_MODE_LINE)
    }

    /// Enables point polygon mode for current backend.
    #[cfg(not(target_family = "wasm"))]
    pub fn enable_polygon_mode_point(self) -> Self {
        Self(self.0 | wgpu::Features::POLYGON_MODE_POINT)
    }
}

impl IntoFuture for Builder {
    type Output = Result<Context, FailedMakeContext>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let state = State::new(self.0).await?;
            Ok(Context(Arc::new(state)))
        })
    }
}

/// The main dunge context.
#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    #[inline]
    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    /// Creates a [shader](Shader) program from a function.
    ///
    /// The provided function defines the GPU computation, which is then compiled into a shader
    /// for the current backend. There are two types of shaders: render shaders and compute shaders.
    ///
    /// No actual computation is performed inside the function itself (aside from compile-time
    /// from the shader's perspective). Instead, the computation is described declaratively
    /// using functions from the [`sl`](crate::sl) module. For example, if you need to compute
    /// [`sin`](crate::sl::sin), use the corresponding function `let y = sl::sin(x);`. This creates
    /// a lazily evaluated sin expression, which will be compiled later during creation of a shader
    /// object. For more details, see the [`sl`](crate::sl) module.
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
    /// | [`PassVertex`](crate::sl::PassVertex)     | Passes a vertex              | Yes             |
    /// | [`PassInstance`](crate::sl::PassInstance) | Passes an instance           | Yes             |
    /// | [`Pass`](crate::sl::Pass)                 | Passes a vertex and instance | Yes             |
    /// | [`Index`](crate::sl::Index)               | Passes a vertex index        | No              |
    /// | [`Groups`](crate::sl::Groups)             | Passes group data            | No              |
    ///
    /// The return type of a render shader must be the [`Render`](crate::sl::Render) struct.
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
    /// vertex stage to the fragment stage, use the [`fragment`](crate::sl::fragment) function.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     prelude::*,
    ///     glam::Mat4,
    ///     sl::{Groups, PassVertex, Render},
    ///     storage::Uniform,
    /// };
    ///
    /// type Vec4f = [f32; 4];
    ///
    /// // describe a vertex type
    /// #[repr(C)]
    /// #[derive(Vertex)]
    /// struct Vert { pos: Vec4f, col: Vec4f }
    ///
    /// # async fn f() -> Result<(), dunge::FailedMakeContext> {
    /// // pass the vertex and a bound 4x4 matrix in the shader
    /// let program = |PassVertex(v): PassVertex<Vert>, Groups(m): Groups<Uniform<Mat4>>| Render {
    ///     // multiply the matrix and the vertex `pos` field  
    ///     place: m * v.pos,
    ///
    ///     // pass `col` from the vertex to fragment stage and return as a final pixel color
    ///     color: sl::fragment(v.col),
    /// };
    ///
    /// let cx = dunge::context().await?;
    /// let shader = cx.make_shader(program);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Compute shader
    ///
    /// Compute shaders can accept the following input types:
    ///
    /// | Type                                      | Semantics in shader         |
    /// | :---------------------------------------- | :-------------------------- |
    /// | [`Invocation`](crate::sl::Invocation)     | Passes an invocation vector |
    /// | [`Groups`](crate::sl::Groups)             | Passes group data           |
    ///
    /// The return type of a compute shader must be the [`Compute`](crate::sl::Compute) struct.
    /// This struct requires two things: an expression in the `compute` field, which will be executed
    /// on each shader invocation, and the workgroup size specified in the `workgroup_size` field.
    /// The `compute` expression can have any type, but its value is not used in the final result.
    /// Instead, the expression is expected to produce side effects - for example,
    /// writing output data to a buffer that can be [read](Context::read) later.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     prelude::*,
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::RwStorage,
    /// };
    ///
    /// // describe an input/output storage array
    /// type Array = RwStorage<[u32; 64]>;
    ///
    /// # async fn f() -> Result<(), dunge::FailedMakeContext> {
    /// // pass an invocation vector and a bound storage in the shader
    /// let program = |Invocation(v): Invocation, Groups(a): Groups<Array>| Compute {
    ///     // read values from the array and rewrite in back
    ///     compute: a.store(v.x(), v.x()),
    ///
    ///     // set the workgroup size
    ///     workgroup_size: [16, 1, 1],
    /// };
    ///
    /// let cx = dunge::context().await?;
    /// let shader = cx.make_shader(program);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn make_shader<M, A, K>(&self, module: M) -> Shader<M::Input, M::Set>
    where
        M: sl::IntoModule<A, K>,
    {
        Shader::new(&self.0, module)
    }

    /// Creates a [set](UniqueSet) of data for the shader.
    ///
    /// A set is a collection of associated data that you can [bind](crate::set::Bind::bind) during
    /// [render](Scheduler::render) or [compute](Scheduler::compute) operations and access from within the shader.
    /// A set can be created from any value that implements the [`Group`] trait, or from a tuple of such types.
    /// You can also derive an implementation of [`Group`](derive@crate::Group) for your custom types.
    ///
    /// # Examples
    ///
    /// For example, here is a shader that fills each fragment with a color passed to it
    ///
    /// ```
    /// use dunge::{
    ///     prelude::*,
    ///     glam::Vec4,
    ///     sl::{Groups, PassVertex, Render},
    ///     storage::Uniform,
    /// };
    ///
    /// type Vec4f = [f32; 4];
    ///
    /// # async fn f(
    /// #     target: dunge::buffer::Texture2d,
    /// #     opts: dunge::Options,
    /// #     layer: dunge::Layer<dunge::render::Input<Vec4f, (), (sl::Ret<sl::Global, dunge::types::Vec4<f32>>,)>>,
    /// #     mesh: dunge::mesh::Mesh<Vec4f>,
    /// # ) -> Result<(), dunge::FailedMakeContext> {
    /// // pass the color value via a uniform
    /// let filler = |PassVertex(v): PassVertex<Vec4f>, Groups(color): Groups<Uniform<Vec4>>| Render {
    ///     // set vertex coordinates
    ///     place: v,
    ///     // pass color from the vertex stage to the fragment stage
    ///     color: sl::fragment(color),
    /// };
    ///
    /// // create the context and shader
    /// let cx = dunge::context().await?;
    /// let shader = cx.make_shader(filler);
    ///
    /// // create a color uniform in RGBA format - for example, red.
    /// let color_uniform = cx.make_uniform(&Vec4::new(1., 0., 0., 1.));
    ///
    /// // create the set value from the uniform
    /// let set = cx.make_set(&shader, color_uniform);
    ///
    /// // now you can bind this set on a render operation
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
    #[inline]
    pub fn make_set<K, S, D>(&self, shader: &Shader<K, S>, set: D) -> UniqueSet<S>
    where
        D: Data<Set = S>,
    {
        UniqueSet::new(&self.0, shader.data(), set)
    }

    /// Creates a [uniform](Uniform) from the given value.
    #[inline]
    pub fn make_uniform<V>(&self, val: &V) -> Uniform<V>
    where
        V: UnsizedValue + ?Sized,
    {
        Uniform::new(self, val)
    }

    /// Creates a [storage](Storage) from the given value.
    #[inline]
    pub fn make_storage<V>(&self, val: &V) -> Storage<V>
    where
        V: UnsizedValue + ?Sized,
    {
        Storage::new(self, val)
    }

    /// Creates a [layer](Layer) for the given [render shader](RenderShader).
    ///
    /// This method also accepts a [config](Config) which defines the layer's properties.
    #[inline]
    pub fn make_layer<V, I, S, C>(
        &self,
        shader: &RenderShader<V, I, S>,
        conf: C,
    ) -> Layer<render::Input<V, I, S>>
    where
        C: Into<Config>,
    {
        let conf = conf.into();
        Layer::new(&self.0, shader.data(), conf)
    }

    /// Creates a [workload](Workload) for the given [compute shader](ComputeShader).
    #[inline]
    pub fn make_workload<S>(&self, shader: &ComputeShader<S>) -> Workload<compute::Input<S>> {
        Workload::new(&self.0, shader.data())
    }

    /// Creates a [mesh](Mesh) with the given [data](mesh::MeshData).
    #[inline]
    pub fn make_mesh<V>(&self, data: &mesh::MeshData<'_, V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
    }

    /// Creates a [row](Row) with the given data.
    #[inline]
    pub fn make_row<U>(&self, data: &[U]) -> Row<U>
    where
        [U]: UnsizedValue,
    {
        Row::new(&self.0, data)
    }

    /// Creates a [2D texture](Texture2d) with the given [data](buffer::TextureData).
    #[inline]
    pub fn make_texture<U>(&self, data: buffer::TextureData<'_, U>) -> Texture2d<U>
    where
        U: u::TextureUsages,
    {
        Texture::new(&self.0, data)
    }

    /// Creates a [sampler](Sampler) with the [filter](Filter) value.
    #[inline]
    pub fn make_sampler(&self, filter: Filter) -> Sampler {
        Sampler::new(&self.0, filter)
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
    /// // create a buffer filled with four `i32` numbers
    /// let data = BufferData::new(&[1, 2, 3, 4])
    ///     .read()     // set a usage to read from the buffer
    ///     .copy_to(); // set a usage to copy to the buffer
    ///
    /// let buffer = cx.make_buffer(data);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
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

    #[inline]
    pub fn update_group<S, G>(
        &self,
        set: &mut UniqueSet<S>,
        handler: &GroupHandler<S, G::Projection>,
        group: G,
    ) where
        G: Visit + Group,
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
