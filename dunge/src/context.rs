use {
    crate::{
        canvas::{CanvasEvent, Info},
        layer::Layer,
        mesh::{Data as MeshData, Mesh},
        pipeline::LayerBuilder,
        posteffect::Builder as PostEffectBuilder,
        postproc::FrameFilter,
        render::{Render, State},
        scheme::Scheme,
        screen::Screen,
        shader::Shader,
        shader_data::{
            globals::Builder as GlobalsBuilder, lights::Builder as LightsBuilder,
            spaces::Builder as SpacesBuilder, texture::Texture,
            textures::Builder as TexturesBuilder, Instance, InstanceColor, ModelColor,
            ModelTransform, TextureData,
        },
        topology::Topology,
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
    pub(crate) limits: Limits,
}

impl Context {
    #[allow(clippy::unnecessary_box_returns)]
    pub(crate) fn new(window: Window, proxy: Proxy, state: State) -> Box<Self> {
        Box::new(Self {
            window,
            proxy,
            render: Render::new(state),
            limits: Limits::default(),
        })
    }

    /// Returns the render info.
    #[must_use]
    pub fn info(&self) -> &Info {
        self.render.info()
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
    pub fn set_frame_parameters(&self, params: FrameParameters) {
        _ = self.proxy.send_event(CanvasEvent::SetScreen(Screen {
            pixel_size: params.pixel_size,
            filter: params.filter,
            ..self.render.screen()
        }));
    }

    /// Creates a [globals](crate::Globals) builder.
    pub fn globals_builder(&self) -> GlobalsBuilder {
        GlobalsBuilder::new(&self.render)
    }

    /// Creates a [lights](crate::Lights) builder.
    pub fn lights_builder(&self) -> LightsBuilder {
        LightsBuilder::new(&self.render)
    }

    /// Creates a [spaces](crate::Spaces) builder.
    pub fn spaces_builder(&self) -> SpacesBuilder {
        SpacesBuilder::new(&self.render)
    }

    /// Creates a [textures](crate::Textures) builder.
    pub fn textures_builder(&self) -> TexturesBuilder {
        TexturesBuilder::new(&self.render)
    }

    /// Creates a [post-effect](crate::PostEffect) builder.
    pub fn posteffect_builder(&self) -> PostEffectBuilder {
        PostEffectBuilder::new(&self.render)
    }

    /// Creates a new shader [scheme](Scheme).
    pub fn create_scheme<S>(&self) -> Scheme<S>
    where
        S: Shader,
    {
        Scheme::new()
    }

    /// Creates a new [layer](Layer) with default parameters.
    ///
    /// This is a shortcut for `context.create_layer_with().build(scheme)`
    /// with an automatically generated shader [scheme](Scheme).
    /// Use the [`create_layer_with`](crate::Context::create_layer_with)
    /// function to create a custom layer.
    pub fn create_layer<S, T>(&self) -> Layer<S, T>
    where
        S: Shader,
        T: Topology,
    {
        let scheme = self.create_scheme();
        self.create_layer_with().build(&scheme)
    }

    /// Creates a [layer](Layer) builder with custom parameters.
    pub fn create_layer_with<S, T>(&self) -> LayerBuilder<S, T> {
        LayerBuilder::new(&self.render)
    }

    /// Creates new [instances](Instance).
    pub fn create_instances(&self, models: &[ModelTransform]) -> Instance {
        Instance::new(models, &self.render)
    }

    /// Creates new color [instances](Instance).
    pub fn create_instances_color(&self, models: &[ModelColor]) -> InstanceColor {
        InstanceColor::new(models, &self.render)
    }

    /// Creates a new [mesh](Mesh).
    pub fn create_mesh<V, T>(&self, data: &MeshData<V, T>) -> Mesh<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        Mesh::new(data, &self.render)
    }

    /// Creates a new [texture](Texture).
    pub fn create_texture(&self, data: TextureData) -> Texture {
        Texture::new(data, &self.render)
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
