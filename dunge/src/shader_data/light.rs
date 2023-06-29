use {
    crate::{
        _color::{IntoLinear, Linear},
        _shader,
        error::{SourceNotFound, TooManySources},
        layout::Plain,
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

/// Parameters of a light source.
#[derive(Clone, Copy)]
pub struct _Source<C = Linear<f32, 3>> {
    pub pos: [f32; 3],
    pub rad: f32,
    pub col: C,
    pub kind: LightKind,
}

impl<C> _Source<C> {
    pub(crate) fn into_linear(self) -> _Source
    where
        C: IntoLinear<3>,
    {
        _Source {
            pos: self.pos,
            rad: self.rad,
            col: self.col.into_linear(),
            kind: self.kind,
        }
    }
}

/// The light kind.
#[derive(Clone, Copy, Default)]
pub enum LightKind {
    #[default]
    Glow,
    Gloom,
}

pub(crate) struct Light {
    ambient_buffer: Buffer,
    sources_buffer: Buffer,
    n_sources: usize,
    bind_group: BindGroup,
}

impl Light {
    pub fn new(
        ambient: [f32; 3],
        srcs: &[_SourceModel],
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Result<Self, TooManySources> {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        if srcs.len() > _shader::MAX_N_SOURCES as usize {
            return Err(TooManySources);
        }

        let ambient_buffer = {
            let [r, g, b] = ambient;
            let ambient_buffer = [r, g, b, 0.];
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: ambient_buffer.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let sources_buffer = {
            let buf = Box::new(Sources::from_slice(srcs));
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("sources buffer"),
                contents: buf.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: _shader::AMBIENT_BINDING,
                    resource: ambient_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: _shader::SOURCES_BINDING,
                    resource: sources_buffer.as_entire_binding(),
                },
            ],
            label: Some("lights bind group"),
        });

        Ok(Self {
            ambient_buffer,
            sources_buffer,
            n_sources: srcs.len(),
            bind_group,
        })
    }

    pub fn update_sources(
        &mut self,
        ambient: [f32; 3],
        srcs: &[_SourceModel],
        queue: &Queue,
    ) -> Result<(), TooManySources> {
        use std::mem;

        if srcs.len() > _shader::MAX_N_SOURCES as usize {
            return Err(TooManySources);
        }

        if !srcs.is_empty() {
            queue.write_buffer(&self.sources_buffer, 0, srcs.as_bytes());
        }

        if self.n_sources != srcs.len() {
            let len = srcs.len() as u32;
            queue.write_buffer(
                &self.sources_buffer,
                mem::size_of::<[_SourceModel; 64]>() as _,
                len.as_bytes(),
            );

            self.n_sources = srcs.len();
        }

        queue.write_buffer(&self.ambient_buffer, 0, ambient.as_bytes());
        Ok(())
    }

    pub fn update_nth(
        &self,
        n: usize,
        source: _SourceModel,
        queue: &Queue,
    ) -> Result<(), SourceNotFound> {
        use std::mem;

        if n >= self.n_sources {
            return Err(SourceNotFound);
        }

        queue.write_buffer(
            &self.sources_buffer,
            (mem::size_of::<_SourceModel>() * n) as _,
            source.as_bytes(),
        );

        Ok(())
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(crate) struct _SourceModel {
    pos: [f32; 3],
    rad: f32,
    col: [f32; 3],
    flags: u32,
}

impl _SourceModel {
    pub fn new(src: _Source) -> Self {
        Self {
            pos: src.pos,
            rad: src.rad,
            col: src.col.0,
            flags: match src.kind {
                LightKind::Glow => 0,
                LightKind::Gloom => 1,
            },
        }
    }
}

unsafe impl Plain for _SourceModel {}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct Sources {
    data: [_SourceModel; 64],
    len: u32,
    pad: [u32; 3],
}

impl Sources {
    fn from_slice(slice: &[_SourceModel]) -> Self {
        let mut sources = Self::default();
        sources.data[..slice.len()].copy_from_slice(slice);
        sources.len = slice.len() as u32;
        sources
    }
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            data: [_SourceModel::default(); 64],
            len: 0,
            pad: [0; 3],
        }
    }
}

unsafe impl Plain for Sources {}
