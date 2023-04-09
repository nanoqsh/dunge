use {
    crate::{
        depth_frame::DepthFrame,
        handles::LayerHandle,
        render::{Layouts, Render, Shaders},
        shader::Shader,
        topology::Topology,
        vertex::Vertex,
    },
    std::marker::PhantomData,
    wgpu::{
        BlendState, CompareFunction, Device, PolygonMode, PrimitiveTopology, RenderPipeline,
        TextureFormat,
    },
};

pub(crate) struct Pipeline(RenderPipeline);

impl Pipeline {
    pub fn new(
        device: &Device,
        shaders: &Shaders,
        layouts: &Layouts,
        format: TextureFormat,
        shader: Shader,
        params: PipelineParameters,
    ) -> Self {
        use wgpu::*;

        Self({
            let module = shaders.module(device, shader);
            let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: layouts.bind_group_layouts(shader).as_slice(),
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: VertexState {
                    module,
                    entry_point: "vs_main",
                    buffers: shader.buffers(),
                },
                fragment: Some(FragmentState {
                    module,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(params.blend),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: params.topology,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: params.cull_faces.then_some(Face::Back),
                    polygon_mode: params.mode,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: params.depth_stencil.map(|depth_compare| DepthStencilState {
                    format: DepthFrame::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                multiview: None,
            })
        })
    }

    pub fn as_ref(&self) -> &RenderPipeline {
        &self.0
    }
}

/// Builds new layer with specific parameters.
#[must_use]
pub struct ParametersBuilder<'a, V, T> {
    render: &'a mut Render,
    params: PipelineParameters,
    vertex_type: PhantomData<(V, T)>,
}

impl<'a, V, T> ParametersBuilder<'a, V, T> {
    pub(crate) fn new(render: &'a mut Render) -> Self {
        Self {
            render,
            params: PipelineParameters::default(),
            vertex_type: PhantomData,
        }
    }

    pub fn with_blend(mut self, blend: Blend) -> Self {
        self.params.blend = match blend {
            Blend::Replace => BlendState::REPLACE,
            Blend::AlphaBlending => BlendState::ALPHA_BLENDING,
        };

        self
    }

    pub fn with_cull_faces(mut self, cull_faces: bool) -> Self {
        self.params.cull_faces = cull_faces;
        self
    }

    pub fn with_draw_mode(mut self, draw_mode: DrawMode) -> Self {
        self.params.mode = match draw_mode {
            DrawMode::Fill => PolygonMode::Fill,
            DrawMode::Line => PolygonMode::Line,
            DrawMode::Point => PolygonMode::Point,
        };

        self
    }

    pub fn with_depth_compare(mut self, depth_compare: Compare) -> Self {
        self.params.depth_stencil = Some(match depth_compare {
            Compare::Never => CompareFunction::Never,
            Compare::Less => CompareFunction::Less,
            Compare::Greater => CompareFunction::Greater,
            Compare::Always => CompareFunction::Always,
        });

        self
    }

    #[must_use]
    pub fn build(self) -> LayerHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        self.render.create_layer(self.params)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PipelineParameters {
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub cull_faces: bool,
    pub mode: PolygonMode,
    pub depth_stencil: Option<CompareFunction>,
}

impl Default for PipelineParameters {
    fn default() -> Self {
        Self {
            blend: BlendState::REPLACE,
            topology: PrimitiveTopology::TriangleList,
            cull_faces: true,
            mode: PolygonMode::Fill,
            depth_stencil: Some(CompareFunction::Less),
        }
    }
}

/// Layer's blending
#[derive(Clone, Copy)]
pub enum Blend {
    Replace,
    AlphaBlending,
}

/// Type of drawing mode for polygons
#[derive(Clone, Copy)]
pub enum DrawMode {
    Fill,
    Line,
    Point,
}

/// Depth comparison function
#[derive(Clone, Copy)]
pub enum Compare {
    /// Function never passes
    Never,

    /// Function passes if new value less than existing value
    Less,

    /// Function passes if new value is greater than existing value
    Greater,

    /// Function always passes
    Always,
}
