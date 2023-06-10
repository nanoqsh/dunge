use {
    crate::{
        _shader::{self, _Shader},
        _vertex::{ColorVertex, FlatVertex, TextureVertex, _Vertex},
        color::{IntoLinear, Linear},
        error::{Error, ResourceNotFound},
        frame::Frame,
        handles::*,
        mesh::Mesh,
        pipeline::Pipeline,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::Instance,
    },
    std::marker::PhantomData,
    wgpu::{BindGroup, Queue, RenderPass},
};

/// The frame layer. Can be created from a [`Frame`] instance.
#[must_use]
pub struct Layer<'l, S, T> {
    pass: RenderPass<'l>,
    size: (u32, u32),
    queue: &'l Queue,
    resources: &'l Resources,
    drawn_in_frame: &'l mut bool,
    groups: Groups<'l>,
    instance: Option<&'l Instance>,
    vertex_type: PhantomData<(S, T)>,
}

impl<'l, S, T> Layer<'l, S, T> {
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
            drawn_in_frame,
            groups: Groups::default(),
            instance: None,
            vertex_type: PhantomData,
        }
    }

    pub(crate) fn _new(
        pass: RenderPass<'l>,
        size: (u32, u32),
        queue: &'l Queue,
        resources: &'l Resources,
        drawn_in_frame: &'l mut bool,
    ) -> Self
    where
        S: _Vertex,
    {
        let mut layer = Self {
            pass,
            size,
            queue,
            resources,
            drawn_in_frame,
            groups: Groups::default(),
            instance: None,
            vertex_type: PhantomData,
        };

        // Bind default light and set default ambient
        match S::VALUE.into_inner() {
            _Shader::Color => layer
                .bind_light_handle(LightHandle::DEFAULT, _shader::COLOR_SOURCES_GROUP)
                .expect("bind default light"),
            _Shader::Textured => {
                layer
                    .bind_light_handle(LightHandle::DEFAULT, _shader::TEXTURED_SOURCES_GROUP)
                    .expect("bind default light");

                layer
                    .bind_space_handle(SpaceHandle::DEFAULT, _shader::TEXTURED_SPACE_GROUP)
                    .expect("bind default space");
            }
            _ => {}
        }

        layer
    }

    /// Binds the [globals](crate::handles::GlobalsHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given globals handler was deleted.
    pub fn bind_globals(
        &mut self,
        handle: GlobalsHandle<S>,
    ) -> Result<&mut Self, ResourceNotFound> {
        let globals = self.resources.globals.get(handle.id())?;
        globals.write_camera(self.size, self.queue);

        self.groups.globals = Some(globals.bind_group());
        Ok(self)
    }

    /// Binds the [textures](crate::handles::TexturesHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given textures handler was deleted.
    pub fn bind_textures(
        &mut self,
        handle: TexturesHandle<S>,
    ) -> Result<&mut Self, ResourceNotFound> {
        let textures = self.resources.textures.get(handle.id())?;
        self.groups.textures = Some(textures.bind_group());
        Ok(self)
    }

    /// Binds the [instance](crate::handles::InstanceHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given instance handler was deleted.
    pub fn bind_instance(&mut self, handle: InstanceHandle) -> Result<&mut Self, ResourceNotFound> {
        let instance = self.resources.instances.get(handle.0)?;
        self.instance = Some(instance);
        Ok(self)
    }

    /// Draws the [mesh](crate::handles::MeshHandle).
    ///
    /// # Errors
    /// See [`Error`] for details.
    pub fn draw(&mut self, handle: MeshHandle<S::Vertex, T>) -> Result<(), Error>
    where
        S: Shader,
    {
        use dunge_shader::{Globals, Textures};

        let info = ShaderInfo::new::<S>();
        if info.has_globals() {
            let group = self.groups.globals.ok_or(Error::GlobalsNotSet)?;
            self.pass.set_bind_group(Globals::GROUP, group, &[]);
        }

        if info.has_textures() {
            let group = self.groups.textures.ok_or(Error::TexturesNotSet)?;
            self.pass.set_bind_group(Textures::GROUP, group, &[]);
        }

        self.draw_mesh(self.resources.meshes.get(handle.id())?)
    }

    /// Draws the [mesh](crate::handles::MeshHandle).
    ///
    /// # Errors
    /// Returns [`Error::NotFound`] if given mesh handler was deleted.
    /// Returns [`Error::InstanceNotSet`] if no any [instance](InstanceHandle) is set.
    /// Call [`bind_instance`](crate::Layer::bind_instance) to set an instance.
    pub fn _draw(&mut self, handle: MeshHandle<S, T>) -> Result<(), Error> {
        self.draw_mesh(self.resources.meshes.get(handle.id())?)
    }

    fn draw_mesh(&mut self, mesh: &'l Mesh) -> Result<(), Error> {
        use {crate::mesh::Type, wgpu::IndexFormat};

        let instance = self.instance.ok_or(Error::InstanceNotSet)?;

        self.pass
            .set_vertex_buffer(_shader::INSTANCE_BUFFER_SLOT, instance.buffer().slice(..));
        self.pass
            .set_vertex_buffer(_shader::VERTEX_BUFFER_SLOT, mesh.vertex_buffer().slice(..));

        match mesh.mesh_type() {
            Type::Indexed {
                buffer: index_buffer,
                n_indices,
            } => {
                self.pass
                    .set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                self.pass
                    .draw_indexed(0..*n_indices, 0, 0..instance.n_instances());
            }
            Type::Sequential { n_vertices } => {
                self.pass.draw(0..*n_vertices, 0..instance.n_instances());
            }
        }

        *self.drawn_in_frame = true;
        Ok(())
    }

    fn _bind_view_handle(
        &mut self,
        handle: _ViewHandle,
        group: u32,
    ) -> Result<(), ResourceNotFound> {
        let camera = self.resources.views.get(handle.0)?;
        camera.resize(self.size, self.queue);
        self.pass.set_bind_group(group, camera.bind_group(), &[]);

        Ok(())
    }

    fn bind_texture_handle(
        &mut self,
        handle: _TextureHandle,
        group: u32,
    ) -> Result<(), ResourceNotFound> {
        let texture = self.resources._textures.get(handle.0)?;
        self.pass.set_bind_group(group, texture._bind_group(), &[]);

        Ok(())
    }

    fn bind_light_handle(
        &mut self,
        handle: LightHandle,
        group: u32,
    ) -> Result<(), ResourceNotFound> {
        let light = self.resources.lights.get(handle.0)?;
        self.pass.set_bind_group(group, light.bind_group(), &[]);

        Ok(())
    }

    fn bind_space_handle(
        &mut self,
        handle: SpaceHandle,
        group: u32,
    ) -> Result<(), ResourceNotFound> {
        let space = self.resources.spaces.get(handle.0)?;
        self.pass.set_bind_group(group, space.bind_group(), &[]);

        Ok(())
    }
}

