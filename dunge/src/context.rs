use {
    crate::{
        camera::{IntoProjection, View},
        canvas::CanvasEvent,
        error::*,
        framebuffer::FrameFilter,
        handles::*,
        instance::InstanceModel,
        mesh::Data as MeshData,
        pipeline::ParametersBuilder,
        render::Render,
        screen::Screen,
        shader_data::{Source, SourceModel, Space, SpaceData, SpaceModel, TextureData},
        topology::Topology,
        transform::IntoMat,
        vertex::Vertex,
    },
    winit::{event_loop::EventLoopProxy, window::Window},
};

type Proxy = EventLoopProxy<CanvasEvent>;

/// The application context.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) proxy: Proxy,
    pub(crate) render: Box<Render>,
    limits: Limits,
    models: Vec<InstanceModel>,
    sources: Vec<SourceModel>,
    spaces: Vec<SpaceModel>,
    space_data: Vec<SpaceData<'static>>,
}

impl Context {
    pub(crate) fn new(window: Window, proxy: Proxy, render: Render) -> Self {
        const DEFAULT_CAPACITY: usize = 8;

        Self {
            window,
            proxy,
            render: Box::new(render),
            limits: Limits::default(),
            models: Vec::with_capacity(DEFAULT_CAPACITY),
            sources: Vec::with_capacity(DEFAULT_CAPACITY),
            spaces: Vec::with_capacity(DEFAULT_CAPACITY),
            space_data: Vec::with_capacity(DEFAULT_CAPACITY),
        }
    }

    pub(crate) fn min_frame_delta_time(&self) -> Option<f32> {
        self.limits.min_frame_delta_time
    }

    /// Returns the window.
    #[must_use]
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
    #[must_use]
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
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<(), ResourceNotFound> {
        self.render.delete_layer(handle)
    }

    /// Creates a new texture.
    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        self.render.create_texture(data)
    }

    /// Updates the texture.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), ResourceNotFound> {
        self.render.update_texture(handle, data)
    }

    /// Deletes the texture.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), ResourceNotFound> {
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
    /// See [`ResourceNotFound`] for detailed info.
    pub fn update_instances<I>(
        &mut self,
        handle: InstanceHandle,
        data: I,
    ) -> Result<(), ResourceNotFound>
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
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), ResourceNotFound> {
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
    /// See [`ResourceNotFound`] for detailed info.
    pub fn update_mesh<V, T>(
        &mut self,
        handle: MeshHandle<V, T>,
        data: &MeshData<V, T>,
    ) -> Result<(), ResourceNotFound>
    where
        V: Vertex,
        T: Topology,
    {
        self.render.update_mesh(handle, data)
    }

    /// Deletes the mesh.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), ResourceNotFound> {
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
    /// See [`ResourceNotFound`] for detailed info.
    pub fn update_view<P>(
        &mut self,
        handle: ViewHandle,
        view: View<P>,
    ) -> Result<(), ResourceNotFound>
    where
        P: IntoProjection,
    {
        self.render.update_view(handle, view.into_projection_view())
    }

    /// Deletes the view.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<(), ResourceNotFound> {
        self.render.delete_view(handle)
    }

    /// Creates new light.
    ///
    /// # Errors
    /// Returns the [`TooManySources`] when trying to create too many light sources.
    pub fn create_light<I>(
        &mut self,
        ambient: [f32; 3],
        srcs: I,
    ) -> Result<LightHandle, TooManySources>
    where
        I: IntoIterator<Item = Source>,
    {
        self.sources.clear();
        let models = srcs.into_iter().map(SourceModel::new);
        self.sources.extend(models);
        self.render.create_light(ambient, &self.sources)
    }

    /// Updates the color of the ambient light.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn update_ambient(
        &mut self,
        handle: LightHandle,
        ambient: [f32; 3],
    ) -> Result<(), ResourceNotFound> {
        self.render.update_ambient(handle, ambient)
    }

    /// Updates the light.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_light<I>(
        &mut self,
        handle: LightHandle,
        ambient: [f32; 3],
        srcs: I,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = Source>,
    {
        self.sources.clear();
        let models = srcs.into_iter().map(SourceModel::new);
        self.sources.extend(models);
        self.render.update_light(handle, ambient, &self.sources)
    }

    /// Updates nth source in the light.
    ///
    /// To update all sources at once, call the [`update_light`](crate::Context::update_light) method.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_nth_light(
        &mut self,
        handle: LightHandle,
        n: usize,
        src: Source,
    ) -> Result<(), Error> {
        self.render
            .update_nth_light(handle, n, SourceModel::new(src))
    }

    /// Deletes the light.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_light(&mut self, handle: LightHandle) -> Result<(), ResourceNotFound> {
        self.render.delete_light(handle)
    }

    /// Creates new light space.
    ///
    /// # Errors
    /// Returns the [`TooManySpaces`] when trying to create too many light sources.
    pub fn create_space<'a, I, M>(&mut self, spaces: I) -> Result<SpaceHandle, TooManySpaces>
    where
        I: IntoIterator<Item = Space<'a, M>>,
        M: IntoMat,
    {
        use std::mem;

        self.spaces.clear();
        debug_assert!(self.space_data.is_empty(), "`space_data` is already empty");

        let mut space_data = mem::take(&mut self.space_data);
        for space in spaces {
            space_data.push(space.data);
            self.spaces.push(SpaceModel::new(&space.into_mat()));
        }

        let space = self.render.create_space(&self.spaces, &space_data);

        space_data.clear();
        self.space_data = space_data.into_iter().map(|_| unreachable!()).collect();

        space
    }

    /// Updates the light space.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_space<'a, I, M>(&mut self, handle: SpaceHandle, spaces: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Space<'a, M>>,
        M: IntoMat,
    {
        use std::mem;

        self.spaces.clear();
        debug_assert!(self.space_data.is_empty(), "`space_data` is already empty");

        let mut space_data = mem::take(&mut self.space_data);
        for space in spaces {
            space_data.push(space.data);
            self.spaces.push(SpaceModel::new(&space.into_mat()));
        }

        let updated = self.render.update_space(handle, &self.spaces, &space_data);

        space_data.clear();
        self.space_data = space_data.into_iter().map(|_| unreachable!()).collect();

        updated
    }

    /// Updates nth space in the light space.
    ///
    /// To update all spaces at once, call the [`update_space`](crate::Context::update_space) method.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_nth_space<M>(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        space: Space<M>,
    ) -> Result<(), Error>
    where
        M: IntoMat,
    {
        self.render
            .update_nth_space(handle, n, SpaceModel::new(&space.into_mat()))
    }

    /// Updates nth color in the light space.
    ///
    /// To update all spaces at once, call the [`update_space`](crate::Context::update_space) method.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_nth_space_color(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        col: [f32; 3],
    ) -> Result<(), Error> {
        self.render.update_nth_space_color(handle, n, col)
    }

    /// Updates nth data in the light space.
    ///
    /// To update all spaces at once, call the [`update_space`](crate::Context::update_space) method.
    ///
    /// # Errors
    /// See [`Error`] for detailed info.
    pub fn update_nth_space_data(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        data: SpaceData,
    ) -> Result<(), Error> {
        self.render.update_nth_space_data(handle, n, data)
    }

    /// Deletes the light space.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_space(&mut self, handle: SpaceHandle) -> Result<(), ResourceNotFound> {
        self.render.delete_space(handle)
    }

    /// Takes a screenshot of the current frame.
    ///
    /// If the buffer cannot be copied for some reason,
    /// this method returns an empty.
    #[must_use]
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
