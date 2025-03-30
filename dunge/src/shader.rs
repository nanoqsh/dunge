use {
    crate::{
        bind::TypedGroup,
        sl::{ComputeInput, InputInfo, IntoModule, Module, RenderInput, Stages},
        state::State,
        types::{MemberType, ScalarType, ValueType, VectorType},
    },
    std::{cell::Cell, marker::PhantomData, mem},
    wgpu::{
        BufferAddress, PipelineLayout, ShaderModule, VertexAttribute, VertexBufferLayout,
        VertexFormat, VertexStepMode,
    },
};

pub type RenderShader<V, I> = Shader<RenderInput<V, I>>;
pub type ComputeShader = Shader<ComputeInput>;

/// The shader type.
///
/// Can be created using the context's [`make_shader`](crate::Context::make_shader) function.
pub struct Shader<I> {
    data: ShaderData,
    wgsl: String,
    kind: PhantomData<I>,
}

impl<I> Shader<I> {
    pub(crate) fn new<M, A, K>(state: &State, module: M) -> Self
    where
        M: IntoModule<A, K, Input = I>,
    {
        let mut module = module.into_module();
        let wgsl = mem::take(&mut module.wgsl);
        Self {
            data: ShaderData::new(state, module),
            wgsl,
            kind: PhantomData,
        }
    }

    /// Debug generated wgsl shader.
    ///
    /// Is empty when the `wgsl` feature is disabled.
    pub fn debug_wgsl(&self) -> &str {
        &self.wgsl
    }

    pub(crate) fn data(&self) -> &ShaderData {
        &self.data
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

pub(crate) struct ShaderData {
    id: usize,
    module: ShaderModule,
    layout: PipelineLayout,
    vertex: Box<[Vertex]>,
    slots: Slots,
    groups: Box<[TypedGroup]>,
}

impl ShaderData {
    fn new(state: &State, Module { cx, nm, .. }: Module) -> Self {
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
            out.set(ShaderStages::COMPUTE, stages.cs);
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
                    MemberType::Array(_) => BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
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

        let location = Cell::default();
        let next_location = || {
            let current = location.get();
            location.set(current + 1);
            current
        };

        let make_attr = || {
            let mut offset = 0;
            move |ty, attrs: &mut Vec<_>| {
                let mut f = |format| {
                    let attr = VertexAttribute {
                        format,
                        offset,
                        shader_location: next_location(),
                    };

                    offset += format.size();
                    attrs.push(attr);
                };

                to_format(ty, &mut f);
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

                    let vert = {
                        let mut attr = make_attr();
                        let mut attrs = vec![];
                        for vecty in v.def {
                            attr(ValueType::Vector(vecty), &mut attrs);
                        }

                        Vertex {
                            array_stride: v.size as BufferAddress,
                            step_mode: VertexStepMode::Vertex,
                            attributes: attrs.into(),
                        }
                    };

                    vertex.push(vert);
                }
                InputInfo::Inst(i) => {
                    if set_instance {
                        slots.instance = vertex.len() as u32;
                        set_instance = false;
                    }

                    let mut attr = make_attr();
                    let mut attrs = vec![];
                    attr(i.ty, &mut attrs);
                    let vert = Vertex {
                        array_stride: attrs.iter().map(|attr| attr.format.size()).sum(),
                        step_mode: VertexStepMode::Instance,
                        attributes: attrs.into(),
                    };

                    vertex.push(vert);
                }
                InputInfo::Index | InputInfo::GlobalInvocationId => {}
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

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub(crate) fn layout(&self) -> &PipelineLayout {
        &self.layout
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

        self.vertex.iter().map(layout).collect()
    }

    pub(crate) fn slots(&self) -> Slots {
        self.slots
    }

    pub(crate) fn groups(&self) -> &[TypedGroup] {
        &self.groups
    }
}

fn to_format<F>(ty: ValueType, f: &mut F)
where
    F: FnMut(VertexFormat),
{
    match ty {
        ValueType::Scalar(ScalarType::Float) => f(VertexFormat::Float32),
        ValueType::Scalar(ScalarType::Sint) => f(VertexFormat::Sint32),
        ValueType::Scalar(ScalarType::Uint) | ValueType::Scalar(ScalarType::Bool) => {
            f(VertexFormat::Uint32);
        }
        ValueType::Vector(VectorType::Vec2f) => f(VertexFormat::Float32x2),
        ValueType::Vector(VectorType::Vec3f) => f(VertexFormat::Float32x3),
        ValueType::Vector(VectorType::Vec4f) => f(VertexFormat::Float32x4),
        ValueType::Vector(VectorType::Vec2u) => f(VertexFormat::Uint32x2),
        ValueType::Vector(VectorType::Vec3u) => f(VertexFormat::Uint32x3),
        ValueType::Vector(VectorType::Vec4u) => f(VertexFormat::Uint32x4),
        ValueType::Vector(VectorType::Vec2i) => f(VertexFormat::Sint32x2),
        ValueType::Vector(VectorType::Vec3i) => f(VertexFormat::Sint32x3),
        ValueType::Vector(VectorType::Vec4i) => f(VertexFormat::Sint32x4),
        ValueType::Matrix(mat) => {
            for _ in 0..mat.dims() {
                to_format(ValueType::Vector(mat.vector_type()), f);
            }
        }
        ValueType::Array(_) => todo!(),
    }
}
