//! Layer types.

use {
    crate::{
        buffer::Format,
        shader::{ShaderData, SlotNumbers},
        state::State,
    },
    std::marker::PhantomData,
};

/// [Layer's](Layer) blend mode.
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

/// [Layer's](Layer) primitive topology.
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

/// [Layer's](Layer) polygon mode.
#[derive(Clone, Copy, Default)]
pub enum Polygon {
    #[default]
    Fill,
    #[cfg(not(target_family = "wasm"))]
    Line,
    #[cfg(not(target_family = "wasm"))]
    Point,
}

impl Polygon {
    fn wgpu(self) -> wgpu::PolygonMode {
        match self {
            Self::Fill => wgpu::PolygonMode::Fill,
            #[cfg(not(target_family = "wasm"))]
            Self::Line => wgpu::PolygonMode::Line,
            #[cfg(not(target_family = "wasm"))]
            Self::Point => wgpu::PolygonMode::Point,
        }
    }
}

/// The [layer](Layer) configuration.
///
/// Used when creating a layer in the [`make_layer`](crate::Context::make_layer) method.
#[derive(Clone, Copy, Default)]
pub struct Config {
    pub format: Format,
    pub blend: Blend,
    pub topology: Topology,
    pub polygon: Polygon,
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

/// The render layer.
///
/// Can be created using the [`make_layer`](crate::Context::make_layer) method.
pub struct Layer<I> {
    slots: SlotNumbers,
    depth: bool,
    format: Format,
    render: wgpu::RenderPipeline,
    inp: PhantomData<I>,
}

impl<I> Layer<I> {
    pub(crate) fn new(state: &State, shader: &ShaderData, conf: Config) -> Self {
        let Config {
            format,
            blend,
            topology,
            polygon,
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
                polygon_mode: polygon.wgpu(),
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
            depth,
            format,
            render,
            inp: PhantomData,
        }
    }

    pub(crate) fn slots(&self) -> SlotNumbers {
        self.slots
    }

    /// Whether the layer uses depth.
    pub fn depth(&self) -> bool {
        self.depth
    }

    /// Returns the color [format](Format) used by the layer.
    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn render(&self) -> &wgpu::RenderPipeline {
        &self.render
    }
}
