use {
    crate::state::State,
    dunge_shade::{
        irc::Stage,
        link::{Render, RenderInput},
        module::{Format, GroupFormat, Info, TextureDimension, VertexFormat},
    },
    std::{borrow::Cow, iter, marker::PhantomData, sync::Arc},
};

/// Alias of render [shader](Shader).
pub type RenderShader<S = (), V = (), I = ()> = Shader<RenderInput<V, I>, S>;

/// The shader type.
///
/// Can be created using the context's [`make_shader`](crate::Context::make_shader) function.
pub struct Shader<I, S> {
    data: Arc<ShaderData>,
    kind: PhantomData<(I, S)>,
}

impl<I, S> Shader<I, S> {
    pub(crate) fn new(state: &State, render: Render<I, S>) -> Self {
        let module = render.module();
        Self {
            data: Arc::new(make(state, module.info, module.naga)),
            kind: PhantomData,
        }
    }

    pub(crate) fn data(&self) -> &ShaderData {
        &self.data
    }
}

impl<I, S> Clone for Shader<I, S> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            kind: PhantomData,
        }
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

fn make(state: &State, info: Info, naga: wgpu::naga::Module) -> ShaderData {
    let module = {
        let desc = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Naga(Cow::Owned(naga)),
        };

        state.device().create_shader_module(desc)
    };

    let mut entries = vec![];
    let mut groups = vec![];
    for group in info.groups() {
        let visibility = group
            .stages()
            .map(|s| match s {
                Stage::Regular => wgpu::ShaderStages::empty(),
                Stage::Vertex => wgpu::ShaderStages::VERTEX,
                Stage::Fragment => wgpu::ShaderStages::FRAGMENT,
            })
            .collect();

        for (binding, format) in iter::zip(0.., group.bindings()) {
            let entry = match format {
                GroupFormat::Texture { dim, scalar } => {
                    let sample_type = match scalar {
                        Format::Float32 => wgpu::TextureSampleType::Float { filterable: true },
                        Format::Uint32 => wgpu::TextureSampleType::Uint,
                        Format::Sint32 => wgpu::TextureSampleType::Sint,
                    };

                    let view_dimension = match dim {
                        TextureDimension::D1 => wgpu::TextureViewDimension::D1,
                        TextureDimension::D2 => wgpu::TextureViewDimension::D2,
                        TextureDimension::D3 => wgpu::TextureViewDimension::D3,
                    };

                    wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility,
                        ty: wgpu::BindingType::Texture {
                            sample_type,
                            view_dimension,
                            multisampled: false,
                        },
                        count: None,
                    }
                }
                GroupFormat::Sampler => wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                GroupFormat::Uniform => wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                GroupFormat::Storage => wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
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
        entries.clear();
    }

    let groups = groups.into_boxed_slice();
    let layout = {
        let groups: Vec<_> = groups.iter().map(|g| Some(g.as_ref())).collect();
        let desc = wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &groups,
            ..Default::default()
        };

        state.device().create_pipeline_layout(&desc)
    };

    fn make_attrs<I>(iter: I, location: &mut u32) -> (Vec<wgpu::VertexAttribute>, u64)
    where
        I: ExactSizeIterator<Item = VertexFormat>,
    {
        let mut vertex_attributes = Vec::with_capacity(iter.len());
        let mut offset = 0;
        for format in iter {
            let format = match format {
                VertexFormat::Scalar(Format::Float32) => wgpu::VertexFormat::Float32,
                VertexFormat::Scalar(Format::Uint32) => wgpu::VertexFormat::Uint32,
                VertexFormat::Scalar(Format::Sint32) => wgpu::VertexFormat::Sint32,
                VertexFormat::Vec2(Format::Float32) => wgpu::VertexFormat::Float32x2,
                VertexFormat::Vec2(Format::Uint32) => wgpu::VertexFormat::Uint32x2,
                VertexFormat::Vec2(Format::Sint32) => wgpu::VertexFormat::Sint32x2,
                VertexFormat::Vec3(Format::Float32) => wgpu::VertexFormat::Float32x3,
                VertexFormat::Vec3(Format::Uint32) => wgpu::VertexFormat::Uint32x3,
                VertexFormat::Vec3(Format::Sint32) => wgpu::VertexFormat::Sint32x3,
                VertexFormat::Vec4(Format::Float32) => wgpu::VertexFormat::Float32x4,
                VertexFormat::Vec4(Format::Uint32) => wgpu::VertexFormat::Uint32x4,
                VertexFormat::Vec4(Format::Sint32) => wgpu::VertexFormat::Sint32x4,
            };

            let attr = wgpu::VertexAttribute {
                format,
                offset,
                shader_location: *location,
            };

            offset += format.size();
            *location += 1;
            vertex_attributes.push(attr);
        }

        (vertex_attributes, offset)
    }

    let mut location = 0;
    let (vertex_attributes, vertex_offset) = make_attrs(info.vertex(), &mut location);

    let mut vertex = vec![];
    if !vertex_attributes.is_empty() {
        vertex.push(Vertex {
            array_stride: vertex_offset,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: vertex_attributes.into_boxed_slice(),
        });
    }

    let instance_slot = vertex.len() as u32;
    for format in info.instance() {
        let (instance_attributes, instance_offset) = make_attrs(iter::once(format), &mut location);
        vertex.push(Vertex {
            array_stride: instance_offset,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: instance_attributes.into_boxed_slice(),
        });
    }

    let slots = SlotNumbers {
        vertex: 0,
        instance: instance_slot,
    };

    ShaderData {
        module,
        groups,
        layout,
        vertex: vertex.into_boxed_slice(),
        slots,
    }
}

pub(crate) struct ShaderData {
    module: wgpu::ShaderModule,
    groups: Box<[Arc<wgpu::BindGroupLayout>]>,
    layout: wgpu::PipelineLayout,
    vertex: Box<[Vertex]>,
    slots: SlotNumbers,
}

impl ShaderData {
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
