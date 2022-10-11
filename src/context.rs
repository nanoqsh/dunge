use {
    crate::{
        camera::{IntoProjection, View},
        canvas::CanvasEvent,
        color::IntoLinear,
        instance::{InstanceData, Rotation},
        layout::Layout,
        mesh::MeshData,
        render::{InstanceHandle, MeshHandle, Render, TextureHandle},
        size::Size,
        texture::{FrameFilter, TextureData},
    },
    winit::{event_loop::EventLoopProxy, window::Window},
};

/// The application context.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) proxy: EventLoopProxy<CanvasEvent>,
    pub(crate) render: Render,
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
    pub fn size(&self) -> Size {
        self.render.size()
    }

    /// Sets frame parameters.
    ///
    /// A `pixel_size` sets virtual pixels size in physical pixels.
    /// A `filter` sets the frame filter mode.
    ///
    /// No effect if `pixel_size` is 0.
    pub fn set_frame_parameters(&mut self, pixel_size: u8, filter: FrameFilter) {
        if pixel_size == 0 {
            return;
        }

        self.render.resize(Size {
            pixel_size: pixel_size.try_into().expect("non zero"),
            filter,
            ..self.render.size()
        });
    }

    /// Creates a new texture.
    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        self.render.create_texture(data)
    }

    /// Deletes the texture.
    pub fn delete_texture(&mut self, handle: TextureHandle) {
        self.render.delete_texture(handle);
    }

    /// Creates a new instance.
    pub fn create_instances<I, R>(&mut self, data: I) -> InstanceHandle
    where
        I: IntoIterator<Item = InstanceData<R>>,
        R: Rotation,
    {
        let models = data.into_iter().map(InstanceData::into_model).collect();
        self.render.create_instances(models)
    }

    /// Deletes the instance.
    pub fn delete_instance(&mut self, handle: InstanceHandle) {
        self.render.delete_instance(handle);
    }

    /// Creates a new mesh.
    pub fn create_mesh<V>(&mut self, data: MeshData<V>) -> MeshHandle
    where
        V: Layout,
    {
        self.render.create_mesh(data)
    }

    /// Deletes the mesh.
    pub fn delete_mesh(&mut self, handle: MeshHandle) {
        self.render.delete_mesh(handle);
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
