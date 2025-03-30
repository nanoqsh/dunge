use {
    crate::{
        bind::{self, Binder, ForeignShader, GroupHandler, UniqueBinding, Visit},
        draw::Draw,
        instance::Row,
        layer::{Config, Layer},
        mesh::{self, Mesh},
        shader::{RenderShader, Shader},
        sl,
        state::{AsTarget, State},
        storage::Storage,
        texture::{self, CopyBuffer, CopyBufferView, Filter, Make, MapResult, Mapped, Sampler},
        uniform::Uniform,
        value::{IntoValue, Value},
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
    Context::new().await
}

/// The main dunge context.
///
/// It can be created via the [`context`](fn@crate::context) function
/// or the [`window`](fn@crate::window) function if you need a window
/// and the `winit` feature is enabled.
#[derive(Clone)]
pub struct Context(Arc<State>);

impl Context {
    pub(crate) async fn new() -> Result<Self, FailedMakeContext> {
        use wgpu::{Backends, Instance, InstanceDescriptor, InstanceFlags};

        let backends;

        #[cfg(all(
            any(target_family = "unix", target_family = "windows"),
            not(target_os = "macos")
        ))]
        {
            backends = Backends::VULKAN;
        }

        #[cfg(target_os = "macos")]
        {
            backends = Backends::METAL;
        }

        #[cfg(target_family = "wasm")]
        {
            backends = Backends::BROWSER_WEBGPU;
        }

        let instance = {
            let desc = InstanceDescriptor {
                backends,
                flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                ..Default::default()
            };

            Instance::new(&desc)
        };

        let state = State::new(instance).await?;
        Ok(Self(Arc::new(state)))
    }

    pub(crate) fn state(&self) -> &State {
        &self.0
    }

    pub fn make_shader<M, A, K>(&self, module: M) -> Shader<M::Input>
    where
        M: sl::IntoModule<A, K>,
    {
        Shader::new(&self.0, module)
    }

    pub fn make_binder<'a, K>(&'a self, shader: &'a Shader<K>) -> Binder<'a> {
        Binder::new(&self.0, shader.data())
    }

    pub fn make_uniform<U>(&self, val: U) -> Uniform<U::Value>
    where
        U: IntoValue,
    {
        let val = val.into_value();
        Uniform::new(&self.0, val.value().as_ref())
    }

    pub fn make_storage<U>(&self, data: &[U]) -> Storage<U>
    where
        // TODO: remove NoUninit
        U: Value + bytemuck::NoUninit,
    {
        Storage::new(&self.0, data)
    }

    pub fn make_layer<V, I, O>(&self, shader: &RenderShader<V, I>, opts: O) -> Layer<V, I>
    where
        O: Into<Config>,
    {
        let opts = opts.into();
        Layer::new(&self.0, shader.data(), &opts)
    }

    pub fn make_mesh<V>(&self, data: &mesh::MeshData<V>) -> Mesh<V>
    where
        V: Vertex,
    {
        Mesh::new(&self.0, data)
    }

    pub fn make_row<U>(&self, data: &[U]) -> Row<U>
    where
        // TODO: remove NoUninit
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
        let target = target.as_target();
        self.0.draw(target, draw);
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
