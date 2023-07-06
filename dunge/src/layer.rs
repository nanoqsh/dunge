use {
    crate::{
        color::{Color, Rgba},
        error::{Error, NotSetError, ResourceNotFound},
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
            vertex_type: PhantomData,
        }
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

        self.groups.globals = Some(globals.bind());
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
        self.groups.textures = Some(textures.bind());
        Ok(self)
    }

    /// Binds the [lights](crate::handles::LightsHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given lights handler was deleted.
    pub fn bind_lights(&mut self, handle: LightsHandle<S>) -> Result<&mut Self, ResourceNotFound> {
        let lights = self.resources.lights.get(handle.id())?;
        self.groups.lights = Some(lights.bind());
        Ok(self)
    }

    /// Binds the [spaces](crate::handles::SpacesHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`](crate::error::ResourceNotFound)
    /// if given spaces handler was deleted.
    pub fn bind_spaces(&mut self, handle: SpacesHandle<S>) -> Result<&mut Self, ResourceNotFound> {
        let spaces = self.resources.spaces.get(handle.id())?;
        self.groups.spaces = Some(spaces.bind());
        Ok(self)
    }

    /// Draws the [mesh](crate::handles::MeshHandle).
    ///
    /// # Errors
    /// See [`Error`] for details.
    pub fn draw(
        &mut self,
        mesh: MeshHandle<S::Vertex, T>,
        instance: InstanceHandle,
    ) -> Result<(), Error>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        if info.has_globals() {
            let (index, group) = self.groups.globals.ok_or(NotSetError::Globals)?;
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_textures() {
            let (index, group) = self.groups.textures.ok_or(NotSetError::Textures)?;
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_lights() {
            let (index, group) = self.groups.lights.ok_or(NotSetError::Lights)?;
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_spaces() {
            let (index, group) = self.groups.spaces.ok_or(NotSetError::Spaces)?;
            self.pass.set_bind_group(index, group, &[]);
        }

        let mesh = self.resources.meshes.get(mesh.id())?;
        let instance = self.resources.instances.get(instance.0)?;
        self.draw_mesh(mesh, instance);
        Ok(())
    }

    fn draw_mesh(&mut self, mesh: &'l Mesh, instance: &'l Instance) {
        use wgpu::IndexFormat;

        let instances = instance.buffer();
        self.pass
            .set_vertex_buffer(Pipeline::INSTANCE_BUFFER_SLOT, instances.slice());

        let verts = mesh.vertex_buffer();
        self.pass
            .set_vertex_buffer(Pipeline::VERTEX_BUFFER_SLOT, verts.slice());

        match mesh.index_buffer() {
            Some(buf) => {
                self.pass.set_index_buffer(buf.slice(), IndexFormat::Uint16);
                self.pass.draw_indexed(0..buf.len(), 0, 0..instances.len());
            }
            None => self.pass.draw(0..verts.len(), 0..instances.len()),
        }

        *self.drawn_in_frame = true;
    }
}

#[derive(Default)]
struct Groups<'l> {
    globals: Option<(u32, &'l BindGroup)>,
    textures: Option<(u32, &'l BindGroup)>,
    lights: Option<(u32, &'l BindGroup)>,
    spaces: Option<(u32, &'l BindGroup)>,
}

/// The layer builder. It creates a configured [`Layer`].
#[must_use]
pub struct Builder<'l, 'd, S, T> {
    frame: &'l mut Frame<'d>,
    pipeline: &'d Pipeline,
    clear_color: Option<[f64; 4]>,
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
    /// Don't set this setting if you don't want to fill
    /// the previous layer (or frame) with some color.
    /// Or set to clear the current buffer if a layer is already drawn
    /// into the frame by calling [`commit_in_frame`](crate::Frame::commit_in_frame).
    pub fn with_clear_color(self, Color(col): Rgba) -> Self {
        Self {
            clear_color: Some(col.map(|v| v as f64)),
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
}
