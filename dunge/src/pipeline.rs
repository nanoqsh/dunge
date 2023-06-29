use {
    crate::{
        _shader::_Shader,
        _vertex::_Vertex,
        error::ResourceNotFound,
        framebuffer::Framebuffer,
        groups::_Groups,
        handles::{LayerHandle, ShaderHandle},
        render::{Render, Shaders},
        resources::Resources,
        shader::Shader,
        shader_data::InstanceModel,
        topology::Topology,
        vertex::Vertex,
    },
    dunge_shader::{
        Globals as Gl, Group, Layout, Lights as Lt, Shader as ShaderData, SourceBindings,
        SpaceBindings, Spaces as Sp, TextureBindings, Textures as Tx,
    },
    std::marker::PhantomData,
    wgpu::{
        BindGroupLayout, BlendState, CompareFunction, Device, PolygonMode, PrimitiveTopology,
        RenderPipeline, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
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
pub struct ParametersBuilder<'a, S, T> {
    render: &'a Render,
    resources: &'a mut Resources,
    params: Parameters,
    vertex_type: PhantomData<(S, T)>,
}

impl<'a, S, T> ParametersBuilder<'a, S, T> {
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
    pub fn build(self, shader: ShaderHandle<S>) -> Result<LayerHandle<S, T>, ResourceNotFound>
    where
        S: Shader,
        T: Topology,
    {
        self.resources
            .create_layer(self.render, self.params, shader)
    }

    pub fn _build(self) -> LayerHandle<S, T>
    where
        S: _Vertex,
        T: Topology,
    {
        self.resources._create_layer(self.render, self.params)
    }
}

pub(crate) struct Pipeline {
    inner: RenderPipeline,
    groups: Groups,
}

impl Pipeline {
    pub const VERTEX_BUFFER_SLOT: u32 = 0;
    pub const INSTANCE_BUFFER_SLOT: u32 = 1;

    pub fn new(
        device: &Device,
        shader: &ShaderData,
        vert: Option<&VertexLayout>,
        params: Parameters,
    ) -> Self {
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

        let layouts;
        Self {
            inner: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: ShaderData::VERTEX_ENTRY_POINT,
                    buffers: match vert {
                        Some(vl) => {
                            layouts = [vl.buffer_layout(), InstanceModel::LAYOUT];
                            &layouts[..]
                        }
                        None => &[],
                    },
                },
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: ShaderData::FRAGMENT_ENTRY_POINT,
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
            }),
            groups,
        }
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

        Self {
            inner: {
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
            },
            groups: Groups::new(
                device,
                &Layout {
                    globals: Group {
                        num: 0,
                        bindings: Gl::default(),
                    },
                    textures: Group {
                        num: 0,
                        bindings: Tx::default(),
                    },
                    lights: Group {
                        num: 0,
                        bindings: Lt::default(),
                    },
                    spaces: Group {
                        num: 0,
                        bindings: Sp::default(),
                    },
                },
            ),
        }
    }

    pub fn as_ref(&self) -> &RenderPipeline {
        &self.inner
    }

    pub fn globals(&self) -> Option<&GroupLayout<Globals>> {
        self.groups.globals.as_ref()
    }

    pub fn textures(&self) -> Option<&GroupLayout<Textures>> {
        self.groups.textures.as_ref()
    }

    pub fn lights(&self) -> Option<&GroupLayout<Lights>> {
        self.groups.lights.as_ref()
    }

    pub fn spaces(&self) -> Option<&GroupLayout<Spaces>> {
        self.groups.spaces.as_ref()
    }
}

struct Groups {
    globals: Option<GroupLayout<Globals>>,
    textures: Option<GroupLayout<Textures>>,
    lights: Option<GroupLayout<Lights>>,
    spaces: Option<GroupLayout<Spaces>>,
}

