use {
    crate::{
        bind::{self, Binder, GroupHandler, Set, UniqueBinding, UniqueSet, Visit},
        draw::Draw,
        instance::Row,
        layer::{Config, Layer},
        mesh::{self, Mesh},
        shader::{ComputeShader, RenderShader, Shader},
        sl,
        state::{AsTarget, Scheduler, State},
        storage::{Storage, StorageValue},
        texture::{self, CopyBuffer, CopyBufferView, Filter, Make, MapResult, Mapped, Sampler},
        uniform::Uniform,
        value::Value,
        workload::Workload,
        Vertex,
    },
    std::{error, fmt, future::IntoFuture, sync::Arc},
};

/// Creates the context instance.
///
/// # Errors
/// Returns an error when the context could not be created.
/// See [`FailedMakeContext`] for details.
pub async fn context() -> Result<Context, FailedMakeContext> {
    let state = State::new().await?;
    Ok(Context(Arc::new(state)))
}

/// The main dunge context.
///
/// It can be created via the [`context`](fn@crate::context) function
/// or the [`window`](fn@crate::window) function if you need a window
/// and the `winit` feature is enabled.
#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    pub fn make_shader<M, A, K>(&self, module: M) -> Shader<M::Input, M::Set>
    where
        M: sl::IntoModule<A, K>,
    {
        Shader::new(&self.0, module)
    }

    pub fn make_binder<'a, K, S>(&'a self, shader: &'a Shader<K, S>) -> Binder<'a> {
        Binder::new(&self.0, shader.data())
    }

    pub fn make_set<K, S>(&self, shader: &Shader<K, S>, set: S) -> UniqueSet<S>
    where
        S: Set,
    {
        UniqueSet::new(&self.0, shader.data(), set)
    }

    pub fn make_uniform<V>(&self, val: &V) -> Uniform<V>
    where
        V: Value,
    {
        Uniform::new(&self.0, val.value())
    }

    pub fn make_storage<V>(&self, val: &V) -> Storage<V>
    where
        V: StorageValue + ?Sized,
    {
        Storage::new(&self.0, val.storage_value())
    }

    pub fn make_layer<D, S, O>(&self, shader: &RenderShader<D, S>, opts: O) -> Layer<D, S>
    where
        O: Into<Config>,
    {
        let opts = opts.into();
        Layer::new(&self.0, shader.data(), &opts)
    }

    pub fn make_workload<S>(&self, shader: &ComputeShader<S>) -> Workload {
        Workload::new(&self.0, shader.data())
    }

    pub fn make_mesh<V>(&self, data: &mesh::MeshData<V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
    }

    pub fn make_row<U>(&self, data: &[U]) -> Row<U>
    where
        U: Value + bytemuck::NoUninit,
    {
        Row::new(&self.0, data)
    }

    pub fn make_texture<M>(&self, data: M) -> M::Out
    where
        M: Make,
    {
        texture::make(&self.0, data)
    }

    pub fn make_sampler(&self, filter: Filter) -> Sampler {
        Sampler::new(&self.0, filter)
    }

    pub fn make_copy_buffer(&self, size: (u32, u32)) -> CopyBuffer {
        CopyBuffer::new(&self.0, size)
    }

    pub async fn map_view<'a, S, R>(&self, view: CopyBufferView<'a>, tx: S, rx: R) -> Mapped<'a>
    where
        S: FnOnce(MapResult) + wgpu::WasmNotSend + 'static,
        R: IntoFuture<Output = MapResult>,
    {
        view.map(&self.0, tx, rx).await
    }

    pub fn _update_group<G>(
        &self,
        uni: &mut UniqueBinding,
        handler: &GroupHandler<(), G::Projection>,
        group: &G,
    ) where
        G: Visit,
    {
        bind::_update(&self.0, uni, handler, group);
    }

    pub fn update_group<S, G>(
        &self,
        set: &mut UniqueSet<S>,
        handler: &GroupHandler<S, G::Projection>,
        group: G,
    ) where
        G: Visit,
    {
        bind::update(&self.0, set, handler, group);
    }

    pub fn draw_to<T, D>(&self, target: &T, draw: D)
    where
        T: AsTarget,
        D: Draw,
    {
        let target = target.as_target();
        self.0.draw(target, draw);
    }

    pub async fn shed<F, O>(_f: F)
    where
        F: FnOnce(Scheduler<'_>),
    {
    }
}

/// An error returned from the [context](Context) constructor.
#[derive(Debug)]
pub enum FailedMakeContext {
    BackendSelection,
    RequestDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for FailedMakeContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BackendSelection => write!(f, "failed to select backend"),
            Self::RequestDevice(err) => write!(f, "failed to get device: {err}"),
        }
    }
}

impl error::Error for FailedMakeContext {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BackendSelection => None,
            Self::RequestDevice(err) => Some(err),
        }
    }
}
