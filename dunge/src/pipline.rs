#![allow(clippy::wildcard_imports)]

use wgpu::{
    BindGroupLayout, DepthStencilState, Device, Face, PrimitiveTopology, RenderPipeline,
    TextureFormat, VertexBufferLayout,
};

pub(crate) struct PipelineData<'a> {
    pub(crate) shader_src: &'a str,
    pub(crate) bind_group_layouts: &'a [&'a BindGroupLayout],
    pub(crate) vertex_buffers: &'a [VertexBufferLayout<'a>],
    pub(crate) fragment_texture_format: TextureFormat,
    pub(crate) topology: PrimitiveTopology,
    pub(crate) cull_mode: Option<Face>,
    pub(crate) depth_stencil: Option<DepthStencilState>,
}

pub(crate) struct Pipeline {
    pipeline: RenderPipeline,
}

impl Pipeline {
    pub(crate) fn new(device: &Device, data: PipelineData) -> Self {
        use wgpu::*;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(data.shader_src.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: data.bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: data.vertex_buffers,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: data.fragment_texture_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: data.topology,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: data.cull_mode,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: data.depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self { pipeline }
    }

    pub(crate) fn as_ref(&self) -> &RenderPipeline {
        &self.pipeline
    }
}
