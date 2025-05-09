use {
    crate::{
        Vertex,
        buffer::{
            Buffer, BufferData, Filter, Read, ReadFailed, Sampler, Texture, Texture2d, TextureData,
            Write, WriteFailed,
        },
        instance::Row,
        layer::{Config, Layer},
        mesh::{self, Mesh},
        render::Input,
        set::{self, Data, GroupHandler, UniqueSet, Visit},
        shader::{ComputeShader, RenderShader, Shader},
        sl,
        state::{Scheduler, State},
        storage::{Storage, StorageValue, Uniform},
        usage::u,
        value::Value,
        workload::Workload,
    },
    std::{error, fmt, sync::Arc},
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
#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    #[inline]
    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    #[inline]
    pub fn make_shader<M, A, K>(&self, module: M) -> Shader<M::Input, M::Set>
    where
        M: sl::IntoModule<A, K>,
    {
        Shader::new(&self.0, module)
    }

    #[inline]
    pub fn make_set<K, S, D>(&self, shader: &Shader<K, S>, set: D) -> UniqueSet<S>
    where
        D: Data<Set = S>,
    {
        UniqueSet::new(&self.0, shader.data(), set)
    }

    #[inline]
    pub fn make_uniform<V>(&self, val: &V) -> Uniform<V>
    where
        V: StorageValue + ?Sized,
    {
        Uniform::new(self, val)
    }

    #[inline]
    pub fn make_storage<V>(&self, val: &V) -> Storage<V>
    where
        V: StorageValue + ?Sized,
    {
        Storage::new(self, val)
    }

    #[inline]
    pub fn make_layer<V, I, S, C>(
        &self,
        shader: &RenderShader<V, I, S>,
        conf: C,
    ) -> Layer<Input<V, I, S>>
    where
        C: Into<Config>,
    {
        let conf = conf.into();
        Layer::new(&self.0, shader.data(), &conf)
    }

    #[inline]
    pub fn make_workload<S>(&self, shader: &ComputeShader<S>) -> Workload {
        Workload::new(&self.0, shader.data())
    }

    #[inline]
    pub fn make_mesh<V>(&self, data: &mesh::MeshData<'_, V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
    }

    #[inline]
    pub fn make_row<U>(&self, data: &[U]) -> Row<U>
    where
        U: Value + bytemuck::NoUninit,
    {
        Row::new(&self.0, data)
    }

    #[inline]
    pub fn make_texture<U>(&self, data: TextureData<'_, U>) -> Texture2d<U>
    where
        U: u::TextureUsages,
    {
        Texture::new(&self.0, data)
    }

    #[inline]
    pub fn make_sampler(&self, filter: Filter) -> Sampler {
        Sampler::new(&self.0, filter)
    }

    #[inline]
    pub fn make_buffer<U>(&self, data: BufferData<'_, U>) -> Buffer<U>
    where
        U: u::BufferUsages,
    {
        Buffer::new(&self.0, data)
    }

    #[inline]
    pub async fn read<'buf, U>(&self, buf: &'buf mut Buffer<U>) -> Result<Read<'buf>, ReadFailed>
    where
        U: u::Read,
    {
        buf.read(&self.0).await
    }

    #[inline]
    pub async fn write<'buf, U>(&self, buf: &'buf mut Buffer<U>) -> Result<Write<'buf>, WriteFailed>
    where
        U: u::Write,
    {
        buf.write(&self.0).await
    }

    #[inline]
    pub fn update_group<S, G>(
        &self,
        set: &mut UniqueSet<S>,
        handler: &GroupHandler<S, G::Projection>,
        group: G,
    ) where
        G: Visit,
    {
        set::update(&self.0, set, handler, group);
    }

    #[inline]
    pub async fn shed<F>(&self, f: F)
    where
        F: FnOnce(Scheduler<'_>),
    {
        self.0.run(f).await;
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
