use {
    crate::{
        draw::Draw,
        group::{self, Binder, Group, GroupHandler, UniqueGroupBinding, Update},
        layer::Layer,
        mesh::{self, Mesh},
        shader::Shader,
        sl::IntoModule,
        state::{Render, State},
        texture::{
            self, CopyBuffer, CopyBufferView, DrawTexture, Format, Make, MapResult, Mapped, Sampler,
        },
        vertex::Vertex,
    },
    std::{future::IntoFuture, sync::Arc},
};

#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    pub async fn new() -> Result<Self, Error> {
        use wgpu::{Backends, Instance, InstanceDescriptor};

        let desc = InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        };

        let instance = Instance::new(desc);
        let state = State::new(&instance).await?;
        Ok(Self(Arc::new(state)))
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

    pub fn make_sampler(&self) -> Sampler {
        Sampler::new(&self.0)
    }

    pub fn make_copy_buffer(&self, size: (u32, u32)) -> CopyBuffer {
        CopyBuffer::new(&self.0, size)
    }

    pub async fn map_view<'a, S, R, F>(&self, view: CopyBufferView<'a>, chan: (S, R)) -> Mapped<'a>
    where
        S: FnOnce(MapResult) + wgpu::WasmNotSend + 'static,
        R: FnOnce() -> F,
        F: IntoFuture<Output = MapResult>,
    {
        let (tx, rx) = chan;
        view.map(&self.0, tx, rx).await
    }

    pub fn draw_to_texture<T, D>(&self, render: &mut Render, texture: &T, draw: D)
    where
        T: DrawTexture,
        D: Draw,
    {
        let view = texture.draw_texture();
        self.0.draw(render, view, draw)
    }

    pub fn update_group<G>(
        &self,
        uni: &mut UniqueGroupBinding,
        handler: GroupHandler<G>,
        group: &G,
    ) -> Update
    where
        G: Group,
    {
        group::update(&self.0, uni, handler, group)
    }
}

/// An error returned from the [`Context`] constructor.
#[derive(Debug)]
pub enum Error {
    BackendSelection,
    RequestDevice,
}