impl Groups {
    fn new(device: &Device, layout: &Layout) -> Self {
        use wgpu::*;

        let entry = |binding, visibility| BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        Self {
            globals: {
                let mut entries = vec![];
                if let Some(binding) = layout.globals.bindings.post_data {
                    entries.push(entry(
                        binding,
                        ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ));
                }

                if let Some(binding) = layout.globals.bindings.camera {
                    entries.push(entry(binding, ShaderStages::VERTEX));
                }

                if let Some(binding) = layout.globals.bindings.ambient {
                    entries.push(entry(binding, ShaderStages::FRAGMENT));
                }

                entries.first().map(|_| GroupLayout {
                    layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("globals binding"),
                        entries: &entries,
                    }),
                    bindings: Globals {
                        group: layout.globals.num,
                        camera: layout.globals.bindings.camera.unwrap_or_default(),
                        ambient: layout.globals.bindings.ambient.unwrap_or_default(),
                    },
                })
            },
            textures: {
                let mut entries = vec![];
                if let Some(TextureBindings { tdiff, sdiff }) = layout.textures.bindings.map {
                    entries.extend([
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
                    ]);
                }

                entries.first().map(|_| GroupLayout {
                    layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("textures binding"),
                        entries: &entries,
                    }),
                    bindings: Textures {
                        group: layout.textures.num,
                        map: layout.textures.bindings.map.unwrap_or_default(),
                    },
                })
            },
            lights: {
                let entries: Vec<_> = layout
                    .lights
                    .bindings
                    .source_arrays
                    .iter()
                    .flat_map(|bindings| {
                        [
                            entry(bindings.binding_array, ShaderStages::FRAGMENT),
                            entry(bindings.binding_len, ShaderStages::FRAGMENT),
                        ]
                    })
                    .collect();

                entries.first().map(|_| GroupLayout {
                    layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("lights binding"),
                        entries: &entries,
                    }),
                    bindings: Lights {
                        group: layout.lights.num,
                        source_arrays: layout.lights.bindings.source_arrays.clone(),
                    },
                })
            },
            spaces: 'spaces: {
                let ls = &layout.spaces.bindings.light_spaces;
                if ls.is_empty() {
                    break 'spaces None;
                }

                let mut entries = vec![entry(
                    ls.spaces,
                    ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                )];

                entries.extend(ls.tdiffs.iter().map(|&binding| BindGroupLayoutEntry {
                    binding,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D3,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                }));

                entries.push(BindGroupLayoutEntry {
                    binding: ls.sdiff,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });

                Some(GroupLayout {
                    layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("spaces binding"),
                        entries: &entries,
                    }),
                    bindings: Spaces {
                        group: layout.spaces.num,
                        bindings: ls.clone(),
                    },
                })
            },
        }
    }

    fn layouts(&self) -> Vec<&BindGroupLayout> {
        [
            self.globals.as_ref().map(|group| &group.layout),
            self.textures.as_ref().map(|group| &group.layout),
            self.lights.as_ref().map(|group| &group.layout),
            self.spaces.as_ref().map(|group| &group.layout),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

pub(crate) struct GroupLayout<T> {
    pub layout: BindGroupLayout,
    pub bindings: T,
}

pub(crate) struct Globals {
    pub group: u32,
    pub camera: u32,
    pub ambient: u32,
}

pub(crate) struct Textures {
    pub group: u32,
    pub map: TextureBindings,
}

pub(crate) struct Lights {
    pub group: u32,
    pub source_arrays: Vec<SourceBindings>,
}

pub(crate) struct Spaces {
    pub group: u32,
    pub bindings: SpaceBindings,
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
        use {
            crate::vertex::{Component, Component2D, Component3D},
            std::mem,
        };

        let mut offset = 0;
        let mut location = 0;
        let mut make_attr = |n| {
            let format = Self::format(n);
            let new_offset = Self::offset(n);
            let attr = VertexAttribute {
                format,
                offset,
                shader_location: location + InstanceModel::LOCATION_OFFSET,
            };

            offset += new_offset;
            location += 1;
            attr
        };

        Self {
            size: mem::size_of::<V>(),
            attrs: [
                Some(make_attr(V::Position::N_FLOATS)),
                V::Color::OPTIONAL_N_FLOATS.map(&mut make_attr),
                V::Texture::OPTIONAL_N_FLOATS.map(&mut make_attr),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }

    fn format(n: u64) -> VertexFormat {
        match n {
            2 => VertexFormat::Float32x2,
            3 => VertexFormat::Float32x3,
            _ => unreachable!(),
        }
    }

    fn offset(n: u64) -> u64 {
        use std::mem;

        n * mem::size_of::<f32>() as u64
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
