use {
    crate::{
        camera::{IntoProjection, View},
        canvas::CanvasEvent,
        color::IntoLinear,
        layout::Vertex,
        mesh::MeshData,
        render::{InstanceHandle, MeshHandle, Render, TextureHandle},
        size::Size,
        texture::{FrameFilter, TextureData},
        transform::{IntoQuat, IntoTransform},
        Error,
    },
    winit::{event_loop::EventLoopProxy, window::Window},
};

/// The application context.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) proxy: EventLoopProxy<CanvasEvent>,
    pub(crate) render: Render,
    pub(crate) limits: Limits,
}

impl Context {
    /// Returns the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Plans the main loop to close.
    ///
    /// Calling this function dosn't guarantee closing.
    /// It triggers the [`close_requested`] function in the [`Loop`],
    /// which can handle the closing event.
    ///
    /// [`Loop`]: crate::Loop
    /// [`close_requested`]: crate::Loop::close_requested
    pub fn plan_to_close(&self) {
        _ = self.proxy.send_event(CanvasEvent::Close);
    }

    /// Returns the canvas size.
    pub fn size(&self) -> (u32, u32) {
        self.render.size().as_virtual()
    }

    /// Sets context's [`Limits`].
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Sets context's frame parameters via [`FrameParameters`] struct.
    ///
    /// No effect if `pixel_size` in [`FrameParameters`] is 0.
    pub fn set_frame_parameters(&mut self, params: FrameParameters) {
        if params.pixel_size == 0 {
            return;
        }

        self.render.resize(Some(Size {
            pixel_size: params.pixel_size.try_into().expect("non zero"),
            filter: params.filter,
            ..self.render.size()
        }));
    }

    /// Creates a new texture.
    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        self.render.create_texture(data)
    }

    /// Updates a texture.
    pub fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        self.render.update_texture(handle, data)
    }

    /// Deletes the texture.
    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.render.delete_texture(handle)
    }

    /// Creates new instances.
    pub fn create_instances<I>(&mut self, data: I) -> InstanceHandle
    where
        I: IntoIterator,
        I::Item: IntoTransform,
        <I::Item as IntoTransform>::IntoQuat: IntoQuat,
    {
        let models: Vec<_> = data
            .into_iter()
            .map(|transform| transform.into_transform().into_model())
            .collect();

        self.render.create_instances(&models)
    }

    /// Updates instances.
    pub fn update_instances<I>(&mut self, handle: InstanceHandle, data: I) -> Result<(), Error>
    where
        I: IntoIterator,
        I::Item: IntoTransform,
        <I::Item as IntoTransform>::IntoQuat: IntoQuat,
    {
        let models: Vec<_> = data
            .into_iter()
            .map(|transform| transform.into_transform().into_model())
            .collect();

        self.render.update_instances(handle, &models)
    }

    /// Deletes instances.
    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        self.render.delete_instances(handle)
    }

    /// Creates a new mesh.
    pub fn create_mesh<V>(&mut self, data: MeshData<V>) -> MeshHandle
    where
        V: Vertex,
    {
        self.render.create_mesh(data)
    }

    /// Updates a mesh.
    pub fn update_mesh<V>(&mut self, handle: MeshHandle, data: MeshData<V>) -> Result<(), Error>
    where
        V: Vertex,
    {
        self.render.update_mesh(handle, data)
    }

    /// Deletes the mesh.
    pub fn delete_mesh(&mut self, handle: MeshHandle) -> Result<(), Error> {
        self.render.delete_mesh(handle)
    }

    /// Sets the clear color.
    ///
    /// A new frame will be filled by this color.
    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: IntoLinear,
    {
        self.render.set_clear_color(color.into_linear());
    }

    /// Sets the view.
    pub fn set_view<P>(&mut self, view: View<P>)
    where
        P: IntoProjection,
    {
        self.render.set_view(view.into_projection_view());
    }
}

/// The context's limits.
#[derive(Clone, Copy, Default)]
pub struct Limits {
    pub min_frame_delta_time: Option<f32>,
}

/// Describes frame parameters.
#[derive(Clone, Copy)]
pub struct FrameParameters {
    // Virtual pixels size in physical pixels.
    pub pixel_size: u8,

    // The frame filter mode.
    pub filter: FrameFilter,
}

impl Default for FrameParameters {
    fn default() -> Self {
        Self {
            pixel_size: 1,
            filter: FrameFilter::Nearest,
        }
    }
}
