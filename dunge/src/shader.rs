use {
    crate::{
        sl::{ComputeInput, InputInfo, IntoModule, Module, RenderInput, Stages},
        state::State,
        types::{MemberType, ScalarType, ValueType, VectorType},
    },
    std::{borrow::Cow, cell::Cell, iter, marker::PhantomData, mem, sync::Arc},
};

pub type RenderShader<V, I, S> = Shader<RenderInput<V, I>, S>;
pub type ComputeShader<S> = Shader<ComputeInput, S>;

/// The shader type.
///
/// Can be created using the context's [`make_shader`](crate::Context::make_shader) function.
pub struct Shader<I, S> {
    data: ShaderData,
    wgsl: String,
    kind: PhantomData<(I, S)>,
}

impl<I, S> Shader<I, S> {
    pub(crate) fn new<M, A, K>(state: &State, module: M) -> Self
    where
        M: IntoModule<A, K, Input = I, Set = S>,
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
    array_stride: wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode,
    attributes: Box<[wgpu::VertexAttribute]>,
}

#[derive(Clone, Copy)]
pub(crate) struct SlotNumbers {
    pub vertex: u32,
    pub instance: u32,
}

pub(crate) struct ShaderData {
    module: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    vertex: Box<[Vertex]>,
    slots: SlotNumbers,
    groups: Box<[Arc<wgpu::BindGroupLayout>]>,
}

impl ShaderData {
    fn new(state: &State, Module { cx, nm, .. }: Module) -> Self {
        let module = {
            let desc = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(Cow::Owned(nm)),
            };

            state.device().create_shader_module(desc)
        };

        let visibility = |stages: Stages| {
            let mut out = wgpu::ShaderStages::empty();
            out.set(wgpu::ShaderStages::VERTEX, stages.vs);
            out.set(wgpu::ShaderStages::FRAGMENT, stages.fs);
            out.set(wgpu::ShaderStages::COMPUTE, stages.cs);
            out
        };

        let mut entries = vec![];
        let mut groups = vec![];
        for info in cx.groups() {
            entries.clear();
            for (binding, member) in iter::zip(0.., info.def) {
                let entry = match member.ty {
                    MemberType::Scalar(_) | MemberType::Vector(_) | MemberType::Matrix(_) => {
                        wgpu::BindGroupLayoutEntry {
                            binding,
                            visibility: visibility(info.stages),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    }
                    MemberType::Array(_) | MemberType::DynamicArrayType(_) => {
                        wgpu::BindGroupLayoutEntry {
                            binding,
                            visibility: visibility(info.stages),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: !member.mutable,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    }
                    MemberType::Tx2df => wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    MemberType::Sampl => wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: visibility(info.stages),
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                };

                entries.push(entry);
            }

            let desc = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            };

            let bind = state.device().create_bind_group_layout(&desc);
            groups.push(Arc::new(bind));
        }

        let groups = groups.into_boxed_slice();
        let layout = {
            let groups: Vec<_> = groups.iter().map(|g| g.as_ref()).collect();
            let desc = wgpu::PipelineLayoutDescriptor {
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
                    let attr = wgpu::VertexAttribute {
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
        let mut slots = SlotNumbers {
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
                            array_stride: v.size as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
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
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: attrs.into(),
                    };

                    vertex.push(vert);
                }
                InputInfo::Index | InputInfo::GlobalInvocationId => {}
            }
        }

        Self {
            module,
            layout,
            vertex: Box::from(vertex),
            slots,
            groups,
        }
    }

    pub(crate) fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub(crate) fn layout(&self) -> &wgpu::PipelineLayout {
        &self.layout
    }

    pub(crate) fn vertex_buffers(&self) -> Box<[wgpu::VertexBufferLayout<'_>]> {
        use wgpu::*;

        fn layout(vert: &Vertex) -> VertexBufferLayout<'_> {
            VertexBufferLayout {
                array_stride: vert.array_stride,
                step_mode: vert.step_mode,
                attributes: &vert.attributes,
            }
        }

        self.vertex.iter().map(layout).collect()
    }

    pub(crate) fn slots(&self) -> SlotNumbers {
        self.slots
    }

    pub(crate) fn groups(&self) -> &[Arc<wgpu::BindGroupLayout>] {
        &self.groups
    }
}

fn to_format<F>(ty: ValueType, f: &mut F)
where
    F: FnMut(wgpu::VertexFormat),
{
    match ty {
        ValueType::Scalar(ScalarType::Float) => f(wgpu::VertexFormat::Float32),
        ValueType::Scalar(ScalarType::Sint) => f(wgpu::VertexFormat::Sint32),
        ValueType::Scalar(ScalarType::Uint) | ValueType::Scalar(ScalarType::Bool) => {
            f(wgpu::VertexFormat::Uint32);
        }
        ValueType::Vector(VectorType::Vec2f) => f(wgpu::VertexFormat::Float32x2),
        ValueType::Vector(VectorType::Vec3f) => f(wgpu::VertexFormat::Float32x3),
        ValueType::Vector(VectorType::Vec4f) => f(wgpu::VertexFormat::Float32x4),
        ValueType::Vector(VectorType::Vec2u) => f(wgpu::VertexFormat::Uint32x2),
        ValueType::Vector(VectorType::Vec3u) => f(wgpu::VertexFormat::Uint32x3),
        ValueType::Vector(VectorType::Vec4u) => f(wgpu::VertexFormat::Uint32x4),
        ValueType::Vector(VectorType::Vec2i) => f(wgpu::VertexFormat::Sint32x2),
        ValueType::Vector(VectorType::Vec3i) => f(wgpu::VertexFormat::Sint32x3),
        ValueType::Vector(VectorType::Vec4i) => f(wgpu::VertexFormat::Sint32x4),
        ValueType::Matrix(mat) => {
            for _ in 0..mat.dims() {
                to_format(ValueType::Vector(mat.vector_type()), f);
            }
        }
        ValueType::Array(_) => unreachable!(),
    }
}
