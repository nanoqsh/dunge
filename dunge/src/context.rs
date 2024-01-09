use {
    crate::{
        bind::{self, Binder, GroupHandler, UniqueBinding, Update, Visit},
        draw::Draw,
        layer::Layer,
        mesh::{self, Mesh},
        shader::Shader,
        sl::IntoModule,
        state::{Render, RenderView, State},
        texture::{
            self, CopyBuffer, CopyBufferView, DrawTexture, Filter, Format, Make, MapResult, Mapped,
            Sampler,
        },
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

    #[cfg(feature = "winit")]
    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    pub fn make_shader<M, A>(&self, module: M) -> Shader<M::Vertex>
    where
        M: IntoModule<A>,
    {
        Shader::new(&self.0, module)
    }

    pub fn make_binder<'a, V>(&'a self, shader: &'a Shader<V>) -> Binder<'a> {
        Binder::new(&self.0, shader)
    }

    pub fn make_layer<V>(&self, format: Format, shader: &Shader<V>) -> Layer<V> {
        Layer::new(&self.0, format, shader)
    }

    pub fn make_mesh<V>(&self, data: &mesh::Data<V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
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

    pub fn draw_to_texture<T, D>(&self, render: &mut Render, texture: &T, draw: D)
    where
        T: DrawTexture,
        D: Draw,
    {
        let view = RenderView::from_texture(texture.draw_texture());
        self.0.draw(render, view, draw);
    }

    pub fn update_group<G>(
        &self,
        uni: &mut UniqueBinding,
        handler: &GroupHandler<G>,
        group: &G,
    ) -> Update
    where
        G: Visit,
    {
        bind::update(&self.0, uni, handler, group)
    }
}

/// An error returned from the [`Context`] constructor.
#[derive(Debug)]
pub enum Error {
    BackendSelection,
    RequestDevice,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BackendSelection => write!(f, "failed to select backend"),
            Self::RequestDevice => write!(f, "failed to get device"),
        }
    }
}

impl error::Error for Error {}
