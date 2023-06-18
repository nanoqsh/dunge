use {
    crate::{
        error::ResourceNotFound,
        handles::{LayerHandle, LightsHandle},
        layout::Plain,
        pipeline::LightsBindings as Bindings,
        render::Render,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::{Source, SourceArray},
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device},
};

pub(crate) struct Lights {
    group: u32,
    bind_group: BindGroup,
    source_arrays: Vec<(SourceArray, SourceArrayBuffers)>,
}

impl Lights {
    pub fn new(params: Parameters, device: &Device) -> Self {
        use {
            std::iter,
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                BindGroupDescriptor, BindGroupEntry, BufferUsages,
            },
        };

        let Parameters {
            bindings,
            variables,
            layout,
        } = params;

        let source_arrays: Vec<_> = iter::zip(variables.source_arrays, &bindings.source_arrays)
            .map(|(var, bind)| {
                let array = SourceArray::new(var, bind.size as usize);
                let array_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("source array buffer"),
                    contents: array.buf().as_bytes(),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                let len_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("source len buffer"),
                    contents: &array.len().to_ne_bytes(),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                (
                    array,
                    SourceArrayBuffers {
                        array: array_buf,
                        len: len_buf,
                    },
                )
            })
            .collect();

        let entries: Vec<_> = iter::zip(&source_arrays, &bindings.source_arrays)
            .flat_map(|((_, buf), bind)| {
                [
                    BindGroupEntry {
                        binding: bind.binding_array,
                        resource: buf.array.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: bind.binding_len,
                        resource: buf.len.as_entire_binding(),
                    },
                ]
            })
            .collect();

        Self {
            group: bindings.group,
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("lights bind group"),
            }),
            source_arrays,
        }
    }

    pub fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

struct SourceArrayBuffers {
    array: Buffer,
    len: Buffer,
}

pub(crate) struct Parameters<'a> {
    pub variables: Variables<'a>,
    pub bindings: &'a Bindings,
    pub layout: &'a BindGroupLayout,
}

#[derive(Default)]
pub(crate) struct Variables<'a> {
    pub source_arrays: Vec<&'a [Source]>,
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    render: &'a Render,
    variables: Variables<'a>,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, render: &'a Render) -> Self {
        Self {
            resources,
            render,
            variables: Variables::default(),
        }
    }

    pub fn with_sources(mut self, sources: &'a [Source]) -> Self {
        self.variables.source_arrays.push(sources);
        self
    }

    pub fn build<S>(self, handle: LayerHandle<S>) -> Result<LightsHandle<S>, ResourceNotFound>
    where
        S: Shader,
    {
        let actual = self.variables.source_arrays.len();
        let expected = ShaderInfo::new::<S>().source_arrays;

        assert_eq!(
            actual, expected,
            "the shader requires {expected} source arrays, but {actual} is set",
        );

        self.resources
            .create_lights(self.render, self.variables, handle)
    }
}
