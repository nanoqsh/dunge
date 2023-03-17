use {
    crate::{
        render::{Layouts, Shaders},
        shader::Shader,
    },
    wgpu::{Device, RenderPipeline, TextureFormat},
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
        use_depth_stencil: bool,
    ) -> Self {
        use {crate::depth_frame::DepthFrame, wgpu::*};

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
                        blend: match params.blend {
                            Blend::Replace => Some(BlendState::REPLACE),
                            Blend::AlphaBlending => Some(BlendState::ALPHA_BLENDING),
                        },
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: match params.topology {
                        Topology::TriangleList => PrimitiveTopology::TriangleList,
                        Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
                    },
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: params.cull_faces.then_some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: use_depth_stencil.then(|| DepthStencilState {
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

#[derive(Clone, Copy)]
pub(crate) struct PipelineParameters {
    pub blend: Blend,
    pub topology: Topology,
    pub cull_faces: bool,
    pub depth_compare: Compare,
}

impl Default for PipelineParameters {
    fn default() -> Self {
        Self {
            blend: Blend::Replace,
            topology: Topology::TriangleList,
            cull_faces: true,
            depth_compare: Compare::Less,
        }
    }
}

/// Layer's blending
#[derive(Clone, Copy)]
pub enum Blend {
    Replace,
    AlphaBlending,
}

/// Mesh topology
#[derive(Clone, Copy)]
pub enum Topology {
    TriangleList,
    TriangleStrip,
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
