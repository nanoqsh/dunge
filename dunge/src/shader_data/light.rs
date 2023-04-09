use {
    crate::{layout::Plain, r#loop::Error, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

#[derive(Clone, Copy, Default)]
pub struct Source {
    pub pos: [f32; 3],
    pub rad: f32,
    pub col: [f32; 3],
    pub mode: LightMode,
    pub kind: LightKind,
}

#[derive(Clone, Copy, Default)]
pub enum LightMode {
    #[default]
    Smooth,
    Sharp,
}

#[derive(Clone, Copy, Default)]
pub enum LightKind {
    #[default]
    Glow,
    Gloom,
}

pub(crate) struct Light {
    sources_buffer: Buffer,
    n_sources: usize,
    bind_group: BindGroup,
}

impl Light {
    const MAX_N_SOURCES: usize = 64;

    pub fn new(
        srcs: &[SourceModel],
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Result<Self, Error> {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        if srcs.len() > Self::MAX_N_SOURCES {
            return Err(Error::TooManySources);
        }

        let sources_buffer = {
            let default = [SourceModel::default()];
            let uniform = if srcs.is_empty() { &default } else { srcs };
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("sources buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let n_sources_buffer = {
            let len = srcs.len() as u32;
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("n sources buffer"),
                contents: len.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::SOURCES_BINDING,
                    resource: sources_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::N_SOURCES_BINDING,
                    resource: n_sources_buffer.as_entire_binding(),
                },
            ],
            label: Some("lights bind group"),
        });

        Ok(Self {
            sources_buffer,
            n_sources: srcs.len(),
            bind_group,
        })
    }

    pub fn update(&self, srcs: &[SourceModel], queue: &Queue) -> bool {
        if srcs.is_empty() || self.n_sources != srcs.len() {
            return false;
        }

        queue.write_buffer(&self.sources_buffer, 0, srcs.as_bytes());
        true
    }

    pub fn update_nth(&self, n: usize, source: SourceModel, queue: &Queue) -> Result<(), Error> {
        use std::mem;

        if n >= self.n_sources {
            return Err(Error::SourceNotFound);
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
            col: src.col,
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
