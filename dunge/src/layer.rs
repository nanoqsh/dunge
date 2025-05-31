//! Layer types.

use {
    crate::{
        buffer::Format,
        shader::{ShaderData, SlotNumbers},
        state::State,
    },
    std::marker::PhantomData,
};

#[derive(Clone, Copy, Default)]
pub enum Blend {
    #[default]
    None,
    Replace,
    Alpha,
}

impl Blend {
    fn wgpu(self) -> Option<wgpu::BlendState> {
        match self {
            Self::None => None,
            Self::Replace => Some(wgpu::BlendState::REPLACE),
            Self::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

impl Topology {
    fn wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::PointList => wgpu::PrimitiveTopology::PointList,
            Self::LineList => wgpu::PrimitiveTopology::LineList,
            Self::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Self::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Self::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Fill,
    Line,
    Point,
}

impl Mode {
    fn wgpu(self) -> wgpu::PolygonMode {
        match self {
            Self::Fill => wgpu::PolygonMode::Fill,
            Self::Line => wgpu::PolygonMode::Line,
            Self::Point => wgpu::PolygonMode::Point,
        }
    }
}

#[derive(Clone, Default)]
pub struct Config {
    pub format: Format,
    pub blend: Blend,
    pub topology: Topology,
    pub mode: Mode,
    pub depth: bool,
}

impl From<Format> for Config {
    fn from(format: Format) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }
}

pub struct Layer<I> {
    slots: SlotNumbers,
    depth: bool,
    format: Format,
    render: wgpu::RenderPipeline,
    inp: PhantomData<I>,
}

impl<I> Layer<I> {
    pub(crate) fn new(state: &State, shader: &ShaderData, conf: &Config) -> Self {
        let Config {
            format,
            blend,
            topology,
            mode,
            depth,
        } = conf;

        let targets = [Some(wgpu::ColorTargetState {
            format: format.wgpu(),
            blend: blend.wgpu(),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let module = shader.module();
        let buffers = shader.vertex_buffers();
        let topology = topology.wgpu();
        let desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: topology.is_strip().then_some(wgpu::IndexFormat::Uint32),
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: mode.wgpu(),
                ..Default::default()
            },
            depth_stencil: depth.then_some(wgpu::DepthStencilState {
                format: Format::Depth.wgpu(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &targets,
            }),
            multiview: None,
            cache: None,
        };

        let render = state.device().create_render_pipeline(&desc);

        Self {
            slots: shader.slots(),
            depth: *depth,
            format: *format,
            render,
            inp: PhantomData,
        }
    }

    pub(crate) fn slots(&self) -> SlotNumbers {
        self.slots
    }

    pub fn depth(&self) -> bool {
        self.depth
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn render(&self) -> &wgpu::RenderPipeline {
        &self.render
    }
}
