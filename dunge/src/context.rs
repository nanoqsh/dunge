use {
    crate::{
        bind::{self, Binder, ForeignShader, GroupHandler, UniqueBinding, Visit},
        draw::Draw,
        instance::Row,
        layer::{Config, Layer},
        mesh::{self, Mesh},
        shader::Shader,
        sl::IntoModule,
        state::{AsTarget, State},
        texture::{self, CopyBuffer, CopyBufferView, Filter, Make, MapResult, Mapped, Sampler},
        uniform::{IntoValue, Uniform, Value},
        Vertex,
    },
    std::{error, fmt, future::IntoFuture, sync::Arc},
};

#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    pub(crate) fn new(state: State) -> Self {
        Self(Arc::new(state))
    }

    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    pub fn make_shader<M, A>(&self, module: M) -> Shader<M::Vertex, M::Instance>
    where
        M: IntoModule<A>,
    {
        Shader::new(&self.0, module)
    }

    pub fn make_binder<'a, V, I>(&'a self, shader: &'a Shader<V, I>) -> Binder<'a> {
        Binder::new(&self.0, shader)
    }

    pub fn make_uniform<U>(&self, val: U) -> Uniform<U::Value>
    where
        U: IntoValue,
    {
        let val = val.into_value();
        Uniform::new(&self.0, val.value().as_ref())
    }

    pub fn make_layer<V, I, O>(&self, shader: &Shader<V, I>, opts: O) -> Layer<V, I>
    where
        O: Into<Config>,
    {
        let opts = opts.into();
        Layer::new(&self.0, shader, &opts)
    }

    pub fn make_mesh<V>(&self, data: &mesh::MeshData<V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
    }

    pub fn make_row<U>(&self, data: &[U]) -> Row<U>
    where
        U: Value,
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

    pub async fn map_view<'a, S, R, F>(&self, view: CopyBufferView<'a>, tx: S, rx: R) -> Mapped<'a>
    where
        S: FnOnce(MapResult) + wgpu::WasmNotSend + 'static,
        R: FnOnce() -> F,
        F: IntoFuture<Output = MapResult>,
    {
        view.map(&self.0, tx, rx).await
    }

    pub fn update_group<G>(
        &self,
        uni: &mut UniqueBinding,
        handler: &GroupHandler<G::Projection>,
        group: &G,
    ) -> Result<(), ForeignShader>
    where
        G: Visit,
    {
        bind::update(&self.0, uni, handler, group)
    }

    pub fn draw_to<T, D>(&self, target: &T, draw: D)
    where
        T: AsTarget,
        D: Draw,
    {
        self.0.draw(target.as_target(), draw);
    }
}

/// An error returned from the [`Context`] constructor.
#[derive(Debug)]
pub enum Error {
    BackendSelection,
    RequestDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BackendSelection => write!(f, "failed to select backend"),
            Self::RequestDevice(err) => write!(f, "failed to get device: {err}"),
        }
    }
}

impl error::Error for Error {}
