use {
    crate::{
        group::GroupMemberType,
        group::TypedGroup,
        sl::{InputInfo, IntoModule, Module, Stages},
        state::State,
        vertex::VectorType,
    },
    std::marker::PhantomData,
    wgpu::{PipelineLayout, ShaderModule, VertexAttribute, VertexBufferLayout},
};

pub struct Shader<V> {
    inner: Inner,
    ty: PhantomData<V>,
}

impl<V> Shader<V> {
    pub(crate) fn new<M, A>(state: &State, module: M) -> Self
    where
        M: IntoModule<A, Vertex = V>,
    {
        Self {
            inner: Inner::new(state, module.into_module()),
            ty: PhantomData,
        }
    }

    pub(crate) fn id(&self) -> usize {
        self.inner.id
    }

    pub(crate) fn module(&self) -> &ShaderModule {
        &self.inner.module
    }

    pub(crate) fn layout(&self) -> &PipelineLayout {
        &self.inner.layout
    }

    pub(crate) fn buffers(&self) -> Box<[VertexBufferLayout]> {
        use wgpu::*;

        fn layout(Vertex { size, attributes }: &Vertex) -> VertexBufferLayout {
            VertexBufferLayout {
                array_stride: *size as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes,
            }
        }

        self.inner.vertex.iter().map(layout).collect()
    }

    pub(crate) fn groups(&self) -> &[TypedGroup] {
        &self.inner.groups
    }
}

struct Vertex {
    size: usize,
    attributes: Box<[VertexAttribute]>,
}

struct Inner {
    id: usize,
    module: ShaderModule,
    layout: PipelineLayout,
    vertex: Box<[Vertex]>,
    groups: Box<[TypedGroup]>,
}

impl Inner {
    fn new(state: &State, Module { cx, nm }: Module) -> Self {
        use {
            std::{borrow::Cow, iter},
            wgpu::*,
        };

        let module = {
            let desc = ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Naga(Cow::Owned(nm)),
            };

            state.device().create_shader_module(desc)
        };

        let visibility = |stages: Stages| {
            let mut out = ShaderStages::empty();
            out.set(ShaderStages::VERTEX, stages.vs);
            out.set(ShaderStages::FRAGMENT, stages.fs);
            out
        };

        let mut entries = vec![];
        let mut groups = vec![];
        for info in cx.groups() {
            entries.clear();
            for (binding, member) in iter::zip(0.., info.decl) {
                let entry = match member {
                    GroupMemberType::Tx2df => BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    GroupMemberType::Sampl => BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                };

                entries.push(entry);
            }

            let desc = BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            };

            let bind = state.device().create_bind_group_layout(&desc);
            let layout = TypedGroup::new(info.tyid, bind);
            groups.push(layout);
        }

        let groups = groups.into_boxed_slice();
        let layout = {
            let groups: Vec<_> = groups.iter().map(TypedGroup::bind).collect();
            let desc = PipelineLayoutDescriptor {
                bind_group_layouts: &groups,
                ..Default::default()
            };

            state.device().create_pipeline_layout(&desc)
        };

        let vertex = {
            let vert = |info: InputInfo| {
                let mut offset = 0;
                let mut shader_location = 0;
                let attr = |vecty| {
                    let format = match vecty {
                        VectorType::Vec2f => VertexFormat::Float32x2,
                        VectorType::Vec3f => VertexFormat::Float32x3,
                        VectorType::Vec4f => VertexFormat::Float32x4,
                        VectorType::Vec2u => VertexFormat::Uint32x2,
                        VectorType::Vec3u => VertexFormat::Uint32x3,
                        VectorType::Vec4u => VertexFormat::Uint32x4,
                        VectorType::Vec2i => VertexFormat::Sint32x2,
                        VectorType::Vec3i => VertexFormat::Sint32x3,
                        VectorType::Vec4i => VertexFormat::Sint32x4,
                    };

                    let attr = VertexAttribute {
                        format,
                        offset,
                        shader_location,
                    };

                    offset += format.size();
                    shader_location += 1;
                    attr
                };

                Vertex {
                    size: info.size,
                    attributes: info.decl.into_iter().map(attr).collect(),
                }
            };

            cx.inputs().map(vert).collect()
        };

        Self {
            id: state.next_shader_id(),
            module,
            layout,
            vertex,
            groups,
        }
    }
}
