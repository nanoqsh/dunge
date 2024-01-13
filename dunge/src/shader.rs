use {
    crate::{
        bind::TypedGroup,
        sl::{InputInfo, InstInfo, IntoModule, Module, Stages, VertInfo},
        state::State,
        types::{MemberType, VectorType},
    },
    std::{cell::Cell, marker::PhantomData},
    wgpu::{
        BufferAddress, PipelineLayout, ShaderModule, VertexAttribute, VertexBufferLayout,
        VertexStepMode,
    },
};

pub struct Shader<V, I> {
    inner: Inner,
    ty: PhantomData<(V, I)>,
}

impl<V, I> Shader<V, I> {
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

        fn layout(vert: &Vertex) -> VertexBufferLayout {
            VertexBufferLayout {
                array_stride: vert.array_stride,
                step_mode: vert.step_mode,
                attributes: &vert.attributes,
            }
        }

        self.inner.vertex.iter().map(layout).collect()
    }

    pub(crate) fn slots(&self) -> Slots {
        self.inner.slots
    }

    pub(crate) fn groups(&self) -> &[TypedGroup] {
        &self.inner.groups
    }
}

struct Vertex {
    array_stride: BufferAddress,
    step_mode: VertexStepMode,
    attributes: Box<[VertexAttribute]>,
}

#[derive(Clone, Copy)]
pub(crate) struct Slots {
    pub vertex: u32,
    pub instance: u32,
}

struct Inner {
    id: usize,
    module: ShaderModule,
    layout: PipelineLayout,
    vertex: Box<[Vertex]>,
    slots: Slots,
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
            for (binding, member) in iter::zip(0.., info.def) {
                let entry = match member {
                    MemberType::Scalar(_) | MemberType::Vector(_) | MemberType::Matrix(_) => {
                        BindGroupLayoutEntry {
                            binding,
                            visibility: visibility(info.stages),
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    }
                    MemberType::Tx2df => BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    MemberType::Sampl => BindGroupLayoutEntry {
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

        let to_format = |vecty| match vecty {
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

        let next_location = {
            let location = Cell::default();
            move || {
                let current = location.get();
                location.set(current + 1);
                current
            }
        };

        let vert = |info: VertInfo| {
            let mut offset = 0;
            let attr = |vecty| {
                let format = to_format(vecty);
                let attr = VertexAttribute {
                    format,
                    offset,
                    shader_location: next_location(),
                };

                offset += format.size();
                attr
            };

            Vertex {
                array_stride: info.size as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: info.def.into_iter().map(attr).collect(),
            }
        };

        let inst = |info: InstInfo| {
            let format = to_format(info.vecty);
            let attr = VertexAttribute {
                format,
                offset: 0,
                shader_location: next_location(),
            };

            Vertex {
                array_stride: format.size(),
                step_mode: VertexStepMode::Instance,
                attributes: Box::from([attr]),
            }
        };

        let mut set_instance = true;
        let mut slots = Slots {
            vertex: 0,
            instance: 0,
        };

        let mut vertex = Vec::with_capacity(cx.count_input());
        for input in cx.input() {
            match input {
                InputInfo::Vert(v) => {
                    slots.vertex = vertex.len() as u32;
                    vertex.push(vert(v));
                }
                InputInfo::Inst(i) => {
                    if set_instance {
                        slots.instance = vertex.len() as u32;
                        set_instance = false;
                    }

                    vertex.push(inst(i));
                }
                InputInfo::Index => {}
            }
        }

        Self {
            id: state.next_shader_id(),
            module,
            layout,
            vertex: Box::from(vertex),
            slots,
            groups,
        }
    }
}
