use {
    crate::{
        color::{IntoLinear, Linear},
        error::{SourceNotFound, TooManySources},
        layout::Plain,
        shader,
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

/// Parameters of a light source.
#[derive(Clone, Copy)]
pub struct Source<C = Linear<f32, 3>> {
    pub pos: [f32; 3],
    pub rad: f32,
    pub col: C,
    pub mode: LightMode,
    pub kind: LightKind,
}

impl<C> Source<C> {
    pub(crate) fn into_linear(self) -> Source
    where
        C: IntoLinear<3>,
    {
        Source {
            pos: self.pos,
            rad: self.rad,
            col: self.col.into_linear(),
            mode: self.mode,
            kind: self.kind,
        }
    }
}

/// The light mode.
#[derive(Clone, Copy, Default)]
pub enum LightMode {
    #[default]
    Smooth,
    Sharp,
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
        srcs: &[SourceModel],
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Result<Self, TooManySources> {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        if srcs.len() > shader::MAX_N_SOURCES as usize {
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
                    binding: shader::AMBIENT_BINDING,
                    resource: ambient_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::SOURCES_BINDING,
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

    pub fn update_ambient(&self, col: [f32; 3], queue: &Queue) {
        queue.write_buffer(&self.ambient_buffer, 0, col.as_bytes());
    }

    pub fn update_sources(&self, srcs: &[SourceModel], queue: &Queue) -> bool {
        if srcs.is_empty() || self.n_sources != srcs.len() {
            return false;
        }

        queue.write_buffer(&self.sources_buffer, 0, srcs.as_bytes());
        true
    }

    pub fn update_nth(
        &self,
        n: usize,
        source: SourceModel,
        queue: &Queue,
    ) -> Result<(), SourceNotFound> {
        use std::mem;

        if n >= self.n_sources {
            return Err(SourceNotFound);
        }

        queue.write_buffer(
            &self.sources_buffer,
            (mem::size_of::<SourceModel>() * n) as _,
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
pub(crate) struct SourceModel {
    pos: [f32; 3],
    rad: f32,
    col: [f32; 3],
    flags: u32,
}

impl SourceModel {
    pub fn new(src: Source) -> Self {
        Self {
            pos: src.pos,
            rad: src.rad,
            col: src.col.0,
            flags: {
                let sharp = match src.mode {
                    LightMode::Smooth => 0,
                    LightMode::Sharp => 1,
                };

                let gloom = match src.kind {
                    LightKind::Glow => 0,
                    LightKind::Gloom => 1,
                };

                sharp | gloom << 1
            },
        }
    }
}

unsafe impl Plain for SourceModel {}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct Sources {
    data: [SourceModel; 64],
    len: u32,
    pad: [u32; 3],
}

impl Sources {
    fn from_slice(slice: &[SourceModel]) -> Self {
        let mut sources = Self::default();
        sources.data[..slice.len()].copy_from_slice(slice);
        sources.len = slice.len() as u32;
        sources
    }
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            data: [SourceModel::default(); 64],
            len: 0,
            pad: [0; 3],
        }
    }
}

unsafe impl Plain for Sources {}