impl<T> Layer<'_, TextureVertex, T> {
    /// Binds the [view](crate::handles::ViewHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given view handler was deleted.
    pub fn _bind_view(&mut self, handle: _ViewHandle) -> Result<(), ResourceNotFound> {
        self._bind_view_handle(handle, _shader::TEXTURED_GLOBALS_GROUP)
    }

    /// Binds the [texture](crate::handles::TextureHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given texture handler was deleted.
    pub fn bind_texture(&mut self, handle: _TextureHandle) -> Result<(), ResourceNotFound> {
        self.bind_texture_handle(handle, _shader::TEXTURED_TEXTURE_GROUP)
    }

    /// Binds the [light](crate::handles::LightHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given light handler was deleted.
    pub fn bind_light(&mut self, handle: LightHandle) -> Result<(), ResourceNotFound> {
        self.bind_light_handle(handle, _shader::TEXTURED_SOURCES_GROUP)
    }

    /// Binds the [space](crate::handles::SpaceHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given space handler was deleted.
    pub fn bind_space(&mut self, handle: SpaceHandle) -> Result<(), ResourceNotFound> {
        self.bind_space_handle(handle, _shader::TEXTURED_SPACE_GROUP)
    }
}

impl<T> Layer<'_, ColorVertex, T> {
    /// Binds the [view](crate::handles::ViewHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given view handler was deleted.
    pub fn _bind_view(&mut self, handle: _ViewHandle) -> Result<(), ResourceNotFound> {
        self._bind_view_handle(handle, _shader::COLOR_GLOBALS_GROUP)
    }

    /// Binds the [light](crate::handles::LightHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given light handler was deleted.
    pub fn bind_light(&mut self, handle: LightHandle) -> Result<(), ResourceNotFound> {
        self.bind_light_handle(handle, _shader::COLOR_SOURCES_GROUP)
    }
}

