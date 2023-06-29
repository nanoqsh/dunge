use {
    crate::{
        error::ResourceNotFound,
        handles::{LayerHandle, LightsHandle},
        layout::Plain,
        pipeline::Lights as Bindings,
        render::Render,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::{
            source::{SetLenError, SourceUniform, UpdateError as ArrayUpdateError},
            Source, SourceArray,
        },
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
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
                    contents: array.len().as_bytes(),
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

    pub fn update_array(
        &mut self,
        index: usize,
        offset: usize,
        sources: &[SourceUniform],
        queue: &Queue,
    ) -> Result<(), UpdateError> {
        use std::mem;

        let (array, buffers) = self
            .source_arrays
            .get_mut(index)
            .ok_or(UpdateError::Index)?;

        array.update(offset, sources)?;
        let data = &array.buf()[offset..];
        queue.write_buffer(
            &buffers.array,
            (offset * mem::size_of::<SourceUniform>()) as _,
            data.as_bytes(),
        );

        let old_len = array.len();
        let new_len = (offset + sources.len()) as u32;
        if old_len.get() < new_len {
            array.set_len(new_len)?;
            queue.write_buffer(&buffers.len, 0, array.len().as_bytes());
        }

        Ok(())
    }

    pub fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

#[derive(Debug)]
pub enum UpdateError {
    Index,
    Array(ArrayUpdateError),
    SetLen(SetLenError),
}

impl From<ArrayUpdateError> for UpdateError {
    fn from(v: ArrayUpdateError) -> Self {
        Self::Array(v)
    }
}

impl From<SetLenError> for UpdateError {
    fn from(v: SetLenError) -> Self {
        Self::SetLen(v)
    }
}

struct SourceArrayBuffers {
    array: Buffer,
    len: Buffer,
}

pub(crate) struct Parameters<'a> {
    pub variables: Variables,
    pub bindings: &'a Bindings,
    pub layout: &'a BindGroupLayout,
}

#[derive(Default)]
pub(crate) struct Variables {
    pub source_arrays: Vec<Vec<SourceUniform>>,
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    render: &'a Render,
    variables: Variables,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, render: &'a Render) -> Self {
        Self {
            resources,
            render,
            variables: Variables::default(),
        }
    }

    pub fn with_sources<I>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = Source>,
    {
        self.variables
            .source_arrays
            .push(sources.into_iter().map(Source::into_uniform).collect());

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
