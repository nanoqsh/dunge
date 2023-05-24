use {
    crate::{
        _vertex::_Vertex,
        error::ResourceNotFound,
        framebuffer::Framebuffer,
        groups::_Groups,
        handles::{LayerHandle, ShaderHandle},
        render::{Render, Shaders},
        resources::Resources,
        shader::_Shader,
        shader_data::InstanceModel,
        topology::Topology,
        vertex::{Format, Kind, Vertex},
    },
    dunge_shader::{Layout, Shader, TextureBindings},
    std::marker::PhantomData,
    wgpu::{
        BindGroupLayout, BlendState, CompareFunction, Device, PolygonMode, PrimitiveTopology,
        RenderPipeline, TextureFormat, VertexAttribute, VertexBufferLayout,
    },
};

#[derive(Clone, Copy)]
pub(crate) struct Parameters {
    pub blend: BlendState,
    pub topology: PrimitiveTopology,
    pub cull_faces: bool,
    pub mode: PolygonMode,
    pub depth_stencil: Option<CompareFunction>,
}

impl Default for Parameters {
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

/// Builds new layer with specific parameters.
#[must_use]
pub struct ParametersBuilder<'a, V, T> {
    render: &'a Render,
    resources: &'a mut Resources,
    params: Parameters,
    vertex_type: PhantomData<(V, T)>,
}

impl<'a, V, T> ParametersBuilder<'a, V, T> {
    pub(crate) fn new(render: &'a Render, resources: &'a mut Resources) -> Self {
        Self {
            render,
            resources,
            params: Parameters::default(),
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

    /// Builds new layer.
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`] if given shader handler was deleted.
    pub fn build(self, shader: ShaderHandle<V>) -> Result<LayerHandle<V, T>, ResourceNotFound>
    where
        V: Vertex,
        T: Topology,
    {
        self.resources
            .create_layer(self.render, self.params, shader)
    }

    pub fn _build(self) -> LayerHandle<V, T>
    where
        V: _Vertex,
        T: Topology,
    {
        self.resources._create_layer(self.render, self.params)
    }
}

pub(crate) struct Pipeline(RenderPipeline);

impl Pipeline {
    pub fn new(device: &Device, shader: &Shader, vert: &VertexLayout, params: Parameters) -> Self {
        use wgpu::*;

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader.source.as_str().into()),
        });

        let groups = Groups::new(device, &shader.layout);
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &groups.layouts(),
            push_constant_ranges: &[],
        });

        Self(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &module,
                entry_point: Shader::VERTEX_ENTRY_POINT,
                buffers: &[vert.buffer_layout(), InstanceModel::LAYOUT],
            },
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Shader::FRAGMENT_ENTRY_POINT,
                targets: &[Some(ColorTargetState {
                    format: Framebuffer::RENDER_FORMAT,
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
                format: Framebuffer::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        }))
    }

    pub fn _new(
        device: &Device,
        shaders: &Shaders,
        groups: &_Groups,
        format: TextureFormat,
        shader: _Shader,
        params: Parameters,
    ) -> Self {
        use wgpu::*;

        Self({
            let module = shaders.module(device, shader);
            let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: groups.bind_group_layouts(shader).as_slice(),
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
                    format: Framebuffer::DEPTH_FORMAT,
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

struct Groups {
    globals: Option<BindGroupLayout>,
    textures: Option<BindGroupLayout>,
}

impl Groups {
    fn new(device: &Device, layout: &Layout) -> Self {
        use wgpu::*;

        Self {
            globals: layout.globals.camera.map(|binding| {
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("globals binding"),
                    entries: &[BindGroupLayoutEntry {
                        binding,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                })
            }),
            textures: layout
                .textures
                .texture
                .map(|TextureBindings { tdiff, sdiff }| {
                    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("textures binding"),
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: tdiff,
                                visibility: ShaderStages::FRAGMENT,
                                ty: BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: TextureViewDimension::D2,
                                    sample_type: TextureSampleType::Float { filterable: true },
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: sdiff,
                                visibility: ShaderStages::FRAGMENT,
                                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                    })
                }),
        }
    }

    fn layouts(&self) -> Vec<&BindGroupLayout> {
        [&self.globals, &self.textures]
            .into_iter()
            .flatten()
            .collect()
    }
}

pub(crate) struct VertexLayout {
    size: usize,
    attrs: Vec<VertexAttribute>,
}

impl VertexLayout {
    pub fn new<V>() -> Self
    where
        V: Vertex,
    {
        use {std::mem, wgpu::VertexFormat};

        Self {
            size: mem::size_of::<V>(),
            attrs: V::FIELDS
                .iter()
                .scan(0, |offset, field| {
                    let location = match field.kind {
                        Kind::Position => 0,
                        Kind::Color => 1,
                        Kind::TextureMap => todo!("what if color + texture will be used"),
                    };

                    let attr = VertexAttribute {
                        format: match field.format {
                            Format::FloatX2 => VertexFormat::Float32x2,
                            Format::FloatX3 => VertexFormat::Float32x3,
                        },
                        offset: *offset,
                        shader_location: location + InstanceModel::LOCATION_OFFSET,
                    };

                    *offset += field.format.bytes_size() as u64;
                    Some(attr)
                })
                .collect(),
        }
    }

    fn buffer_layout(&self) -> VertexBufferLayout {
        use wgpu::VertexStepMode;

        VertexBufferLayout {
            array_stride: self.size as _,
            step_mode: VertexStepMode::Vertex,
            attributes: &self.attrs,
        }
    }
}
