use {
    crate::{
        color::{Color, Rgba},
        frame::Frame,
        mesh::{Mesh, MeshBuffer},
        pipeline::{Parameters as PipelineParameters, Pipeline, VertexLayout},
        shader::{Shader, ShaderInfo},
        shader_data::{
            globals::Globals, lights::Lights, spaces::Spaces, textures::Textures, Instance,
        },
        topology::{Topology, TriangleList},
    },
    dunge_shader::Shader as ShaderData,
    std::marker::PhantomData,
    wgpu::{BindGroup, Device, RenderPass},
};

#[must_use]
pub struct Layer<S, T = TriangleList> {
    pipeline: Pipeline,
    ty: PhantomData<(S, T)>,
}

impl<S, T> Layer<S, T> {
    pub(crate) fn new(device: &Device, shader: &ShaderData, params: PipelineParameters) -> Self
    where
        S: Shader,
        T: Topology,
    {
        let vert = VertexLayout::new::<S::Vertex>();
        Self {
            pipeline: Pipeline::new(
                device,
                shader,
                Some(&vert),
                PipelineParameters {
                    topology: T::VALUE.into_inner(),
                    ..params
                },
            ),
            ty: PhantomData,
        }
    }

    pub(crate) fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }
}

/// The frame's active layer.
///
/// Can be created from a [`Frame`] instance by calling a [`layer`](Frame::layer) function.
#[must_use]
pub struct ActiveLayer<'l, S, T> {
    pass: RenderPass<'l>,
    size: (u32, u32),
    groups: Groups<'l>,
    ty: PhantomData<(S, T)>,
}

impl<'l, S, T> ActiveLayer<'l, S, T> {
    pub(crate) fn new(pass: RenderPass<'l>, size: (u32, u32)) -> Self {
        Self {
            pass,
            size,
            groups: Groups::default(),
            ty: PhantomData,
        }
    }

    pub fn bind_globals(&mut self, globals: &'l Globals<S>) -> &mut Self {
        globals.update_size(self.size);
        self.groups.globals = Some(globals.bind());
        self
    }

    pub fn bind_textures(&mut self, textures: &'l Textures<S>) -> &mut Self {
        self.groups.textures = Some(textures.bind());
        self
    }

    pub fn bind_lights(&mut self, lights: &'l Lights<S>) -> &mut Self {
        self.groups.lights = Some(lights.bind());
        self
    }

    pub fn bind_spaces(&mut self, spaces: &'l Spaces<S>) -> &mut Self {
        self.groups.spaces = Some(spaces.bind());
        self
    }

    /// Draws the [mesh](crate::Mesh).
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
            let (index, group) = self.groups.lights.expect("lights is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        if info.has_spaces() {
            let (index, group) = self.groups.spaces.expect("spaces is not set");
            self.pass.set_bind_group(index, group, &[]);
        }

        self.draw_mesh(mesh.buffer(), instance);
    }

    fn draw_mesh(&mut self, mesh: MeshBuffer<'l>, instance: &'l Instance) {
        use wgpu::IndexFormat;

        let instances = instance.buffer();
        self.pass
            .set_vertex_buffer(Pipeline::INSTANCE_BUFFER_SLOT, instances.slice());

        self.pass
            .set_vertex_buffer(Pipeline::VERTEX_BUFFER_SLOT, mesh.verts.slice());

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
    /// the previous layer (or frame) with some color.
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
    pub fn start(self) -> ActiveLayer<'l, S, T> {
        self.frame
            .start_layer(self.pipeline, self.clear_color, self.clear_depth)
    }
}
