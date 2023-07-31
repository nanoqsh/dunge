use {
    crate::{
        color::{Color, Rgba},
        frame::Frame,
        mesh::{Mesh, MeshBuffer},
        pipeline::{Inputs, Parameters as PipelineParameters, Pipeline, Slots},
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::{Globals, Instance, InstanceColor, Lights, Spaces, Textures},
        topology::{Topology, TriangleList},
    },
    dunge_shader::Shader as ShaderData,
    std::marker::PhantomData,
    wgpu::{BindGroup, RenderPass},
};

/// The drawable layer of the [frame](Frame).
///
/// Can be created from the [context](crate::Context) by calling
/// the [`create_layer`](crate::Context::create_layer) or
/// [`create_layer_with`](crate::Context::create_layer_with) functions.
#[must_use]
pub struct Layer<S, T = TriangleList> {
    pipeline: Box<Pipeline>,
    ty: PhantomData<(S, T)>,
}

impl<S, T> Layer<S, T> {
    pub(crate) fn new(state: &State, shader: &ShaderData, params: PipelineParameters) -> Self
    where
        S: Shader,
        T: Topology,
    {
        let inputs = Inputs::new::<S::Vertex>(S::INSTANCE_COLORS);
        Self {
            pipeline: Box::new(Pipeline::new(
                state,
                shader,
                Some(&inputs),
                PipelineParameters {
                    topology: T::VALUE.into_inner(),
                    ..params
                },
            )),
            ty: PhantomData,
        }
    }

    pub(crate) fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }
}

/// The frame's active layer.
///
/// Can be created from the [`Frame`] instance by calling the [`layer`](Frame::layer) function.
#[must_use]
pub struct ActiveLayer<'l, S, T> {
    pass: RenderPass<'l>,
    size: (u32, u32),
    slots: Slots,
    instance_color: Option<&'l InstanceColor>,
    groups: Groups<'l>,
    ty: PhantomData<(S, T)>,
}

impl<'l, S, T> ActiveLayer<'l, S, T> {
    pub(crate) fn new(pass: RenderPass<'l>, size: (u32, u32), slots: Slots) -> Self {
        Self {
            pass,
            size,
            slots,
            instance_color: None,
            groups: Groups::default(),
            ty: PhantomData,
        }
    }

    /// Binds the globals.
    pub fn bind_globals(&mut self, globals: &'l Globals<S>) -> &mut Self {
        globals.update_size(self.size);
        self.groups.globals = Some(globals.bind());
        self
    }

    /// Binds the textures.
    pub fn bind_textures(&mut self, textures: &'l Textures<S>) -> &mut Self {
        self.groups.textures = Some(textures.bind());
        self
    }

    /// Binds the light sources.
    pub fn bind_lights(&mut self, lights: &'l Lights<S>) -> &mut Self {
        self.groups.lights = Some(lights.bind());
        self
    }

    /// Binds the light spaces.
    pub fn bind_spaces(&mut self, spaces: &'l Spaces<S>) -> &mut Self {
        self.groups.spaces = Some(spaces.bind());
        self
    }

    /// Binds the color instance.
    ///
    /// # Panics
    /// Panics if the shader has no instance colors.
    pub fn bind_instance_color(&mut self, cols: &'l InstanceColor) -> &mut Self
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        assert!(
            info.has_instance_colors(),
            "the shader has no instance colors",
        );

        self.instance_color = Some(cols);
        self
    }

    /// Draws the [mesh](crate::Mesh).
    ///
    /// # Panics
    /// - If globals is not set but required.
    /// - If textures is not set but required.
    /// - If light sources is not set but required.
    /// - If light spaces is not set but required.
    /// - If instance color is not set but required.
    pub fn draw(&mut self, mesh: &'l Mesh<S::Vertex, T>, instance: &'l Instance)
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        if info.has_globals() {
            let (index, group) = self.groups.globals.expect("globals is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_textures() {
            let (index, group) = self.groups.textures.expect("textures is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_lights() {
            let (index, group) = self.groups.lights.expect("light sources is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_spaces() {
            let (index, group) = self.groups.spaces.expect("light spaces is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_instance_colors() {
            let instance_color = self.instance_color.expect("instance color is not set");
            self.pass
                .set_vertex_buffer(self.slots.instance_color, instance_color.buffer().slice());
        }

        self.draw_mesh(mesh.buffer(), instance);
    }

    fn draw_mesh(&mut self, mesh: MeshBuffer<'l>, instance: &'l Instance) {
        use wgpu::IndexFormat;

        let instances = instance.buffer();
        self.pass
            .set_vertex_buffer(self.slots.instance, instances.slice());

        self.pass
            .set_vertex_buffer(self.slots.vertex, mesh.verts.slice());

        match mesh.indxs {
            Some(buf) => {
                self.pass.set_index_buffer(buf.slice(), IndexFormat::Uint16);
                self.pass.draw_indexed(0..buf.len(), 0, 0..instances.len());
            }
            None => self.pass.draw(0..mesh.verts.len(), 0..instances.len()),
        }
    }
}

#[derive(Default)]
struct Groups<'l> {
    globals: Option<(u32, &'l BindGroup)>,
    textures: Option<(u32, &'l BindGroup)>,
    lights: Option<(u32, &'l BindGroup)>,
    spaces: Option<(u32, &'l BindGroup)>,
}

/// The layer builder. It creates a configured [`ActiveLayer`].
#[must_use]
pub struct Builder<'d, 'l, S, T> {
    frame: &'l mut Frame<'d>,
    pipeline: &'l Pipeline,
    clear_color: Option<[f64; 4]>,
    clear_depth: bool,
    vertex_type: PhantomData<(S, T)>,
}

impl<'d, 'l, S, T> Builder<'d, 'l, S, T> {
    pub(crate) fn new(frame: &'l mut Frame<'d>, pipeline: &'l Pipeline) -> Self {
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
    /// the previous layer or frame with some color.
    pub fn with_clear_color(self, Color(col): Rgba) -> Self {
        Self {
            clear_color: Some(col.map(f64::from)),
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
    pub fn start(self) -> ActiveLayer<'l, S, T> {
        self.frame
            .start_layer(self.pipeline, self.clear_color, self.clear_depth)
    }
}
