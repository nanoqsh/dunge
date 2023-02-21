use {
    crate::{
        camera::Camera,
        color::{IntoLinear, Linear},
        frame::Frame,
        instance::Instance,
        mesh::Mesh,
        r#loop::Error,
        render::{InstanceHandle, MeshHandle, TextureHandle, ViewHandle},
        shader,
        storage::Storage,
        texture::Texture,
        vertex::{ColorVertex, FlatVertex, TextureVertex, UsesDepth},
    },
    std::marker::PhantomData,
    wgpu::{Queue, RenderPass},
};

/// The layer builder. It creates a configured [`Layer`].
#[must_use]
pub struct Builder<'l, 'd, V> {
    frame: &'l mut Frame<'d>,
    clear_color: Option<Linear<f64>>,
    clear_depth: bool,
    vertex_type: PhantomData<V>,
}

impl<'l, 'd, V> Builder<'l, 'd, V> {
    pub(crate) fn new(frame: &'l mut Frame<'d>) -> Self {
        Self {
            frame,
            clear_color: None,
            clear_depth: false,
            vertex_type: PhantomData,
        }
    }

    /// Sets clear color for the layer.
    ///
    /// It takes a color parameter, which must implement the [`IntoLinear`] trait.
    ///
    /// Don't set this setting if you don't want to fill
    /// the previous layer (or frame) with some color.
    /// Or set to clear the current buffer if a layer is already drawn
    /// into the frame by calling [`commit_in_frame`](crate::Frame::commit_in_frame).
    ///
    /// # Example
    /// ```
    /// # use dunge::color::Srgba;
    /// # struct Frame;
    /// # impl Frame {
    /// #     fn texture_layer(self) -> Self { self }
    /// #     fn with_clear_color(self, _: Srgba<u8>) -> Self { self }
    /// #     fn start(self) {}
    /// # }
    /// # let frame = Frame;
    /// let color = Srgba([20, 30, 40, 255]);
    /// let mut layer = frame
    ///     .texture_layer()
    ///     .with_clear_color(color)
    ///     .start();
    /// ```
    ///
    /// To clear a layer with a transparent color, it is enough to pass `()` as a parameter.
    ///
    /// # Example
    /// ```
    /// # struct Frame;
    /// # impl Frame {
    /// #     fn texture_layer(self) -> Self { self }
    /// #     fn with_clear_color(self, _: ()) -> Self { self }
    /// #     fn start(self) {}
    /// # }
    /// # let frame = Frame;
    /// let mut layer = frame
    ///     .texture_layer()
    ///     .with_clear_color(())
    ///     .start();
    /// ```
    pub fn with_clear_color<C>(self, color: C) -> Self
    where
        C: IntoLinear,
    {
        Self {
            clear_color: Some(color.into_linear()),
            ..self
        }
    }

    /// Sets the flag to clear the depth buffer or not for the layer.
    pub fn with_clear_depth(self) -> Self
    where
        V: UsesDepth,
    {
        Self {
            clear_depth: true,
            ..self
        }
    }
}

impl<'l> Builder<'l, '_, TextureVertex> {
    pub fn start(self) -> Layer<'l, TextureVertex> {
        self.frame.start_layer(self.clear_color, self.clear_depth)
    }
}

impl<'l> Builder<'l, '_, ColorVertex> {
    pub fn start(self) -> Layer<'l, ColorVertex> {
        self.frame.start_layer(self.clear_color, self.clear_depth)
    }
}

impl<'l> Builder<'l, '_, FlatVertex> {
    pub fn start(self) -> Layer<'l, FlatVertex> {
        self.frame.start_layer(self.clear_color, self.clear_depth)
    }
}

/// The frame layer. Can be created from a [`Frame`] instance.
#[must_use]
pub struct Layer<'l, V> {
    pass: RenderPass<'l>,
    size: (u32, u32),
    queue: &'l Queue,
    resources: &'l Resources,
    instance: Option<&'l Instance>,
    vertex_type: PhantomData<V>,
    drawn_in_frame: &'l mut bool,
}

impl<'l, V> Layer<'l, V> {
    pub(crate) fn new(
        pass: RenderPass<'l>,
        size: (u32, u32),
        queue: &'l Queue,
        resources: &'l Resources,
        drawn_in_frame: &'l mut bool,
    ) -> Self {
        Self {
            pass,
            size,
            queue,
            resources,
            instance: None,
            vertex_type: PhantomData,
            drawn_in_frame,
        }
    }

    /// Binds a [instance](crate::InstanceHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given instance handler was deleted.
    pub fn bind_instance(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        let instance = self.resources.instances.get(handle.0)?;
        self.instance = Some(instance);

        Ok(())
    }

    /// Draws nothing.
    pub fn draw_empty(&mut self) {
        *self.drawn_in_frame = true;
    }

    /// Draws a [mesh](crate::MeshHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given mesh handler was deleted.
    /// Returns [`Error::InstanceNotSet`] if no any [instance](InstanceHandle) is set.
    /// Call [`bind_instance`](crate::Layer::bind_instance) to set an instance.
    pub fn draw(&mut self, handle: MeshHandle<V>) -> Result<(), Error> {
        use wgpu::IndexFormat;

        let mesh = self.resources.meshes.get(handle.id())?;
        let instance = self.instance.ok_or(Error::InstanceNotSet)?;

        self.pass
            .set_vertex_buffer(shader::INSTANCE_BUFFER_SLOT, instance.buffer().slice(..));
        self.pass
            .set_vertex_buffer(shader::VERTEX_BUFFER_SLOT, mesh.vertex_buffer().slice(..));
        self.pass
            .set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
        self.pass
            .draw_indexed(0..mesh.n_indices(), 0, 0..instance.n_instances());

        self.draw_empty();
        Ok(())
    }

    fn bind_view_handle(&mut self, handle: ViewHandle, group: u32) -> Result<(), Error> {
        let camera = self.resources.views.get(handle.0)?;
        camera.resize(self.size, self.queue);
        self.pass.set_bind_group(group, camera.bind_group(), &[]);

        Ok(())
    }

    fn bind_texture_handle(&mut self, handle: TextureHandle, group: u32) -> Result<(), Error> {
        let texture = self.resources.textures.get(handle.0)?;
        self.pass.set_bind_group(group, texture.bind_group(), &[]);

        Ok(())
    }
}

impl Layer<'_, TextureVertex> {
    /// Binds a [view](crate::ViewHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given view handler was deleted.
    pub fn bind_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        self.bind_view_handle(handle, shader::TEXTURED_CAMERA_GROUP)
    }
}

impl Layer<'_, ColorVertex> {
    /// Binds a [view](crate::ViewHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given view handler was deleted.
    pub fn bind_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        self.bind_view_handle(handle, shader::COLOR_CAMERA_GROUP)
    }
}

impl Layer<'_, TextureVertex> {
    /// Binds a [texture](crate::TextureHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given texture handler was deleted.
    pub fn bind_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.bind_texture_handle(handle, shader::TEXTURED_SDIFF_GROUP)
    }
}

impl Layer<'_, FlatVertex> {
    /// Binds a [texture](crate::TextureHandle).
    ///
    /// # Errors
    /// Returns [`Error::ResourceNotFound`] if given texture handler was deleted.
    pub fn bind_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.bind_texture_handle(handle, shader::FLAT_SDIFF_GROUP)
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) views: Storage<Camera>,
}
