use {
    crate::{
        context::{Compare, LayerParameters},
        render::Render,
        shader::Shader,
    },
    wgpu::RenderPipeline,
};

pub(crate) struct Pipeline(RenderPipeline);

impl Pipeline {
    pub fn new(render: &Render, shader: Shader, params: LayerParameters) -> Self {
        use {crate::depth_frame::DepthFrame, wgpu::*};

        Self({
            let module = render.shader_module(shader);

            let pipeline_layout =
                render
                    .device()
                    .create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: render.bind_group_layouts(shader).as_slice(),
                        push_constant_ranges: &[],
                    });

            render
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module,
                        entry_point: "vs_main",
                        buffers: shader.buffers(),
                    },
                    fragment: Some(FragmentState {
                        module,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: render.format(),
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Ccw,
                        cull_mode: params.cull_faces.then_some(Face::Back),
                        polygon_mode: PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: DepthFrame::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: match params.depth_compare {
                            Compare::Never => CompareFunction::Never,
                            Compare::Less => CompareFunction::Less,
                            Compare::Greater => CompareFunction::Greater,
                            Compare::Always => CompareFunction::Always,
                        },
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
