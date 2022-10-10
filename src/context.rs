use {
    crate::{
        camera::{Projection, View},
        color::IntoLinear,
        instance::IntoInstances,
        layout::Layout,
        mesh::MeshData,
        render::{InstanceHandle, MeshHandle, Render, TextureHandle},
        size::Size,
        texture::TextureData,
    },
    std::num::NonZeroU32,
    winit::window::Window,
};

/// The application context.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) render: Render,
}

impl Context {
    /// Returns the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns the canvas size.
    pub fn size(&self) -> Size {
        self.render.size()
    }

    pub fn set_pixel_size(&mut self, pixel_size: u32) {
        let mut size = self.render.size();
        size.pixel_size = NonZeroU32::new(pixel_size.max(1)).expect("non zero");
        self.render.resize(size);
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
    pub fn create_instances<I>(&mut self, data: I) -> InstanceHandle
    where
        I: IntoInstances,
    {
        self.render.create_instances(data.into_instances())
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
        P: Into<Projection>,
    {
        self.render.set_view(view.into_projection());
    }
}
