use {
    crate::{
        layer::Layer,
        pipeline::Lights as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::source::{SetLenError, Source, SourceArray, UpdateError as ArrayUpdateError},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Queue},
};

/// Shader lights.
///
/// Can be created from the [context](crate::Context) by calling
/// the [`lights_builder`](crate::Context::lights_builder) function.
pub struct Lights<S> {
    group: u32,
    bind_group: BindGroup,
    source_arrays: Vec<(SourceArray, SourceArrayBuffers)>,
    queue: Arc<Queue>,
    ty: PhantomData<S>,
}

impl<S> Lights<S> {
    fn new(params: Parameters, state: &State) -> Self {
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

        let device = state.device();
        let source_arrays: Vec<_> = iter::zip(variables.source_arrays, &bindings.source_arrays)
            .map(|(var, bind)| {
                let array = SourceArray::new(var, bind.size as usize);
                let array_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("source array buffer"),
                    contents: bytemuck::cast_slice(array.sources()),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                let len_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("source len buffer"),
                    contents: bytemuck::cast_slice(&[array.len()]),
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
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    /// Updates the lights with new [sources](`Source`).
    ///
    /// # Errors
    /// Will return
    /// - [`UpdateError::Index`] if the index is invalid.
    /// - [`UpdateError::Array`] if wrong offset or slice with wrong length is passed.
    /// - [`UpdateError::SetLen`] if the new length exceeds the maximum available.
    ///
    /// # Panics
    /// Panics if the shader has no light sources.
    pub fn update_sources(
        &mut self,
        index: usize,
        offset: usize,
        sources: &[Source],
    ) -> Result<(), UpdateError>
    where
        S: Shader,
    {
        use std::mem;

        let info = ShaderInfo::new::<S>();
        assert!(info.has_lights(), "the shader has no light sources");

        let (array, buffers) = self
            .source_arrays
            .get_mut(index)
            .ok_or(UpdateError::Index)?;

        array.update(offset, sources)?;
        let data = &array.sources()[offset..];
        self.queue.write_buffer(
            &buffers.array,
            (offset * mem::size_of::<Source>()) as _,
            bytemuck::cast_slice(data),
        );

        let old_len = array.len();
        let new_len = (offset + sources.len()) as u32;
        if old_len.get() < new_len {
            array.set_len(new_len)?;
            self.queue
                .write_buffer(&buffers.len, 0, bytemuck::cast_slice(&[array.len()]));
        }

        Ok(())
    }

    pub(crate) fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

/// An error returned from the [`update_sources`](Lights::update_sources) function.
#[derive(Debug)]
pub enum UpdateError {
    /// The index is invalid.
    Index,

    /// Wrong offset or slice with wrong length.
    Array(ArrayUpdateError),

    /// The new length exceeds the maximum available.
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

struct Parameters<'a> {
    variables: Variables,
    bindings: &'a Bindings,
    layout: &'a BindGroupLayout,
}

#[derive(Default)]
struct Variables {
    source_arrays: Vec<Vec<Source>>,
}

/// The [lights](Lights) builder.
#[must_use]
pub struct Builder<'a> {
    state: &'a State,
    variables: Variables,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self {
            state,
            variables: Variables::default(),
        }
    }

    /// Sets light [sources](Source) for the lights object.
    pub fn with_sources(mut self, sources: Vec<Source>) -> Self {
        self.variables.source_arrays.push(sources);
        self
    }

    /// Builds the lights.
    ///
    /// # Panics
    /// Panics if the shader requires source arrays, but they aren't set.
    #[must_use]
    pub fn build<S, T>(self, layer: &Layer<S, T>) -> Lights<S>
    where
        S: Shader,
    {
        let actual = self.variables.source_arrays.len();
        let info = ShaderInfo::new::<S>();
        let expected = info.source_arrays();

        assert_eq!(
            actual, expected,
            "the shader requires {expected} source arrays, but {actual} is set",
        );

        let lights = layer.pipeline().lights().expect("the shader has no lights");
        let params = Parameters {
            variables: self.variables,
            bindings: &lights.bindings,
            layout: &lights.layout,
        };

        Lights::new(params, self.state)
    }
}