impl<T> Layer<'_, FlatVertex, T> {
    /// Binds the [texture](crate::handles::TextureHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given texture handler was deleted.
    pub fn bind_texture(&mut self, handle: _TextureHandle) -> Result<(), ResourceNotFound> {
        self.bind_texture_handle(handle, _shader::FLAT_TEXTURE_GROUP)
    }
}

#[derive(Default)]
struct Groups<'l> {
    globals: Option<&'l BindGroup>,
    textures: Option<&'l BindGroup>,
}

/// The layer builder. It creates a configured [`Layer`].
#[must_use]
pub struct Builder<'l, 'd, S, T> {
    frame: &'l mut Frame<'d>,
    pipeline: &'d Pipeline,
    clear_color: Option<Linear<f32>>,
    clear_depth: bool,
    vertex_type: PhantomData<(S, T)>,
}

impl<'l, 'd, S, T> Builder<'l, 'd, S, T> {
    pub(crate) fn new(frame: &'l mut Frame<'d>, pipeline: &'d Pipeline) -> Self {
        Self {
            frame,
            pipeline,
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
    /// # use dunge::color::Standard;
    /// # #[derive(Debug)]
    /// # struct Error;
    /// # struct Frame;
    /// # impl Frame {
    /// #     fn layer(self, _: ()) -> Result<Self, Error> { Ok(self) }
    /// #     fn with_clear_color(self, _: Standard<u8>) -> Self { self }
    /// #     fn start(self) {}
    /// # }
    /// # let frame = Frame;
    /// # let layer_handle = ();
    /// let color = Standard([20, 30, 40, !0]);
    /// let mut layer = frame
    ///     .layer(layer_handle)?
    ///     .with_clear_color(color)
    ///     .start();
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// To clear a layer with a transparent color, it is enough to pass `()` as a parameter.
    ///
    /// # Example
    /// ```
    /// # #[derive(Debug)]
    /// # struct Error;
    /// # struct Frame;
    /// # impl Frame {
    /// #     fn layer(self, _: ()) -> Result<Self, Error> { Ok(self) }
    /// #     fn with_clear_color(self, _: ()) -> Self { self }
    /// #     fn start(self) {}
    /// # }
    /// # let frame = Frame;
    /// # let layer_handle = ();
    /// let mut layer = frame
    ///     .layer(layer_handle)?
    ///     .with_clear_color(())
    ///     .start();
    /// # Ok::<(), Error>(())
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
    pub fn with_clear_depth(self) -> Self {
        Self {
            clear_depth: true,
            ..self
        }
    }

    /// Starts draw the layer.
    pub fn start(self) -> Layer<'l, S, T> {
        self.frame
            .start_layer(self.pipeline, self.clear_color, self.clear_depth)
    }

    /// Starts draw the layer.
    pub fn _start(self) -> Layer<'l, S, T>
    where
        S: _Vertex,
    {
        self.frame
            ._start_layer(self.pipeline, self.clear_color, self.clear_depth)
    }
}
