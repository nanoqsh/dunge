use {
    crate::{
        camera::{IntoProjection, View},
        canvas::CanvasEvent,
        color::Rgb,
        error::*,
        framebuffer::FrameFilter,
        handles::*,
        mesh::Data as MeshData,
        pipeline::ParametersBuilder,
        render::{Render, RenderContext},
        resources::Resources,
        screen::Screen,
        shader::{self, Shader, ShaderInfo},
        shader_data::{
            globals::Builder as GlobalsBuilder, lights::Builder as LightsBuilder,
            spaces::Builder as SpacesBuilder, textures::Builder as TexturesBuilder, InstanceModel,
            Source, SourceUniform, SpaceData, TextureData,
        },
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
    pub(crate) render: Render,
    pub(crate) resources: Resources,
    limits: Limits,
    models: Vec<InstanceModel>,
    sources: Vec<SourceUniform>,
}

impl Context {
    pub(crate) fn new(window: Window, proxy: Proxy, render_context: RenderContext) -> Self {
        const DEFAULT_CAPACITY: usize = 8;

        Self {
            window,
            proxy,
            render: Render::new(render_context),
            resources: Resources::default(),
            limits: Limits::default(),
            models: Vec::with_capacity(DEFAULT_CAPACITY),
            sources: Vec::with_capacity(DEFAULT_CAPACITY),
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
        self.render.screen().virtual_size().into()
    }

    /// Sets context's [`Limits`].
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Sets context's frame parameters via [`FrameParameters`] struct.
    pub fn set_frame_parameters(&mut self, params: FrameParameters) {
        _ = self.proxy.send_event(CanvasEvent::SetScreen(Screen {
            pixel_size: params.pixel_size,
            filter: params.filter,
            ..self.render.screen()
        }));
    }

    pub fn globals_builder(&mut self) -> GlobalsBuilder {
        GlobalsBuilder::new(&mut self.resources, &self.render)
    }

    pub fn update_globals_view<S, P>(
        &mut self,
        handle: GlobalsHandle<S>,
        view: View<P>,
    ) -> Result<(), ResourceNotFound>
    where
        S: Shader,
        P: IntoProjection,
    {
        assert!(ShaderInfo::new::<S>().has_camera, "the shader has no view");

        self.resources
            .update_globals_view(handle, view.into_projection_view())
    }

    pub fn update_globals_ambient<S>(
        &mut self,
        handle: GlobalsHandle<S>,
        col: Rgb,
    ) -> Result<(), ResourceNotFound>
    where
        S: Shader,
    {
        assert!(
            ShaderInfo::new::<S>().has_ambient,
            "the shader has no ambient",
        );

        self.resources
            .update_globals_ambient(&self.render, handle, col.0)
    }

    pub fn textures_builder(&mut self) -> TexturesBuilder {
        TexturesBuilder::new(&mut self.resources, &self.render)
    }

    pub fn update_textures_map<S>(
        &mut self,
        handle: TexturesHandle<S>,
        data: TextureData,
    ) -> Result<(), TexturesError>
    where
        S: Shader,
    {
        assert!(
            ShaderInfo::new::<S>().has_map,
            "the shader has no texture map",
        );

        self.resources
            .update_textures_map(&self.render, handle, data)
    }

    pub fn lights_builder(&mut self) -> LightsBuilder {
        LightsBuilder::new(&mut self.resources, &self.render)
    }

    pub fn update_lights_sources<S, I>(
        &mut self,
        handle: LightsHandle<S>,
        index: usize,
        sources: I,
    ) -> Result<(), SourceError>
    where
        S: Shader,
        I: IntoIterator<Item = Source>,
    {
        assert!(
            ShaderInfo::new::<S>().source_arrays > 0,
            "the shader has no light sources",
        );

        self.sources.clear();
        self.sources
            .extend(sources.into_iter().map(Source::into_uniform));

        self.resources
            .update_lights_sources(&self.render, handle, index, &self.sources)
    }

    pub fn spaces_builder(&mut self) -> SpacesBuilder {
        SpacesBuilder::new(&mut self.resources, &self.render)
    }

    pub fn update_spaces_data<S>(
        &mut self,
        handle: SpacesHandle<S>,
        index: usize,
        data: SpaceData,
    ) -> Result<(), SpaceError>
    where
        S: Shader,
    {
        assert!(
            !ShaderInfo::new::<S>().light_spaces.is_empty(),
            "the shader has no light spaces",
        );

        self.resources
            .update_spaces_data(&self.render, handle, index, data)
    }

    pub fn create_shader<S>(&mut self) -> ShaderHandle<S>
    where
        S: Shader,
    {
        self.resources.create_shader(shader::scheme::<S>())
    }

    /// Creates a new layer with default parameters.
    ///
    /// This is a shortcut for `context.create_layer_with_parameters().build()`.
    /// See [`create_layer_with_parameters`](crate::Context::create_layer_with_parameters) for more info.
    pub fn create_layer<S, T>(
        &mut self,
        shader: ShaderHandle<S>,
    ) -> Result<LayerHandle<S, T>, ResourceNotFound>
    where
        S: Shader,
        T: Topology,
    {
        self.create_layer_with_parameters().build(shader)
    }

    /// Creates a layer [builder](ParametersBuilder) with custom parameters.
    pub fn create_layer_with_parameters<V, T>(&mut self) -> ParametersBuilder<V, T> {
        ParametersBuilder::new(&self.render, &mut self.resources)
    }

    /// Deletes the layer.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<(), ResourceNotFound> {
        self.resources.delete_layer(handle)
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
        self.resources.create_instances(&self.render, &self.models)
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
        self.resources
            .update_instances(&self.render, handle, &self.models)
    }

    /// Deletes instances.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), ResourceNotFound> {
        self.resources.delete_instances(handle)
    }

    /// Creates a new mesh.
    pub fn create_mesh<V, T>(&mut self, data: &MeshData<V, T>) -> MeshHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        self.resources.create_mesh(&self.render, data)
    }

    /// Deletes the mesh.
    ///
    /// # Errors
    /// See [`ResourceNotFound`] for detailed info.
    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), ResourceNotFound> {
        self.resources.delete_mesh(handle)
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
    Antialiasing,
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
