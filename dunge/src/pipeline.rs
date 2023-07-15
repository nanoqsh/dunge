use {
    crate::{
        framebuffer::Framebuffer,
        layer::Layer,
        render::State,
        scheme::Scheme,
        shader::Shader,
        shader_data::{ModelColor, ModelTransform},
        topology::Topology,
        vertex::Vertex,
    },
    dunge_shader::{Layout, Shader as ShaderData, SourceBindings, SpaceBindings, TextureBindings},
    std::marker::PhantomData,
    wgpu::{
        BindGroupLayout, BlendState, CompareFunction, Device, PolygonMode, PrimitiveTopology,
        RenderPipeline, VertexAttribute, VertexBufferLayout, VertexFormat,
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

/// Layer's blending.
#[derive(Clone, Copy)]
pub enum Blend {
    Replace,
    AlphaBlending,
}

/// Type of drawing mode for polygons.
#[derive(Clone, Copy)]
pub enum DrawMode {
    Fill,
    Line,
    Point,
}

/// Depth comparison function.
#[derive(Clone, Copy)]
pub enum Compare {
    /// Function never passes.
    Never,

    /// Function passes if new value less than existing value.
    Less,

    /// Function passes if new value is greater than existing value.
    Greater,

    /// Function always passes.
    Always,
}

/// Builds new layer with specific parameters.
#[must_use]
pub struct LayerBuilder<'a, S, T> {
    state: &'a State,
    params: Parameters,
    vertex_type: PhantomData<(S, T)>,
}

impl<'a, S, T> LayerBuilder<'a, S, T> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self {
            state,
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

    /// Builds a new layer.
    pub fn build(self, scheme: &Scheme<S>) -> Layer<S, T>
    where
        S: Shader,
        T: Topology,
    {
        Layer::new(self.state.device(), scheme.data(), self.params)
    }
}

pub(crate) struct Pipeline {
    inner: RenderPipeline,
    instances: Instances,
    groups: Groups,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        shader: &ShaderData,
        inputs: Option<&Inputs>,
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

        Self {
            inner: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: ShaderData::VERTEX_ENTRY_POINT,
                    buffers: &inputs.map(Inputs::buffer_layouts).unwrap_or_default(),
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
            instances: Instances {
                color: inputs.is_some_and(|inputs| inputs.instance_color),
            },
            groups,
        }
    }

    pub fn as_ref(&self) -> &RenderPipeline {
        &self.inner
    }

    pub fn slots(&self) -> Slots {
        self.instances.slots()
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

#[derive(Clone, Copy)]
struct Instances {
    color: bool,
}

impl Instances {
    fn slots(self) -> Slots {
        Slots {
            instance: 0,
            instance_color: 1,
            vertex: 1 + self.color as u32,
        }
    }
}

pub(crate) struct Slots {
    pub instance: u32,
    pub instance_color: u32,
    pub vertex: u32,
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
            textures: 'textures: {
                let tx = &layout.textures.bindings;
                if tx.is_empty() {
                    break 'textures None;
                }

                let mut entries = Vec::with_capacity(tx.map.tmaps.len() + 1);
                for &tmap in &tx.map.tmaps {
                    entries.push(BindGroupLayoutEntry {
                        binding: tmap,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    });
                }

                entries.push(BindGroupLayoutEntry {
                    binding: tx.map.smap,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });

                Some(GroupLayout {
                    layout: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("textures binding"),
                        entries: &entries,
                    }),
                    bindings: Textures {
                        group: layout.textures.num,
                        map: tx.map.clone(),
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

                entries.extend(ls.tspaces.iter().map(|&binding| BindGroupLayoutEntry {
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
                    binding: ls.sspace,
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

pub(crate) struct Inputs {
    instance_color: bool,
    vertex: VertexInputLayout,
}

impl Inputs {
    pub fn new<V>(instance_color: bool) -> Self
    where
        V: Vertex,
    {
        let mut shader_location = ModelTransform::LAYOUT_ATTRIBUTES_LEN;
        if instance_color {
            shader_location += ModelColor::LAYOUT_ATTRIBUTES_LEN;
        }

        Self {
            instance_color,
            vertex: VertexInputLayout::new::<V>(shader_location),
        }
    }

    fn buffer_layouts(&self) -> Vec<VertexBufferLayout> {
        if self.instance_color {
            vec![
                ModelTransform::LAYOUT,
                ModelColor::LAYOUT,
                self.vertex.buffer_layout(),
            ]
        } else {
            vec![ModelTransform::LAYOUT, self.vertex.buffer_layout()]
        }
    }
}

struct VertexInputLayout {
    size: usize,
    attrs: Vec<VertexAttribute>,
}

impl VertexInputLayout {
    fn new<V>(mut shader_location: u32) -> Self
    where
        V: Vertex,
    {
        use {
            crate::vertex::{Component, Component2D, Component3D},
            std::mem,
        };

        let mut offset = 0;
        let mut make_attr = |n| {
            let format = Self::format(n);
            let new_offset = Self::f32_offset(n);
            let attr = VertexAttribute {
                format,
                offset,
                shader_location,
            };

            offset += new_offset;
            shader_location += 1;
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

    fn f32_offset(n: u64) -> u64 {
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
