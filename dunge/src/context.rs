use {
    crate::{
        camera::{IntoProjection, View},
        canvas::CanvasEvent,
        handles::*,
        instance::InstanceModel,
        mesh::Data as MeshData,
        pipeline::ParametersBuilder,
        render::Render,
        render_frame::FrameFilter,
        screen::Screen,
        texture::Data as TextureData,
        topology::Topology,
        transform::IntoMat,
        vertex::Vertex,
        Error,
    },
    winit::{event_loop::EventLoopProxy, window::Window},
};

type Proxy = EventLoopProxy<CanvasEvent>;

/// The application context.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) proxy: Proxy,
    pub(crate) render: Render,
    limits: Limits,
    models: Vec<InstanceModel>,
}

impl Context {
    pub(crate) fn new(window: Window, proxy: Proxy, render: Render) -> Self {
        const DEFAULT_MODELS_CAPACITY: usize = 8;

        Self {
            window,
            proxy,
            render,
            limits: Limits::default(),
            models: Vec::with_capacity(DEFAULT_MODELS_CAPACITY),
        }
    }

    pub(crate) fn min_frame_delta_time(&self) -> Option<f32> {
        self.limits.min_frame_delta_time
    }

    /// Returns the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Plans the main loop to close.
    ///
    /// Calling this function dosn't guarantee closing.
    /// It triggers the [`close_requested`](crate::Loop::close_requested)
    /// function in the [`Loop`](crate::Loop), which can handle the closing event.
    pub fn plan_to_close(&self) {
        _ = self.proxy.send_event(CanvasEvent::Close);
    }

    /// Returns the canvas size.
    pub fn size(&self) -> (u32, u32) {
        self.render.screen().virtual_size()
    }

    /// Sets context's [`Limits`].
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Sets context's frame parameters via [`FrameParameters`] struct.
    pub fn set_frame_parameters(&mut self, params: FrameParameters) {
        self.render.set_screen(Some(Screen {
            pixel_size: params.pixel_size,
            filter: params.filter,
            ..self.render.screen()
        }));
    }

    /// Creates a new layer with default parameters.
    ///
    /// This is a shortcut for `context.create_layer_with_parameters().build()`.
    /// See [`create_layer_with_parameters`](crate::Context::create_layer_with_parameters) for more info.
    pub fn create_layer<V, T>(&mut self) -> LayerHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        self.create_layer_with_parameters().build()
    }

    /// Creates a layer [builder](ParametersBuilder) with custom parameters.
    pub fn create_layer_with_parameters<V, T>(&mut self) -> ParametersBuilder<V, T> {
        ParametersBuilder::new(&mut self.render)
    }

    /// Deletes the layer.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn delete_layer<V, T>(&mut self, handle: LayerHandle<V, T>) -> Result<(), Error> {
        self.render.delete_layer(handle)
    }

    /// Creates a new texture.
    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        self.render.create_texture(data)
    }

    /// Updates the texture.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        self.render.update_texture(handle, data)
    }

    /// Deletes the texture.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.render.delete_texture(handle)
    }

    /// Creates new instances.
    pub fn create_instances<I>(&mut self, data: I) -> InstanceHandle
    where
        I: IntoIterator,
        I::Item: IntoMat,
    {
        self.models.clear();
        let models = data
            .into_iter()
            .map(|transform| InstanceModel::from(transform.into_mat()));

        self.models.extend(models);
        self.render.create_instances(&self.models)
    }

    /// Updates instances.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_instances<I>(&mut self, handle: InstanceHandle, data: I) -> Result<(), Error>
    where
        I: IntoIterator,
        I::Item: IntoMat,
    {
        self.models.clear();
        let models = data
            .into_iter()
            .map(|transform| InstanceModel::from(transform.into_mat()));

        self.models.extend(models);
        self.render.update_instances(handle, &self.models)
    }

    /// Deletes instances.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        self.render.delete_instances(handle)
    }

    /// Creates a new mesh.
    pub fn create_mesh<V, T>(&mut self, data: &MeshData<V, T>) -> MeshHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        self.render.create_mesh(data)
    }

    /// Updates the mesh.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_mesh<V, T>(
        &mut self,
        handle: MeshHandle<V, T>,
        data: &MeshData<V, T>,
    ) -> Result<(), Error>
    where
        V: Vertex,
        T: Topology,
    {
        self.render.update_mesh(handle, data)
    }

    /// Deletes the mesh.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), Error> {
        self.render.delete_mesh(handle)
    }

    /// Creates a new view.
    pub fn create_view<P>(&mut self, view: View<P>) -> ViewHandle
    where
        P: IntoProjection,
    {
        self.render.create_view(view.into_projection_view())
    }

    /// Updates the view.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_view<P>(&mut self, handle: ViewHandle, view: View<P>) -> Result<(), Error>
    where
        P: IntoProjection,
    {
        self.render.update_view(handle, view.into_projection_view())
    }

    /// Deletes the view.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        self.render.delete_view(handle)
    }

    /// Takes a screenshot of the current frame.
    ///
    /// If the buffer cannot be copied for some reason,
    /// this method returns an empty.
    pub fn take_screenshot(&self) -> Screenshot {
        self.render.take_screenshot()
    }
}

/// The context's limits.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Sets a minimal time between two frames in seconds.
    ///
    /// If the value is set, then the [context](crate::Context) will draw
    /// a next frame no earlier than the specified time.
    pub min_frame_delta_time: Option<f32>,
}

impl Default for Limits {
    fn default() -> Self {
        const FPS: f32 = 60.;

        Self {
            min_frame_delta_time: Some(1. / FPS),
        }
    }
}

/// Describes frame parameters.
#[derive(Clone, Copy, Default)]
pub struct FrameParameters {
    /// Virtual pixels size in physical pixels.
    pub pixel_size: PixelSize,

    /// The frame filter mode.
    pub filter: FrameFilter,
}

/// Virtual pixels size in physical pixels.
#[derive(Clone, Copy, Default)]
pub enum PixelSize {
    XHalf,
    #[default]
    X1,
    X2,
    X3,
    X4,
}

/// The representation of a screenshot.
pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
